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
	"encoding/base32"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"

	"github.com/rdcs/rdcs-api/internal/model"
)

// ---------------------------------------------------------------------------
// Unit tests for TOTP core functions (totp.go)
// ---------------------------------------------------------------------------

func TestGenerateTOTPSecret(t *testing.T) {
	secret, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() error = %v", err)
	}

	// Must be valid base32 (no padding).
	decoded, err := base32.StdEncoding.WithPadding(base32.NoPadding).DecodeString(secret)
	if err != nil {
		t.Fatalf("secret is not valid base32: %v", err)
	}

	if len(decoded) != TOTP_SECRET_LENGTH {
		t.Errorf("decoded secret length = %d, want %d", len(decoded), TOTP_SECRET_LENGTH)
	}

	// Two secrets should be different (cryptographic randomness).
	secret2, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() second call error = %v", err)
	}
	if secret == secret2 {
		t.Error("two generated secrets should not be identical")
	}
}

func TestGenerateTOTPURI(t *testing.T) {
	secret := "JBSWY3DPEHPK3PXP"
	email := "user@example.com"
	issuer := "RDCS"

	uri := GenerateTOTPURI(secret, email, issuer)

	if !strings.HasPrefix(uri, "otpauth://totp/") {
		t.Errorf("URI does not start with otpauth://totp/: %s", uri)
	}

	if !strings.Contains(uri, "secret="+secret) {
		t.Errorf("URI does not contain secret: %s", uri)
	}

	if !strings.Contains(uri, "issuer="+issuer) {
		t.Errorf("URI does not contain issuer: %s", uri)
	}

	if !strings.Contains(uri, "algorithm=SHA1") {
		t.Errorf("URI does not contain algorithm=SHA1: %s", uri)
	}

	if !strings.Contains(uri, "digits=6") {
		t.Errorf("URI does not contain digits=6: %s", uri)
	}

	if !strings.Contains(uri, "period=30") {
		t.Errorf("URI does not contain period=30: %s", uri)
	}
}

func TestValidateTOTPCode_ValidCode(t *testing.T) {
	secret, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() error = %v", err)
	}

	now := time.Now()
	code, err := GenerateTOTPCodeAtTime(secret, now)
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime() error = %v", err)
	}

	if !ValidateTOTPCode(secret, code) {
		t.Errorf("ValidateTOTPCode() = false for valid code %q", code)
	}
}

func TestValidateTOTPCode_InvalidCode(t *testing.T) {
	secret, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() error = %v", err)
	}

	if ValidateTOTPCode(secret, "000000") {
		t.Error("ValidateTOTPCode() = true for likely-invalid code 000000")
	}
}

func TestValidateTOTPCode_AdjacentWindow(t *testing.T) {
	secret, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() error = %v", err)
	}

	// Generate a code for 30 seconds ago (previous window).
	prevTime := time.Now().Add(-TOTP_PERIOD * time.Second)
	prevCode, err := GenerateTOTPCodeAtTime(secret, prevTime)
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime(prev) error = %v", err)
	}

	// The code from the previous window should still be accepted (skew=1).
	if !ValidateTOTPCode(secret, prevCode) {
		t.Errorf("ValidateTOTPCode() = false for previous-window code %q", prevCode)
	}

	// Generate a code for 30 seconds in the future (next window).
	nextTime := time.Now().Add(TOTP_PERIOD * time.Second)
	nextCode, err := GenerateTOTPCodeAtTime(secret, nextTime)
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime(next) error = %v", err)
	}

	if !ValidateTOTPCode(secret, nextCode) {
		t.Errorf("ValidateTOTPCode() = false for next-window code %q", nextCode)
	}
}

func TestValidateTOTPCodeAtTime_ExpiredCode(t *testing.T) {
	secret, err := GenerateTOTPSecret()
	if err != nil {
		t.Fatalf("GenerateTOTPSecret() error = %v", err)
	}

	// Generate a code for 90 seconds ago (outside ±1 skew window).
	oldTime := time.Now().Add(-3 * TOTP_PERIOD * time.Second)
	oldCode, err := GenerateTOTPCodeAtTime(secret, oldTime)
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime(old) error = %v", err)
	}

	// Should be rejected since it's outside the ±30s tolerance.
	if ValidateTOTPCodeAtTime(secret, oldCode, time.Now()) {
		t.Errorf("ValidateTOTPCodeAtTime() = true for expired code %q", oldCode)
	}
}

