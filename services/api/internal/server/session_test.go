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
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/model"
	"github.com/rdcs/rdcs-api/internal/repository"
)

// ---------------------------------------------------------------------------
// Mock ConnectionRecordRepository
// ---------------------------------------------------------------------------

type mockConnRepo struct {
	records []*model.ConnectionRecord
	total   int
	csvData string
	csvErr  error
}

func (m *mockConnRepo) List(_ context.Context, _ uuid.UUID, _ repository.ConnFilter) ([]*model.ConnectionRecord, error) {
	return m.records, nil
}

func (m *mockConnRepo) Count(_ context.Context, _ uuid.UUID, _ repository.ConnFilter) (int, error) {
	return m.total, nil
}

func (m *mockConnRepo) Create(_ context.Context, _ *model.ConnectionRecord) error {
	return nil
}

func (m *mockConnRepo) ExportCSV(_ context.Context, _ uuid.UUID, _ repository.ConnFilter, w io.Writer) error {
	if m.csvErr != nil {
		return m.csvErr
	}
	data := m.csvData
	if data == "" {
		data = "id,team_id,controller_code\n"
	}
	_, err := w.Write([]byte(data))
	return err
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// newSessionTestServer creates a Server wired with connection and audit mock repos.
func newSessionTestServer(conns *mockConnRepo, audits *mockAuditLogRepo) *Server {
	return &Server{
		Connections: conns,
		AuditLogs:   audits,
	}
}

// ---------------------------------------------------------------------------
// Tests — Connection Records (Sessions)
// ---------------------------------------------------------------------------

func TestHandleListSessions_Paginated(t *testing.T) {
	recID := uuid.MustParse("11111111-1111-1111-1111-111111111111")
	now := time.Now().UTC()

	records := []*model.ConnectionRecord{
		{
			ID:               recID,
			TeamID:           testTeamID,
			ControllerCode:   "CTRL001",
			ControlledCode:   "CTLD001",
			Path:             "L1",
			StartedAt:        now,
			BytesTransferred: 1024,
			CreatedAt:        now,
		},
	}

	conns := &mockConnRepo{records: records, total: 42}
	srv := newSessionTestServer(conns, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/sessions?limit=20&offset=0", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListSessions(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]interface{}
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	// Verify "total" field is present and correct.
	totalFloat, ok := resp["total"].(float64)
	if !ok {
		t.Fatal("response missing 'total' field or wrong type")
	}
	if int(totalFloat) != 42 {
		t.Errorf("total = %v, want 42", totalFloat)
	}

	// Verify "sessions" array exists and has correct length.
	sessions, ok := resp["sessions"].([]interface{})
	if !ok {
		t.Fatal("response missing 'sessions' array")
	}
	if len(sessions) != 1 {
		t.Errorf("sessions length = %d, want 1", len(sessions))
	}
}

func TestHandleListSessions_Filters(t *testing.T) {
	conns := &mockConnRepo{records: []*model.ConnectionRecord{}, total: 0}
	srv := newSessionTestServer(conns, &mockAuditLogRepo{})

	startDate := "2026-01-01T00:00:00Z"
	endDate := "2026-12-31T23:59:59Z"
	url := "/api/v1/teams/" + testTeamID.String() + "/sessions?start_date=" + startDate + "&end_date=" + endDate + "&path=L2&limit=10&offset=5"

	req := httptest.NewRequest(http.MethodGet, url, nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListSessions(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]interface{}
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if _, ok := resp["sessions"]; !ok {
		t.Error("response missing 'sessions' field")
	}
	if _, ok := resp["total"]; !ok {
		t.Error("response missing 'total' field")
	}
}

func TestHandleListSessions_InvalidStartDate(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/sessions?start_date=not-a-date", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListSessions(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid start_date, got %d", rec.Code)
	}
}

func TestHandleListSessions_InvalidEndDate(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/sessions?end_date=bad", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListSessions(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid end_date, got %d", rec.Code)
	}
}

func TestHandleListSessions_InvalidTeamID(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/not-a-uuid/sessions", nil)
	req = withURLParams(req, map[string]string{"teamID": "not-a-uuid"})
	rec := httptest.NewRecorder()

	srv.handleListSessions(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid teamID, got %d", rec.Code)
	}
}

// ---------------------------------------------------------------------------
// Tests — CSV Export
// ---------------------------------------------------------------------------

