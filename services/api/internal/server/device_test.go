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
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/model"
	"github.com/rdcs/rdcs-api/internal/repository"
)

// ---------------------------------------------------------------------------
// Mock DeviceRepository
// ---------------------------------------------------------------------------

type mockDeviceRepo struct {
	devices []*model.Device
}

func (m *mockDeviceRepo) GetByID(_ context.Context, id uuid.UUID) (*model.Device, error) {
	for _, d := range m.devices {
		if d.ID == id {
			return d, nil
		}
	}
	return nil, fmt.Errorf("device not found")
}

func (m *mockDeviceRepo) GetByCode(_ context.Context, code string) (*model.Device, error) {
	for _, d := range m.devices {
		if d.DeviceCode == code {
			return d, nil
		}
	}
	return nil, fmt.Errorf("device not found")
}

func (m *mockDeviceRepo) ListByTeam(_ context.Context, teamID uuid.UUID, filter repository.DeviceFilter) ([]*model.Device, error) {
	var result []*model.Device
	for _, d := range m.devices {
		if d.TeamID != teamID {
			continue
		}
		if filter.Status != "" && d.Status != filter.Status {
			continue
		}
		if filter.Platform != "" && d.Platform != filter.Platform {
			continue
		}
		if filter.Search != "" && !strings.Contains(strings.ToLower(d.DeviceName), strings.ToLower(filter.Search)) {
			continue
		}
		result = append(result, d)
	}

	// Apply pagination.
	limit := filter.Limit
	if limit <= 0 {
		limit = 50
	}
	offset := filter.Offset
	if offset < 0 {
		offset = 0
	}
	if offset > len(result) {
		return nil, nil
	}
	end := offset + limit
	if end > len(result) {
		end = len(result)
	}
	return result[offset:end], nil
}

func (m *mockDeviceRepo) CountByTeam(_ context.Context, teamID uuid.UUID, filter repository.DeviceFilter) (int, error) {
	count := 0
	for _, d := range m.devices {
		if d.TeamID != teamID {
			continue
		}
		if filter.Status != "" && d.Status != filter.Status {
			continue
		}
		if filter.Platform != "" && d.Platform != filter.Platform {
			continue
		}
		if filter.Search != "" && !strings.Contains(strings.ToLower(d.DeviceName), strings.ToLower(filter.Search)) {
			continue
		}
		count++
	}
	return count, nil
}

func (m *mockDeviceRepo) Create(_ context.Context, device *model.Device) error {
	// Check for duplicate device_code.
	for _, d := range m.devices {
		if d.DeviceCode == device.DeviceCode {
			return fmt.Errorf("duplicate key value violates unique constraint")
		}
	}
	device.ID = uuid.New()
	device.CreatedAt = time.Now()
	m.devices = append(m.devices, device)
	return nil
}

func (m *mockDeviceRepo) Update(_ context.Context, device *model.Device) error {
	for i, d := range m.devices {
		if d.ID == device.ID {
			m.devices[i] = device
			return nil
		}
	}
	return fmt.Errorf("device not found")
}

func (m *mockDeviceRepo) Delete(_ context.Context, id uuid.UUID) error {
	for i, d := range m.devices {
		if d.ID == id {
			m.devices = append(m.devices[:i], m.devices[i+1:]...)
			return nil
		}
	}
	return fmt.Errorf("device not found")
}

// ---------------------------------------------------------------------------
// Mock AuditLogRepository
// ---------------------------------------------------------------------------

type mockAuditLogRepo struct {
	logs []*model.AuditLog
}

func (m *mockAuditLogRepo) List(_ context.Context, _ uuid.UUID, _ repository.AuditFilter) ([]*model.AuditLog, error) {
	return m.logs, nil
}

func (m *mockAuditLogRepo) Count(_ context.Context, _ uuid.UUID, _ repository.AuditFilter) (int, error) {
	return len(m.logs), nil
}

