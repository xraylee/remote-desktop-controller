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

package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"

	"github.com/rdcs/rdcs-api/internal/model"
)

// mockMemberRepo is a minimal in-memory MemberRepository for testing.
type mockMemberRepo struct {
	members []*model.Member
}

func (m *mockMemberRepo) GetByID(_ context.Context, id uuid.UUID) (*model.Member, error) {
	for _, mem := range m.members {
		if mem.ID == id {
			return mem, nil
		}
	}
	return nil, errNotFound
}

func (m *mockMemberRepo) GetByEmail(_ context.Context, email string) (*model.Member, error) {
	for _, mem := range m.members {
		if mem.Email == email {
			return mem, nil
		}
	}
	return nil, errNotFound
}

func (m *mockMemberRepo) ListByTeam(_ context.Context, _ uuid.UUID) ([]*model.Member, error) {
	return m.members, nil
}

func (m *mockMemberRepo) Create(_ context.Context, member *model.Member) error {
	member.ID = uuid.New()
	member.CreatedAt = time.Now()
	m.members = append(m.members, member)
	return nil
}

func (m *mockMemberRepo) Update(_ context.Context, member *model.Member) error {
	for i, mem := range m.members {
		if mem.ID == member.ID {
			m.members[i] = member
			return nil
		}
	}
	return errNotFound
}

func (m *mockMemberRepo) Delete(_ context.Context, id uuid.UUID) error {
	for i, mem := range m.members {
		if mem.ID == id {
			m.members = append(m.members[:i], m.members[i+1:]...)
			return nil
		}
	}
	return errNotFound
}

var errNotFound = &notFoundError{}

type notFoundError struct{}

func (e *notFoundError) Error() string { return "not found" }

func newTestMemberWithPassword(password string) *model.Member {
	hash, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	return &model.Member{
		ID:           uuid.MustParse("11111111-1111-1111-1111-111111111111"),
		TeamID:       uuid.MustParse("22222222-2222-2222-2222-222222222222"),
		Name:         "Test User",
		Email:        "test@example.com",
		Role:         "admin",
		PasswordHash: string(hash),
		CreatedAt:    time.Now(),
	}
}

func TestHandleLogin_ValidCredentials(t *testing.T) {
	privPEM, pubPEM, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMemberWithPassword("correct-password")
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, privPEM, "RDCS")

	body, _ := json.Marshal(LoginRequest{
		Email:    "test@example.com",
		Password: "correct-password",
	})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp LoginResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if resp.AccessToken == "" {
		t.Error("access token is empty")
	}
	if resp.RefreshToken == "" {
		t.Error("refresh token is empty")
	}
	if resp.Member.Email != "test@example.com" {
		t.Errorf("member email = %q, want %q", resp.Member.Email, "test@example.com")
	}
	if resp.Member.Role != "admin" {
		t.Errorf("member role = %q, want %q", resp.Member.Role, "admin")
	}

	// Verify the access token is valid.
	claims, err := ValidateToken(resp.AccessToken, pubPEM)
	if err != nil {
		t.Fatalf("ValidateToken() error = %v", err)
	}
	if claims.MemberID != member.ID.String() {
		t.Errorf("claims MemberID = %q, want %q", claims.MemberID, member.ID.String())
	}
}

func TestHandleLogin_InvalidPassword(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	member := newTestMemberWithPassword("correct-password")
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, privPEM, "RDCS")

	body, _ := json.Marshal(LoginRequest{
		Email:    "test@example.com",
		Password: "wrong-password",
	})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleLogin_UnknownEmail(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	repo := &mockMemberRepo{members: []*model.Member{}}
	handler := NewHandler(repo, privPEM, "RDCS")

	body, _ := json.Marshal(LoginRequest{
		Email:    "nobody@example.com",
		Password: "any-password",
	})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleLogin_EmptyFields(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	repo := &mockMemberRepo{}
	handler := NewHandler(repo, privPEM, "RDCS")

	tests := []struct {
		name string
		body LoginRequest
	}{
		{"empty email", LoginRequest{Email: "", Password: "pass"}},
		{"empty password", LoginRequest{Email: "a@b.com", Password: ""}},
		{"both empty", LoginRequest{Email: "", Password: ""}},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b, _ := json.Marshal(tt.body)
			req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(b))
			req.Header.Set("Content-Type", "application/json")
			rec := httptest.NewRecorder()

			handler.HandleLogin(rec, req)

			if rec.Code != http.StatusBadRequest {
				t.Errorf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestHandleLogin_InvalidJSON(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	repo := &mockMemberRepo{}
	handler := NewHandler(repo, privPEM, "RDCS")

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader([]byte("{invalid json")))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}
