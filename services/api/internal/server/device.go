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
	"log/slog"
	"net/http"
	"strings"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/model"
	"github.com/rdcs/rdcs-api/internal/repository"
)

// deviceListResponse is the JSON envelope for paginated device listings.
type deviceListResponse struct {
	Devices  []deviceJSON `json:"devices"`
	Total    int          `json:"total"`
	Page     int          `json:"page"`
	PageSize int          `json:"page_size"`
}

// deviceJSON is the API representation of a single device.
type deviceJSON struct {
	ID            string  `json:"id"`
	TeamID        string  `json:"team_id"`
	DeviceCode    string  `json:"device_code"`
	DeviceName    string  `json:"device_name"`
	Platform      string  `json:"platform"`
	OsVersion     *string `json:"os_version,omitempty"`
	ClientVersion *string `json:"client_version,omitempty"`
	Status        string  `json:"status"`
	LastSeen      *string `json:"last_seen,omitempty"`
	IPAddress     *string `json:"ip_address,omitempty"`
	CreatedAt     string  `json:"created_at"`
}

// createDeviceRequest is the JSON body expected on POST /api/v1/teams/{teamID}/devices.
type createDeviceRequest struct {
	DeviceCode string `json:"device_code"`
	DeviceName string `json:"device_name"`
	Platform   string `json:"platform"`
}

// toDeviceJSON converts a model.Device into its API representation.
func toDeviceJSON(d *model.Device) deviceJSON {
	dj := deviceJSON{
		ID:            d.ID.String(),
		TeamID:        d.TeamID.String(),
		DeviceCode:    d.DeviceCode,
		DeviceName:    d.DeviceName,
		Platform:      d.Platform,
		OsVersion:     d.OsVersion,
		ClientVersion: d.ClientVersion,
		Status:        d.Status,
		IPAddress:     d.IPAddress,
		CreatedAt:     d.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
	if d.LastSeen != nil {
		s := d.LastSeen.UTC().Format("2006-01-02T15:04:05Z")
		dj.LastSeen = &s
	}
	return dj
}

// handleListDevices returns a paginated, filtered list of devices for a team.
//
// GET /api/v1/teams/{teamID}/devices?status=&platform=&search=&limit=20&offset=0
func (s *Server) handleListDevices(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	limit := queryInt(r, "limit", 20)
	offset := queryInt(r, "offset", 0)

	filter := repository.DeviceFilter{
		Status:   strings.TrimSpace(r.URL.Query().Get("status")),
		Platform: strings.TrimSpace(r.URL.Query().Get("platform")),
		Search:   strings.TrimSpace(r.URL.Query().Get("search")),
		Limit:    limit,
		Offset:   offset,
	}

	devices, err := s.Devices.ListByTeam(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to list devices", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	total, err := s.Devices.CountByTeam(r.Context(), teamID, filter)
	if err != nil {
		slog.Error("failed to count devices", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	page := 1
	if limit > 0 {
		page = (offset / limit) + 1
	}

	items := make([]deviceJSON, 0, len(devices))
	for _, d := range devices {
		items = append(items, toDeviceJSON(d))
	}

	writeJSONResponse(w, http.StatusOK, deviceListResponse{
		Devices:  items,
		Total:    total,
		Page:     page,
		PageSize: limit,
	})
}

// handleGetDevice returns the detail of a single device identified by its device code.
//
// GET /api/v1/teams/{teamID}/devices/{deviceCode}
func (s *Server) handleGetDevice(w http.ResponseWriter, r *http.Request) {
	deviceCode := chi.URLParam(r, "deviceCode")
	if deviceCode == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "missing_device_code"})
		return
	}

	device, err := s.Devices.GetByCode(r.Context(), deviceCode)
	if err != nil {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "device_not_found"})
		return
	}

	// Verify the device belongs to the requested team.
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}
	if device.TeamID != teamID {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "device_not_found"})
		return
	}

	writeJSONResponse(w, http.StatusOK, toDeviceJSON(device))
}

// handleCreateDevice registers a new device for the given team.
//
// POST /api/v1/teams/{teamID}/devices
// Request body: {"device_code": "123456789", "device_name": "...", "platform": "macos"}
func (s *Server) handleCreateDevice(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	var req createDeviceRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	if req.DeviceCode == "" || req.DeviceName == "" || req.Platform == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "device_code, device_name, and platform are required"})
		return
	}

	validPlatforms := map[string]bool{"windows": true, "macos": true, "linux": true}
	if !validPlatforms[req.Platform] {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "platform must be windows, macos, or linux"})
		return
	}

	device := &model.Device{
		TeamID:     teamID,
		DeviceCode: req.DeviceCode,
		DeviceName: req.DeviceName,
		Platform:   req.Platform,
		Status:     "offline",
	}

	if err := s.Devices.Create(r.Context(), device); err != nil {
		// Handle unique constraint violation on device_code.
		if strings.Contains(err.Error(), "duplicate key") || strings.Contains(err.Error(), "unique constraint") {
			writeJSONResponse(w, http.StatusConflict, map[string]string{"error": "device_code_already_exists"})
			return
		}
		slog.Error("failed to create device", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	writeJSONResponse(w, http.StatusCreated, toDeviceJSON(device))
}

// handleDeleteDevice removes a device identified by its device code and creates
// an audit log entry for the deletion.
//
// DELETE /api/v1/teams/{teamID}/devices/{deviceCode}
func (s *Server) handleDeleteDevice(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	deviceCode := chi.URLParam(r, "deviceCode")
	if deviceCode == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "missing_device_code"})
		return
	}

	device, err := s.Devices.GetByCode(r.Context(), deviceCode)
	if err != nil {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "device_not_found"})
		return
	}

	if device.TeamID != teamID {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "device_not_found"})
		return
	}

	if err := s.Devices.Delete(r.Context(), device.ID); err != nil {
		slog.Error("failed to delete device", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	// Create audit log entry for the deletion.
	claims := auth.ClaimsFromContext(r.Context())
	var actorID *uuid.UUID
	if claims != nil {
		if parsed, parseErr := uuid.Parse(claims.MemberID); parseErr == nil {
			actorID = &parsed
		}
	}

	targetType := "device"
	details, _ := json.Marshal(map[string]string{
		"device_code": device.DeviceCode,
		"device_name": device.DeviceName,
		"platform":    device.Platform,
	})

	auditLog := &model.AuditLog{
		TeamID:     teamID,
		ActorID:    actorID,
		Action:     "device.delete",
		TargetType: &targetType,
		TargetID:   &device.ID,
		Details:    details,
	}
	if err := s.AuditLogs.Create(r.Context(), auditLog); err != nil {
		slog.Error("failed to create audit log for device deletion", "error", err)
		// Do not fail the response — the device was already deleted.
	}

	w.WriteHeader(http.StatusNoContent)
}