func (m *mockAuditLogRepo) Create(_ context.Context, log *model.AuditLog) error {
	log.ID = uuid.New()
	log.CreatedAt = time.Now()
	m.logs = append(m.logs, log)
	return nil
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

var (
	testTeamID   = uuid.MustParse("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
	testMemberID = uuid.MustParse("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")
)

func newDeviceTestServer(devRepo *mockDeviceRepo, auditRepo *mockAuditLogRepo) *Server {
	return &Server{
		Devices:   devRepo,
		AuditLogs: auditRepo,
	}
}

func newTestDevice(code, name, platform, status string) *model.Device {
	return &model.Device{
		ID:         uuid.New(),
		TeamID:     testTeamID,
		DeviceCode: code,
		DeviceName: name,
		Platform:   platform,
		Status:     status,
		CreatedAt:  time.Now(),
	}
}

// withURLParams injects chi URL parameters into the request context.
func withURLParams(r *http.Request, params map[string]string) *http.Request {
	rctx := chi.NewRouteContext()
	for k, v := range params {
		rctx.URLParams.Add(k, v)
	}
	return r.WithContext(context.WithValue(r.Context(), chi.RouteCtxKey, rctx))
}

// withClaims injects JWT claims into the request context.
func withClaims(r *http.Request, claims *auth.Claims) *http.Request {
	return r.WithContext(auth.ContextWithClaims(r.Context(), claims))
}

func decodeJSON(t *testing.T, body io.Reader, v interface{}) {
	t.Helper()
	if err := json.NewDecoder(body).Decode(v); err != nil {
		t.Fatalf("failed to decode JSON response: %v", err)
	}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

func TestHandleListDevices_ReturnsPaginated(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/"+testTeamID.String()+"/devices?limit=2&offset=0", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 3 {
		t.Errorf("total = %d, want 3", resp.Total)
	}
	if len(resp.Devices) != 2 {
		t.Errorf("devices count = %d, want 2 (limit=2)", len(resp.Devices))
	}
	if resp.Page != 1 {
		t.Errorf("page = %d, want 1", resp.Page)
	}
	if resp.PageSize != 2 {
		t.Errorf("page_size = %d, want 2", resp.PageSize)
	}
}

func TestHandleListDevices_SecondPage(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/"+testTeamID.String()+"/devices?limit=2&offset=2", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 3 {
		t.Errorf("total = %d, want 3", resp.Total)
	}
	if len(resp.Devices) != 1 {
		t.Errorf("devices count = %d, want 1 (remaining on page 2)", len(resp.Devices))
	}
	if resp.Page != 2 {
		t.Errorf("page = %d, want 2", resp.Page)
	}
}

func TestHandleListDevices_FilterByStatus(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/"+testTeamID.String()+"/devices?status=online", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 2 {
		t.Errorf("total = %d, want 2 (online only)", resp.Total)
	}
	if len(resp.Devices) != 2 {
		t.Errorf("devices count = %d, want 2", len(resp.Devices))
	}
	for _, d := range resp.Devices {
		if d.Status != "online" {
			t.Errorf("device %s status = %q, want online", d.DeviceCode, d.Status)
		}
	}
}

func TestHandleListDevices_FilterByPlatform(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/"+testTeamID.String()+"/devices?platform=windows", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 1 {
		t.Errorf("total = %d, want 1 (windows only)", resp.Total)
	}
	if len(resp.Devices) != 1 {
		t.Fatalf("devices count = %d, want 1", len(resp.Devices))
	}
	if resp.Devices[0].Platform != "windows" {
		t.Errorf("platform = %q, want windows", resp.Devices[0].Platform)
	}
}

func TestHandleListDevices_FilterBySearch(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/"+testTeamID.String()+"/devices?search=office", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 2 {
		t.Errorf("total = %d, want 2 (matching 'office')", resp.Total)
	}
}

func TestHandleListDevices_CombinedFilters(t *testing.T) {
	devRepo := &mockDeviceRepo{
		devices: []*model.Device{
			newTestDevice("DEV001", "Office-Mac", "macos", "online"),
			newTestDevice("DEV002", "Office-Win", "windows", "offline"),
			newTestDevice("DEV003", "Dev-Linux", "linux", "online"),
		},
	}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/devices?status=online&platform=linux", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 1 {
		t.Errorf("total = %d, want 1 (online + linux)", resp.Total)
	}
}