func TestHandleExportSessionsCSV_Download(t *testing.T) {
	conns := &mockConnRepo{records: []*model.ConnectionRecord{}, total: 0}
	srv := newSessionTestServer(conns, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/sessions/export", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleExportSessionsCSV(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	contentType := rec.Header().Get("Content-Type")
	if !strings.Contains(contentType, "text/csv") {
		t.Errorf("Content-Type = %q, want text/csv", contentType)
	}

	disposition := rec.Header().Get("Content-Disposition")
	if !strings.Contains(disposition, "attachment") {
		t.Errorf("Content-Disposition = %q, want attachment", disposition)
	}
	if !strings.Contains(disposition, "sessions_") {
		t.Errorf("Content-Disposition filename should contain 'sessions_', got %q", disposition)
	}
}

func TestHandleExportSessionsCSV_UTF8BOM(t *testing.T) {
	conns := &mockConnRepo{records: []*model.ConnectionRecord{}, total: 0}
	srv := newSessionTestServer(conns, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/sessions/export", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleExportSessionsCSV(rec, req)

	body := rec.Body.Bytes()
	if len(body) < 3 {
		t.Fatalf("response body too short to contain BOM, len=%d", len(body))
	}

	// UTF-8 BOM: 0xEF 0xBB 0xBF
	bom := []byte{0xEF, 0xBB, 0xBF}
	if !bytes.Equal(body[:3], bom) {
		t.Errorf("first 3 bytes = %v, want UTF-8 BOM %v", body[:3], bom)
	}
}

func TestHandleExportSessionsCSV_WithFilters(t *testing.T) {
	conns := &mockConnRepo{records: []*model.ConnectionRecord{}, total: 0}
	srv := newSessionTestServer(conns, &mockAuditLogRepo{})

	url := "/api/v1/teams/" + testTeamID.String() + "/sessions/export?start_date=2026-01-01T00:00:00Z&end_date=2026-12-31T23:59:59Z&path=L1"
	req := httptest.NewRequest(http.MethodGet, url, nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleExportSessionsCSV(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleExportSessionsCSV_InvalidTeamID(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/not-a-uuid/sessions/export", nil)
	req = withURLParams(req, map[string]string{"teamID": "not-a-uuid"})
	rec := httptest.NewRecorder()

	srv.handleExportSessionsCSV(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid teamID, got %d", rec.Code)
	}
}

// ---------------------------------------------------------------------------
// Tests — Audit Logs
// ---------------------------------------------------------------------------

func TestHandleListAuditLogs_WithFilters(t *testing.T) {
	logID := uuid.MustParse("22222222-2222-2222-2222-222222222222")
	actorID := uuid.MustParse("33333333-3333-3333-3333-333333333333")
	now := time.Now().UTC()

	logs := []*model.AuditLog{
		{
			ID:        logID,
			TeamID:    testTeamID,
			ActorID:   &actorID,
			Action:    "device.delete",
			CreatedAt: now,
		},
	}

	audits := &mockAuditLogRepo{logs: logs}
	srv := newSessionTestServer(&mockConnRepo{}, audits)

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/audit?action=device.delete&limit=50&offset=0", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListAuditLogs(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]interface{}
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	// Verify total is present.
	totalFloat, ok := resp["total"].(float64)
	if !ok {
		t.Fatal("response missing 'total' field or wrong type")
	}
	if int(totalFloat) != 1 {
		t.Errorf("total = %v, want 1", totalFloat)
	}

	// Verify logs array.
	logItems, ok := resp["logs"].([]interface{})
	if !ok {
		t.Fatal("response missing 'logs' array")
	}
	if len(logItems) != 1 {
		t.Errorf("logs length = %d, want 1", len(logItems))
	}
}

func TestHandleListAuditLogs_FilterByActorID(t *testing.T) {
	audits := &mockAuditLogRepo{logs: []*model.AuditLog{}}
	srv := newSessionTestServer(&mockConnRepo{}, audits)

	actorID := uuid.MustParse("33333333-3333-3333-3333-333333333333")
	url := "/api/v1/teams/" + testTeamID.String() + "/audit?actor_id=" + actorID.String()

	req := httptest.NewRequest(http.MethodGet, url, nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListAuditLogs(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleListAuditLogs_InvalidActorID(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/audit?actor_id=not-a-uuid", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListAuditLogs(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid actor_id, got %d", rec.Code)
	}
}

func TestHandleListAuditLogs_DefaultPagination(t *testing.T) {
	audits := &mockAuditLogRepo{logs: []*model.AuditLog{}}
	srv := newSessionTestServer(&mockConnRepo{}, audits)

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/audit", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListAuditLogs(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleListAuditLogs_InvalidTeamID(t *testing.T) {
	srv := newSessionTestServer(&mockConnRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/teams/not-a-uuid/audit", nil)
	req = withURLParams(req, map[string]string{"teamID": "not-a-uuid"})
	rec := httptest.NewRecorder()

	srv.handleListAuditLogs(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400 for invalid teamID, got %d", rec.Code)
	}
}