// ---------------------------------------------------------------------------
// Integration tests for TOTP handler endpoints
// ---------------------------------------------------------------------------

func newTestMemberWithTOTP(password string, secret string, enabled bool) *model.Member {
	hash, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	return &model.Member{
		ID:           uuid.MustParse("11111111-1111-1111-1111-111111111111"),
		TeamID:       uuid.MustParse("22222222-2222-2222-2222-222222222222"),
		Name:         "Test User",
		Email:        "test@example.com",
		Role:         "admin",
		PasswordHash: string(hash),
		TotpSecret:   &secret,
		TotpEnabled:  enabled,
		CreatedAt:    time.Now(),
	}
}

// contextWithTestClaims creates a request context with JWT claims for testing.
func contextWithTestClaims(r *http.Request, memberID string) *http.Request {
	claims := &Claims{
		MemberID: memberID,
		TeamID:   "22222222-2222-2222-2222-222222222222",
		Role:     "admin",
	}
	return r.WithContext(ContextWithClaims(r.Context(), claims))
}

func TestHandleLogin_TOTPRequired_NoCode(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("correct-password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, privPEM, "RDCS")

	body, _ := json.Marshal(LoginRequest{
		Email:    "test@example.com",
		Password: "correct-password",
		// No TOTPCode provided.
	})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	json.NewDecoder(rec.Body).Decode(&resp)
	if resp["error"] != "totp_code_required" {
		t.Errorf("error = %q, want %q", resp["error"], "totp_code_required")
	}
}

func TestHandleLogin_TOTPRequired_InvalidCode(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("correct-password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, privPEM, "RDCS")

	body, _ := json.Marshal(LoginRequest{
		Email:    "test@example.com",
		Password: "correct-password",
		TOTPCode: "000000",
	})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()

	handler.HandleLogin(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp map[string]string
	json.NewDecoder(rec.Body).Decode(&resp)
	if resp["error"] != "invalid_totp_code" {
		t.Errorf("error = %q, want %q", resp["error"], "invalid_totp_code")
	}
}

func TestHandleLogin_TOTPRequired_ValidCode(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("correct-password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, privPEM, "RDCS")

	code, err := GenerateTOTPCodeAtTime(secret, time.Now())
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime() error = %v", err)
	}

	body, _ := json.Marshal(LoginRequest{
		Email:    "test@example.com",
		Password: "correct-password",
		TOTPCode: code,
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
}

func TestHandleLogin_TOTPDisabled_NoCodeNeeded(t *testing.T) {
	privPEM, _, err := GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}

	// Member with TOTP not enabled.
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
}

func TestHandleTOTPSetup_Success(t *testing.T) {
	member := newTestMemberWithPassword("password")
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/setup", nil)
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPSetup(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp totpSetupResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if resp.Secret == "" {
		t.Error("secret is empty")
	}
	if !strings.HasPrefix(resp.URI, "otpauth://totp/") {
		t.Errorf("URI does not start with otpauth://totp/: %s", resp.URI)
	}
	if !strings.Contains(resp.URI, "RDCS") {
		t.Errorf("URI does not contain issuer RDCS: %s", resp.URI)
	}

	// Verify the secret was persisted on the member.
	updated := repo.members[0]
	if updated.TotpSecret == nil || *updated.TotpSecret != resp.Secret {
		t.Error("TOTP secret was not persisted on the member")
	}
	if updated.TotpEnabled {
		t.Error("TotpEnabled should still be false after setup")
	}
}

func TestHandleTOTPSetup_AlreadyEnabled(t *testing.T) {
	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/setup", nil)
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPSetup(rec, req)

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected status 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPSetup_Unauthorized(t *testing.T) {
	handler := NewHandler(&mockMemberRepo{}, "", "RDCS")

	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/setup", nil)
	// No claims in context.
	rec := httptest.NewRecorder()

	handler.HandleTOTPSetup(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPVerify_Success(t *testing.T) {
	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithPassword("password")
	member.TotpSecret = &secret
	member.TotpEnabled = false
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	code, err := GenerateTOTPCodeAtTime(secret, time.Now())
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime() error = %v", err)
	}

	body, _ := json.Marshal(totpCodeRequest{Code: code})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/verify", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPVerify(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp totpEnabledResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if !resp.Enabled {
		t.Error("expected enabled = true")
	}

	// Verify member state was updated.
	updated := repo.members[0]
	if !updated.TotpEnabled {
		t.Error("member.TotpEnabled should be true after verify")
	}
}

func TestHandleTOTPVerify_InvalidCode(t *testing.T) {
	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithPassword("password")
	member.TotpSecret = &secret
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	body, _ := json.Marshal(totpCodeRequest{Code: "000000"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/verify", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPVerify(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPVerify_NotSetup(t *testing.T) {
	member := newTestMemberWithPassword("password")
	// No TotpSecret set.
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	body, _ := json.Marshal(totpCodeRequest{Code: "123456"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/verify", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPVerify(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPVerify_EmptyCode(t *testing.T) {
	member := newTestMemberWithPassword("password")
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	body, _ := json.Marshal(totpCodeRequest{Code: ""})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/verify", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPVerify(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected status 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPDisable_Success(t *testing.T) {
	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	code, err := GenerateTOTPCodeAtTime(secret, time.Now())
	if err != nil {
		t.Fatalf("GenerateTOTPCodeAtTime() error = %v", err)
	}

	body, _ := json.Marshal(totpCodeRequest{Code: code})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/disable", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPDisable(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp totpEnabledResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if resp.Enabled {
		t.Error("expected enabled = false")
	}

	// Verify member state was cleared.
	updated := repo.members[0]
	if updated.TotpEnabled {
		t.Error("member.TotpEnabled should be false after disable")
	}
	if updated.TotpSecret != nil {
		t.Error("member.TotpSecret should be nil after disable")
	}
}

func TestHandleTOTPDisable_NotEnabled(t *testing.T) {
	member := newTestMemberWithPassword("password")
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	body, _ := json.Marshal(totpCodeRequest{Code: "123456"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/disable", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPDisable(rec, req)

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected status 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHandleTOTPDisable_InvalidCode(t *testing.T) {
	secret, _ := GenerateTOTPSecret()
	member := newTestMemberWithTOTP("password", secret, true)
	repo := &mockMemberRepo{members: []*model.Member{member}}
	handler := NewHandler(repo, "", "RDCS")

	body, _ := json.Marshal(totpCodeRequest{Code: "000000"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/totp/disable", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req = contextWithTestClaims(req, member.ID.String())
	rec := httptest.NewRecorder()

	handler.HandleTOTPDisable(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d: %s", rec.Code, rec.Body.String())
	}

	// Member should still be enabled since disable failed.
	if !repo.members[0].TotpEnabled {
		t.Error("member should still have TOTP enabled after failed disable")
	}
}

// ---------------------------------------------------------------------------
// Context helper tests
// ---------------------------------------------------------------------------

func TestClaimsFromContext_NilWhenEmpty(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	claims := ClaimsFromContext(req.Context())
	if claims != nil {
		t.Error("expected nil claims from empty context")
	}
}

func TestClaimsFromContext_RoundTrip(t *testing.T) {
	original := &Claims{
		MemberID: "abc-123",
		TeamID:   "team-456",
		Role:     "admin",
	}
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	req = req.WithContext(ContextWithClaims(req.Context(), original))

	got := ClaimsFromContext(req.Context())
	if got == nil {
		t.Fatal("expected non-nil claims")
	}
	if got.MemberID != original.MemberID {
		t.Errorf("MemberID = %q, want %q", got.MemberID, original.MemberID)
	}
	if got.TeamID != original.TeamID {
		t.Errorf("TeamID = %q, want %q", got.TeamID, original.TeamID)
	}
	if got.Role != original.Role {
		t.Errorf("Role = %q, want %q", got.Role, original.Role)
	}
}