func TestHandleGetDevice_ReturnsDetail(t *testing.T) {
	device := newTestDevice("DEV001", "Office-Mac", "macos", "online")
	devRepo := &mockDeviceRepo{devices: []*model.Device{device}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/devices/DEV001", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     testTeamID.String(),
		"deviceCode": "DEV001",
	})
	rec := httptest.NewRecorder()

	srv.handleGetDevice(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.DeviceCode != "DEV001" {
		t.Errorf("device_code = %q, want DEV001", resp.DeviceCode)
	}
	if resp.DeviceName != "Office-Mac" {
		t.Errorf("device_name = %q, want Office-Mac", resp.DeviceName)
	}
	if resp.Platform != "macos" {
		t.Errorf("platform = %q, want macos", resp.Platform)
	}
	if resp.Status != "online" {
		t.Errorf("status = %q, want online", resp.Status)
	}
}

func TestHandleGetDevice_NotFound(t *testing.T) {
	devRepo := &mockDeviceRepo{devices: []*model.Device{}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/devices/NONEXISTENT", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     testTeamID.String(),
		"deviceCode": "NONEXISTENT",
	})
	rec := httptest.NewRecorder()

	srv.handleGetDevice(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	decodeJSON(t, rec.Body, &resp)

	if resp["error"] != "device_not_found" {
		t.Errorf("error = %q, want device_not_found", resp["error"])
	}
}

