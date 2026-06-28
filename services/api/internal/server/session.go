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
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/repository"
)

// handleListSessions returns a paginated list of connection records for a team.
//
// GET /api/v1/teams/{teamID}/sessions?start_date=&end_date=&path=&limit=20&offset=0
func (s *Server) handleListSessions(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	filter, err := parseConnFilter(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}

	records, err := s.Connections.List(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to list sessions", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	total, err := s.Connections.Count(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to count sessions", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	writeJSONResponse(w, http.StatusOK, map[string]interface{}{
		"sessions": records,
		"total":    total,
	})
}

// handleExportSessionsCSV exports connection records as a CSV file download.
//
// GET /api/v1/teams/{teamID}/sessions/export?start_date=&end_date=&path=
func (s *Server) handleExportSessionsCSV(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	filter, err := parseConnFilter(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}

	// CSV export returns all matching records (no pagination).
	filter.Limit = 0
	filter.Offset = 0

	filename := fmt.Sprintf("sessions_%s.csv", time.Now().UTC().Format("20060102_150405"))

	w.Header().Set("Content-Type", "text/csv; charset=utf-8")
	w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%q", filename))
	w.WriteHeader(http.StatusOK)

	// UTF-8 BOM for Excel compatibility.
	bom := []byte{0xEF, 0xBB, 0xBF}
	if _, err := w.Write(bom); err != nil {
		slog.Error("failed to write UTF-8 BOM", "error", err)
		return
	}

	if err := s.Connections.ExportCSV(r.Context(), teamID, filter, w); err != nil {
		slog.Error("failed to export sessions CSV", "error", err)
		// Headers already sent; cannot change status code.
		return
	}
}

// handleListAuditLogs returns a paginated list of audit logs for a team.
//
// GET /api/v1/teams/{teamID}/audit?action=&actor_id=&limit=50&offset=0
func (s *Server) handleListAuditLogs(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	filter, err := parseAuditFilter(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": err.Error()})
		return
	}

	logs, err := s.AuditLogs.List(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to list audit logs", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	total, err := s.AuditLogs.Count(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to count audit logs", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	writeJSONResponse(w, http.StatusOK, map[string]interface{}{
		"logs":  logs,
		"total": total,
	})
}

// parseTeamID extracts and validates the teamID URL parameter.
func parseTeamID(r *http.Request) (uuid.UUID, error) {
	raw := chi.URLParam(r, "teamID")
	return uuid.Parse(raw)
}

// parseConnFilter builds a ConnFilter from the request query parameters.
func parseConnFilter(r *http.Request) (repository.ConnFilter, error) {
	q := r.URL.Query()
	var filter repository.ConnFilter

	if v := q.Get("start_date"); v != "" {
		t, err := time.Parse(time.RFC3339, v)
		if err != nil {
			return filter, fmt.Errorf("invalid start_date: %w", err)
		}
		filter.StartedAfter = &t
	}
	if v := q.Get("end_date"); v != "" {
		t, err := time.Parse(time.RFC3339, v)
		if err != nil {
			return filter, fmt.Errorf("invalid end_date: %w", err)
		}
		filter.StartedBefore = &t
	}
	if v := q.Get("path"); v != "" {
		filter.Path = v
	}

	filter.Limit = queryInt(r, "limit", 20)
	filter.Offset = queryInt(r, "offset", 0)

	return filter, nil
}

// parseAuditFilter builds an AuditFilter from the request query parameters.
func parseAuditFilter(r *http.Request) (repository.AuditFilter, error) {
	q := r.URL.Query()
	var filter repository.AuditFilter

	if v := q.Get("action"); v != "" {
		filter.Action = v
	}
	if v := q.Get("actor_id"); v != "" {
		id, err := uuid.Parse(v)
		if err != nil {
			return filter, fmt.Errorf("invalid actor_id: %w", err)
		}
		filter.ActorID = &id
	}

	filter.Limit = queryInt(r, "limit", 50)
	filter.Offset = queryInt(r, "offset", 0)

	return filter, nil
}

// queryInt reads an integer query parameter with a default value.
func queryInt(r *http.Request, key string, defaultVal int) int {
	v := r.URL.Query().Get(key)
	if v == "" {
		return defaultVal
	}
	n, err := strconv.Atoi(v)
	if err != nil || n < 0 {
		return defaultVal
	}
	return n
}

// writeJSONResponse encodes v as JSON and writes it to w with the given status code.
func writeJSONResponse(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(v); err != nil {
		slog.Error("failed to write json response", "error", err)
	}
}
