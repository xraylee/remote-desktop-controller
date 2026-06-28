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
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/model"
)

// ---------------------------------------------------------------------------
// Mock MemberRepository
// ---------------------------------------------------------------------------

type mockMemberRepo struct {
	members []*model.Member
}

func (m *mockMemberRepo) GetByID(_ context.Context, id uuid.UUID) (*model.Member, error) {
	for _, member := range m.members {
		if member.ID == id {
			return member, nil
		}
	}
	return nil, fmt.Errorf("member not found")
}

func (m *mockMemberRepo) GetByEmail(_ context.Context, email string) (*model.Member, error) {
	for _, member := range m.members {
		if member.Email == email {
			return member, nil
		}
	}
	return nil, fmt.Errorf("member not found")
}

func (m *mockMemberRepo) ListByTeam(_ context.Context, teamID uuid.UUID) ([]*model.Member, error) {
	var result []*model.Member
	for _, member := range m.members {
		if member.TeamID == teamID {
			result = append(result, member)
		}
	}
	return result, nil
}

func (m *mockMemberRepo) Create(_ context.Context, member *model.Member) error {
	// Check for duplicate email.
	for _, existing := range m.members {
		if existing.Email == member.Email {
			return fmt.Errorf("duplicate key value violates unique constraint")
		}
	}
	member.ID = uuid.New()
	member.CreatedAt = time.Now()
	m.members = append(m.members, member)
	return nil
}

func (m *mockMemberRepo) Update(_ context.Context, member *model.Member) error {
	for i, existing := range m.members {
		if existing.ID == member.ID {
			m.members[i] = member
			return nil
		}
	}
	return fmt.Errorf("member not found")
}

