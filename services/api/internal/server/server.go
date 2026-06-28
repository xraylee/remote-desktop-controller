// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package server

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	chimw "github.com/go-chi/chi/v5/middleware"
	"github.com/jmoiron/sqlx"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/config"
	"github.com/rdcs/rdcs-api/internal/middleware"
	"github.com/rdcs/rdcs-api/internal/repository"
	"github.com/rdcs/rdcs-api/internal/ws"
)

// Server wraps a chi.Router together with its runtime configuration and data access.
type Server struct {
	router chi.Router
	cfg    *config.Config
	db     *sqlx.DB
	Hub    *ws.Hub

	// Repositories
	Teams       repository.TeamRepository
	Members     repository.MemberRepository
	Devices     repository.DeviceRepository
	Connections repository.ConnectionRecordRepository
	AuditLogs   repository.AuditLogRepository
	Recordings  repository.RecordingRepository
}

// New creates a Server, wires up middleware, repositories, and registers all routes.
func New(cfg *config.Config, db *sqlx.DB) *Server {
	r := chi.NewRouter()

	origins := strings.Split(cfg.CORSOrigins, ",")

	// Global middleware stack (order matters).
	r.Use(middleware.RequestID)
	r.Use(middleware.Logger)
	r.Use(middleware.CORS(origins))
	r.Use(middleware.RateLimit(cfg.RateLimitRPS))
	r.Use(chimw.Recoverer)
	r.Use(chimw.Compress(5))

	s := &Server{
		router:      r,
		cfg:         cfg,
		db:          db,
		Hub:         ws.NewHub(),
		Teams:       repository.NewTeamRepository(db),
		Members:     repository.NewMemberRepository(db),
		Devices:     repository.NewDeviceRepository(db),
		Connections: repository.NewConnectionRecordRepository(db),
		AuditLogs:   repository.NewAuditLogRepository(db),
		Recordings:  repository.NewRecordingRepository(db),
	}
	s.registerRoutes()
	return s
}

// registerRoutes sets up all HTTP endpoints.
func (s *Server) registerRoutes() {
	s.router.Get("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = fmt.Fprint(w, "ok")
	})

	// Auth handler for login (no JWT required).
	authHandler := auth.NewHandler(s.Members, s.cfg.JWTPrivateKey, s.cfg.TOTPIssuer)

	// Public API routes (no authentication required).
	s.router.Route("/api/v1", func(r chi.Router) {
		r.Post("/auth/login", authHandler.HandleLogin)
	})

	// Protected API routes (JWT authentication required).
	s.router.Group(func(r chi.Router) {
		r.Use(middleware.Auth(s.cfg.JWTPublicKey))

		// TOTP two-factor authentication management.
		r.Post("/api/v1/auth/totp/setup", authHandler.HandleTOTPSetup)
		r.Post("/api/v1/auth/totp/verify", authHandler.HandleTOTPVerify)
		r.Post("/api/v1/auth/totp/disable", authHandler.HandleTOTPDisable)

		// Dashboard statistics and trends.
		r.Get("/api/v1/teams/{teamID}/dashboard/stats", s.handleGetDashboardStats)
		r.Get("/api/v1/teams/{teamID}/dashboard/trends", s.handleGetConnectionTrends)
		r.Get("/api/v1/teams/{teamID}/dashboard/activities", s.handleGetRecentActivities)

		// Device management.
		r.Route("/api/v1/teams/{teamID}/devices", func(r chi.Router) {
			r.Get("/", s.handleListDevices)
			r.Post("/", s.handleCreateDevice)
			r.Get("/{deviceCode}", s.handleGetDevice)
			r.Delete("/{deviceCode}", s.handleDeleteDevice)
		})

		// Member management.
		r.Get("/api/v1/teams/{teamID}/members", s.handleListMembers)
		r.Post("/api/v1/teams/{teamID}/invite", s.handleInviteMember)
		r.Put("/api/v1/teams/{teamID}/members/{memberID}", s.handleUpdateMember)
		r.Delete("/api/v1/teams/{teamID}/members/{memberID}", s.handleRemoveMember)

		// Connection records and audit logs.
		r.Get("/api/v1/teams/{teamID}/sessions", s.handleListSessions)
		r.Get("/api/v1/teams/{teamID}/sessions/export", s.handleExportSessionsCSV)
		r.Get("/api/v1/teams/{teamID}/records", s.handleListConnectionRecords)
		r.Get("/api/v1/teams/{teamID}/records/export", s.handleExportConnectionRecords)
		r.Get("/api/v1/teams/{teamID}/audit", s.handleListAuditLogs)
		r.Get("/api/v1/teams/{teamID}/audit-logs", s.handleListAuditLogsAPI)
	})

	// WebSocket endpoint for real-time event push.
	// JWT is validated from the ?token= query parameter, not the Authorization header.
	s.router.Get("/api/v1/ws", ws.HandleWebSocket(s.Hub, s.cfg.JWTPublicKey))
}

// Start launches the HTTP server and blocks until ctx is cancelled.
// It performs a graceful shutdown with a 10-second deadline.
func (s *Server) Start(ctx context.Context) error {
	// Start the WebSocket hub for real-time event push.
	go s.Hub.Run(ctx)

	addr := fmt.Sprintf(":%d", s.cfg.Port)
	srv := &http.Server{
		Addr:              addr,
		Handler:           s.router,
		ReadHeaderTimeout: 10 * time.Second,
	}

	// Channel to capture server errors.
	errCh := make(chan error, 1)
	go func() {
		slog.Info("RDCS API server starting", "addr", addr)
		if err := srv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
		}
		close(errCh)
	}()

	// Wait for cancellation or server error.
	select {
	case <-ctx.Done():
		slog.Info("shutdown signal received, draining connections…")
	case err := <-errCh:
		return fmt.Errorf("server listen: %w", err)
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := srv.Shutdown(shutdownCtx); err != nil {
		return fmt.Errorf("graceful shutdown: %w", err)
	}

	slog.Info("server stopped gracefully")
	return nil
}