func TestHandleGetDevice_WrongTeam(t *testing.T) {
	device := newTestDevice("DEV001", "Office-Mac", "macos", "online")
	devRepo := &mockDeviceRepo{devices: []*model.Device{device}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	otherTeam := uuid.New().String()
	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+otherTeam+"/devices/DEV001", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     otherTeam,
		"deviceCode": "DEV001",
	})
	rec := httptest.NewRecorder()

	srv.handleGetDevice(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404 for wrong team, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleCreateDevice_Succeeds(t *testing.T) {
	devRepo := &mockDeviceRepo{devices: []*model.Device{}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(createDeviceRequest{
		DeviceCode: "NEW001",
		DeviceName: "New-Device",
		Platform:   "macos",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/devices", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleCreateDevice(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp deviceJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.DeviceCode != "NEW001" {
		t.Errorf("device_code = %q, want NEW001", resp.DeviceCode)
	}
	if resp.DeviceName != "New-Device" {
		t.Errorf("device_name = %q, want New-Device", resp.DeviceName)
	}
	if resp.Platform != "macos" {
		t.Errorf("platform = %q, want macos", resp.Platform)
	}
	if resp.Status != "offline" {
		t.Errorf("status = %q, want offline (default)", resp.Status)
	}
	if resp.ID == "" {
		t.Error("id should be assigned after creation")
	}

	// Verify the device was actually stored.
	if len(devRepo.devices) != 1 {
		t.Errorf("repo device count = %d, want 1", len(devRepo.devices))
	}
}

func TestHandleCreateDevice_InvalidPlatform(t *testing.T) {
	devRepo := &mockDeviceRepo{devices: []*model.Device{}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(createDeviceRequest{
		DeviceCode: "NEW001",
		DeviceName: "New-Device",
		Platform:   "android",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/devices", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleCreateDevice(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleCreateDevice_MissingFields(t *testing.T) {
	devRepo := &mockDeviceRepo{devices: []*model.Device{}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	tests := []struct {
		name string
		req  createDeviceRequest
	}{
		{"missing code", createDeviceRequest{DeviceName: "N", Platform: "macos"}},
		{"missing name", createDeviceRequest{DeviceCode: "C", Platform: "macos"}},
		{"missing platform", createDeviceRequest{DeviceCode: "C", DeviceName: "N"}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			body, _ := json.Marshal(tt.req)
			req := httptest.NewRequest(http.MethodPost,
				"/api/v1/teams/"+testTeamID.String()+"/devices", bytes.NewReader(body))
			req.Header.Set("Content-Type", "application/json")
			req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
			rec := httptest.NewRecorder()

			srv.handleCreateDevice(rec, req)

			if rec.Code != http.StatusBadRequest {
				t.Errorf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestHandleCreateDevice_DuplicateCode(t *testing.T) {
	existing := newTestDevice("DUP001", "Existing", "macos", "online")
	devRepo := &mockDeviceRepo{devices: []*model.Device{existing}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(createDeviceRequest{
		DeviceCode: "DUP001",
		DeviceName: "Another",
		Platform:   "linux",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/devices", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleCreateDevice(rec, req)

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected status 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleDeleteDevice_Succeeds(t *testing.T) {
	device := newTestDevice("DEL001", "To-Delete", "windows", "offline")
	devRepo := &mockDeviceRepo{devices: []*model.Device{device}}
	auditRepo := &mockAuditLogRepo{}
	srv := newDeviceTestServer(devRepo, auditRepo)

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/devices/DEL001", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     testTeamID.String(),
		"deviceCode": "DEL001",
	})
	rec := httptest.NewRecorder()

	srv.handleDeleteDevice(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected status 204, got %d: %s", rec.Code, rec.Body.String())
	}

	// Verify device was removed from repository.
	if len(devRepo.devices) != 0 {
		t.Errorf("repo device count = %d, want 0 after deletion", len(devRepo.devices))
	}
}

func TestHandleDeleteDevice_NotFound(t *testing.T) {
	devRepo := &mockDeviceRepo{devices: []*model.Device{}}
	srv := newDeviceTestServer(devRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/devices/GHOST", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     testTeamID.String(),
		"deviceCode": "GHOST",
	})
	rec := httptest.NewRecorder()

	srv.handleDeleteDevice(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleDeleteDevice_CreatesAuditLog(t *testing.T) {
	device := newTestDevice("DEL002", "Audit-Test", "linux", "online")
	devRepo := &mockDeviceRepo{devices: []*model.Device{device}}
	auditRepo := &mockAuditLogRepo{}
	srv := newDeviceTestServer(devRepo, auditRepo)

	claims := &auth.Claims{
		MemberID: testMemberID.String(),
		TeamID:   testTeamID.String(),
		Role:     "admin",
	}

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/devices/DEL002", nil)
	req = withURLParams(req, map[string]string{
		"teamID":     testTeamID.String(),
		"deviceCode": "DEL002",
	})
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleDeleteDevice(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected status 204, got %d: %s", rec.Code, rec.Body.String())
	}

	// Verify audit log was created.
	if len(auditRepo.logs) != 1 {
		t.Fatalf("audit log count = %d, want 1", len(auditRepo.logs))
	}

	log := auditRepo.logs[0]
	if log.TeamID != testTeamID {
		t.Errorf("audit log team_id = %v, want %v", log.TeamID, testTeamID)
	}
	if log.Action != "device.delete" {
		t.Errorf("audit log action = %q, want device.delete", log.Action)
	}
	if log.ActorID == nil || *log.ActorID != testMemberID {
		t.Errorf("audit log actor_id = %v, want %v", log.ActorID, testMemberID)
	}
	if log.TargetType == nil || *log.TargetType != "device" {
		t.Errorf("audit log target_type = %v, want device", log.TargetType)
	}
	if log.TargetID == nil || *log.TargetID != device.ID {
		t.Errorf("audit log target_id = %v, want %v", log.TargetID, device.ID)
	}

	// Verify details contain device info.
	var details map[string]string
	if err := json.Unmarshal(log.Details, &details); err != nil {
		t.Fatalf("failed to unmarshal audit log details: %v", err)
	}
	if details["device_code"] != "DEL002" {
		t.Errorf("details device_code = %q, want DEL002", details["device_code"])
	}
	if details["device_name"] != "Audit-Test" {
		t.Errorf("details device_name = %q, want Audit-Test", details["device_name"])
	}
}

func TestHandleListDevices_InvalidTeamID(t *testing.T) {
	srv := newDeviceTestServer(&mockDeviceRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/not-a-uuid/devices", nil)
	req = withURLParams(req, map[string]string{"teamID": "not-a-uuid"})
	rec := httptest.NewRecorder()

	srv.handleListDevices(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleCreateDevice_InvalidJSON(t *testing.T) {
	srv := newDeviceTestServer(&mockDeviceRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/devices",
		bytes.NewReader([]byte("{invalid json")))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleCreateDevice(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}