func (m *mockMemberRepo) Delete(_ context.Context, id uuid.UUID) error {
	for i, existing := range m.members {
		if existing.ID == id {
			m.members = append(m.members[:i], m.members[i+1:]...)
			return nil
		}
	}
	return fmt.Errorf("member not found")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

var (
	ownerID  = uuid.MustParse("cccccccc-cccc-cccc-cccc-cccccccccccc")
	adminID  = uuid.MustParse("dddddddd-dddd-dddd-dddd-dddddddddddd")
)

func newMemberTestServer(memberRepo *mockMemberRepo, auditRepo *mockAuditLogRepo) *Server {
	return &Server{
		Members:   memberRepo,
		AuditLogs: auditRepo,
	}
}

func newTestMember(teamID uuid.UUID, name, email, role string) *model.Member {
	return &model.Member{
		ID:           uuid.New(),
		TeamID:       teamID,
		Name:         name,
		Email:        email,
		Role:         role,
		PasswordHash: "hashed_password",
		TotpEnabled:  false,
		CreatedAt:    time.Now(),
	}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

func TestHandleListMembers_ReturnsTeamMembers(t *testing.T) {
	owner := &model.Member{
		ID:           ownerID,
		TeamID:       testTeamID,
		Name:         "Owner",
		Email:        "owner@example.com",
		Role:         "owner",
		PasswordHash: "hashed",
		CreatedAt:    time.Now(),
	}
	manager := newTestMember(testTeamID, "Manager", "manager@example.com", "manager")
	member := newTestMember(testTeamID, "Member", "member@example.com", "member")
	otherTeamMember := newTestMember(uuid.New(), "Other", "other@example.com", "member")

	memberRepo := &mockMemberRepo{
		members: []*model.Member{owner, manager, member, otherTeamMember},
	}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/"+testTeamID.String()+"/members", nil)
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleListMembers(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp memberListResponse
	decodeJSON(t, rec.Body, &resp)

	if resp.Total != 3 {
		t.Errorf("total = %d, want 3 (only members of testTeamID)", resp.Total)
	}
	if len(resp.Members) != 3 {
		t.Errorf("members count = %d, want 3", len(resp.Members))
	}

	// Verify the other-team member is not included.
	for _, m := range resp.Members {
		if m.Email == "other@example.com" {
			t.Error("other team member should not be included")
		}
	}
}

func TestHandleListMembers_InvalidTeamID(t *testing.T) {
	srv := newMemberTestServer(&mockMemberRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodGet,
		"/api/v1/teams/not-a-uuid/members", nil)
	req = withURLParams(req, map[string]string{"teamID": "not-a-uuid"})
	rec := httptest.NewRecorder()

	srv.handleListMembers(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleInviteMember_Succeeds(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	auditRepo := &mockAuditLogRepo{}
	srv := newMemberTestServer(memberRepo, auditRepo)

	body, _ := json.Marshal(inviteMemberRequest{
		Name:  "New Member",
		Email: "new@example.com",
		Role:  "member",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/invite", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})

	// Set claims to identify the actor.
	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleInviteMember(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp memberJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.Name != "New Member" {
		t.Errorf("name = %q, want New Member", resp.Name)
	}
	if resp.Email != "new@example.com" {
		t.Errorf("email = %q, want new@example.com", resp.Email)
	}
	if resp.Role != "member" {
		t.Errorf("role = %q, want member", resp.Role)
	}
	if resp.ID == "" {
		t.Error("id should be assigned after creation")
	}

	// Verify member was stored.
	if len(memberRepo.members) != 1 {
		t.Errorf("repo member count = %d, want 1", len(memberRepo.members))
	}

	// Verify audit log was created.
	if len(auditRepo.logs) != 1 {
		t.Fatalf("audit log count = %d, want 1", len(auditRepo.logs))
	}
	if auditRepo.logs[0].Action != "member.invite" {
		t.Errorf("audit log action = %q, want member.invite", auditRepo.logs[0].Action)
	}
}

func TestHandleInviteMember_DefaultRole(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(inviteMemberRequest{
		Name:  "Default Role",
		Email: "default@example.com",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/invite", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleInviteMember(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected status 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp memberJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.Role != "member" {
		t.Errorf("role = %q, want member (default)", resp.Role)
	}
}

func TestHandleInviteMember_InvalidRole(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(inviteMemberRequest{
		Name:  "Bad Role",
		Email: "bad@example.com",
		Role:  "superadmin",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/invite", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleInviteMember(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleInviteMember_MissingFields(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	tests := []struct {
		name string
		req  inviteMemberRequest
	}{
		{"missing name", inviteMemberRequest{Email: "a@b.com"}},
		{"missing email", inviteMemberRequest{Name: "A"}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			body, _ := json.Marshal(tt.req)
			req := httptest.NewRequest(http.MethodPost,
				"/api/v1/teams/"+testTeamID.String()+"/invite", bytes.NewReader(body))
			req.Header.Set("Content-Type", "application/json")
			req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
			rec := httptest.NewRecorder()

			srv.handleInviteMember(rec, req)

			if rec.Code != http.StatusBadRequest {
				t.Errorf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestHandleInviteMember_DuplicateEmail(t *testing.T) {
	existing := newTestMember(testTeamID, "Existing", "dup@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{existing}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(inviteMemberRequest{
		Name:  "Duplicate",
		Email: "dup@example.com",
		Role:  "member",
	})

	req := httptest.NewRequest(http.MethodPost,
		"/api/v1/teams/"+testTeamID.String()+"/invite", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{"teamID": testTeamID.String()})
	rec := httptest.NewRecorder()

	srv.handleInviteMember(rec, req)

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected status 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleUpdateMember_Role(t *testing.T) {
	member := newTestMember(testTeamID, "Member", "member@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	auditRepo := &mockAuditLogRepo{}
	srv := newMemberTestServer(memberRepo, auditRepo)

	newRole := "manager"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &newRole,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp memberJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.Role != "manager" {
		t.Errorf("role = %q, want manager", resp.Role)
	}

	// Verify audit log was created.
	if len(auditRepo.logs) != 1 {
		t.Fatalf("audit log count = %d, want 1", len(auditRepo.logs))
	}
	if auditRepo.logs[0].Action != "member.update" {
		t.Errorf("audit log action = %q, want member.update", auditRepo.logs[0].Action)
	}
}

func TestHandleUpdateMember_Name(t *testing.T) {
	member := newTestMember(testTeamID, "Old Name", "member@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	newName := "New Name"
	body, _ := json.Marshal(updateMemberRequest{
		Name: &newName,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp memberJSON
	decodeJSON(t, rec.Body, &resp)

	if resp.Name != "New Name" {
		t.Errorf("name = %q, want New Name", resp.Name)
	}
}

func TestHandleUpdateMember_CannotChangeOwnerRole(t *testing.T) {
	owner := &model.Member{
		ID:           ownerID,
		TeamID:       testTeamID,
		Name:         "Owner",
		Email:        "owner@example.com",
		Role:         "owner",
		PasswordHash: "hashed",
		CreatedAt:    time.Now(),
	}
	memberRepo := &mockMemberRepo{members: []*model.Member{owner}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	newRole := "member"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &newRole,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+ownerID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": ownerID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected status 403, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	decodeJSON(t, rec.Body, &resp)
	if resp["error"] != "cannot_change_owner_role" {
		t.Errorf("error = %q, want cannot_change_owner_role", resp["error"])
	}
}

func TestHandleUpdateMember_CannotChangeOwnRole(t *testing.T) {
	manager := &model.Member{
		ID:           adminID,
		TeamID:       testTeamID,
		Name:         "Manager",
		Email:        "manager@example.com",
		Role:         "manager",
		PasswordHash: "hashed",
		CreatedAt:    time.Now(),
	}
	memberRepo := &mockMemberRepo{members: []*model.Member{manager}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	newRole := "member"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &newRole,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+adminID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": adminID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected status 403, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	decodeJSON(t, rec.Body, &resp)
	if resp["error"] != "cannot_change_own_role" {
		t.Errorf("error = %q, want cannot_change_own_role", resp["error"])
	}
}

func TestHandleUpdateMember_NotFound(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	newRole := "manager"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &newRole,
	})

	nonexistentID := uuid.New().String()
	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+nonexistentID,
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": nonexistentID,
	})
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleUpdateMember_WrongTeam(t *testing.T) {
	member := newTestMember(uuid.New(), "Other Team", "other@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	newRole := "manager"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &newRole,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404 for wrong team, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleRemoveMember_Succeeds(t *testing.T) {
	member := newTestMember(testTeamID, "ToRemove", "remove@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	auditRepo := &mockAuditLogRepo{}
	srv := newMemberTestServer(memberRepo, auditRepo)

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(), nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected status 204, got %d: %s", rec.Code, rec.Body.String())
	}

	// Verify member was removed from repository.
	if len(memberRepo.members) != 0 {
		t.Errorf("repo member count = %d, want 0 after removal", len(memberRepo.members))
	}

	// Verify audit log was created.
	if len(auditRepo.logs) != 1 {
		t.Fatalf("audit log count = %d, want 1", len(auditRepo.logs))
	}
	if auditRepo.logs[0].Action != "member.remove" {
		t.Errorf("audit log action = %q, want member.remove", auditRepo.logs[0].Action)
	}
}

func TestHandleRemoveMember_CannotRemoveSelf(t *testing.T) {
	manager := &model.Member{
		ID:           adminID,
		TeamID:       testTeamID,
		Name:         "Manager",
		Email:        "manager@example.com",
		Role:         "manager",
		PasswordHash: "hashed",
		CreatedAt:    time.Now(),
	}
	memberRepo := &mockMemberRepo{members: []*model.Member{manager}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+adminID.String(), nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": adminID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected status 403, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	decodeJSON(t, rec.Body, &resp)
	if resp["error"] != "cannot_remove_self" {
		t.Errorf("error = %q, want cannot_remove_self", resp["error"])
	}

	// Verify member was NOT removed.
	if len(memberRepo.members) != 1 {
		t.Errorf("repo member count = %d, want 1 (not removed)", len(memberRepo.members))
	}
}

func TestHandleRemoveMember_CannotRemoveOwner(t *testing.T) {
	owner := &model.Member{
		ID:           ownerID,
		TeamID:       testTeamID,
		Name:         "Owner",
		Email:        "owner@example.com",
		Role:         "owner",
		PasswordHash: "hashed",
		CreatedAt:    time.Now(),
	}
	memberRepo := &mockMemberRepo{members: []*model.Member{owner}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+ownerID.String(), nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": ownerID.String(),
	})

	claims := &auth.Claims{
		MemberID: adminID.String(),
		TeamID:   testTeamID.String(),
		Role:     "manager",
	}
	req = withClaims(req, claims)
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected status 403, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	decodeJSON(t, rec.Body, &resp)
	if resp["error"] != "cannot_remove_owner" {
		t.Errorf("error = %q, want cannot_remove_owner", resp["error"])
	}
}

func TestHandleRemoveMember_NotFound(t *testing.T) {
	memberRepo := &mockMemberRepo{members: []*model.Member{}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	nonexistentID := uuid.New().String()
	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+nonexistentID, nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": nonexistentID,
	})
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleRemoveMember_WrongTeam(t *testing.T) {
	member := newTestMember(uuid.New(), "Other Team", "other@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(), nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected status 404 for wrong team, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleRemoveMember_InvalidMemberID(t *testing.T) {
	srv := newMemberTestServer(&mockMemberRepo{}, &mockAuditLogRepo{})

	req := httptest.NewRequest(http.MethodDelete,
		"/api/v1/teams/"+testTeamID.String()+"/members/not-a-uuid", nil)
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": "not-a-uuid",
	})
	rec := httptest.NewRecorder()

	srv.handleRemoveMember(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleUpdateMember_InvalidRole(t *testing.T) {
	member := newTestMember(testTeamID, "Member", "member@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	badRole := "superadmin"
	body, _ := json.Marshal(updateMemberRequest{
		Role: &badRole,
	})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleUpdateMember_NoFieldsProvided(t *testing.T) {
	member := newTestMember(testTeamID, "Member", "member@example.com", "member")
	memberRepo := &mockMemberRepo{members: []*model.Member{member}}
	srv := newMemberTestServer(memberRepo, &mockAuditLogRepo{})

	body, _ := json.Marshal(updateMemberRequest{})

	req := httptest.NewRequest(http.MethodPut,
		"/api/v1/teams/"+testTeamID.String()+"/members/"+member.ID.String(),
		bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = withURLParams(req, map[string]string{
		"teamID":   testTeamID.String(),
		"memberID": member.ID.String(),
	})
	rec := httptest.NewRecorder()

	srv.handleUpdateMember(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}
