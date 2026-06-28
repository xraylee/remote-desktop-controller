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
	"context"
	"encoding/json"
	"log/slog"
	"net/http"

	"github.com/google/uuid"
	"golang.org/x/crypto/bcrypt"

	"github.com/rdcs/rdcs-api/internal/repository"
)

// claimsContextKey is the unexported key type used to store JWT claims in a context.
type claimsContextKey string

const claimsKey claimsContextKey = "claims"

// ContextWithClaims returns a copy of ctx with the given Claims stored,
// so that downstream handlers can retrieve them via ClaimsFromContext.
func ContextWithClaims(ctx context.Context, claims *Claims) context.Context {
	return context.WithValue(ctx, claimsKey, claims)
}

// ClaimsFromContext retrieves the JWT Claims stored in ctx by ContextWithClaims.
// Returns nil if no claims are present.
func ClaimsFromContext(ctx context.Context) *Claims {
	if v, ok := ctx.Value(claimsKey).(*Claims); ok {
		return v
	}
	return nil
}

// LoginRequest is the JSON body expected on POST /api/v1/auth/login.
type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
	TOTPCode string `json:"totp_code,omitempty"`
}

// LoginResponse is returned on successful authentication.
type LoginResponse struct {
	TokenPair
	Member memberInfo `json:"member"`
}

// memberInfo is a safe subset of model.Member that omits sensitive fields.
type memberInfo struct {
	ID        string `json:"id"`
	TeamID    string `json:"team_id"`
	Name      string `json:"name"`
	Email     string `json:"email"`
	Role      string `json:"role"`
	CreatedAt string `json:"created_at"`
}

// Handler groups the HTTP handlers exposed by the auth package.
type Handler struct {
	Members    repository.MemberRepository
	PrivateKey string
	Issuer     string
}

// NewHandler creates a new auth Handler with the given dependencies.
func NewHandler(members repository.MemberRepository, privateKey string, issuer string) *Handler {
	return &Handler{
		Members:    members,
		PrivateKey: privateKey,
		Issuer:     issuer,
	}
}

// HandleLogin validates email+password credentials and returns a JWT
// token pair together with basic member information.
//
// When the member has TOTP two-factor authentication enabled, a valid
// six-digit totp_code must be included in the request body.
//
// POST /api/v1/auth/login
func (h *Handler) HandleLogin(w http.ResponseWriter, r *http.Request) {
	var req LoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	if req.Email == "" || req.Password == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "email and password are required"})
		return
	}

	member, err := h.Members.GetByEmail(r.Context(), req.Email)
	if err != nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "invalid_credentials"})
		return
	}

	if err := bcrypt.CompareHashAndPassword([]byte(member.PasswordHash), []byte(req.Password)); err != nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "invalid_credentials"})
		return
	}

	// TOTP two-factor authentication check.
	if member.TotpEnabled {
		if req.TOTPCode == "" {
			writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "totp_code_required"})
			return
		}
		if member.TotpSecret == nil || !ValidateTOTPCode(*member.TotpSecret, req.TOTPCode) {
			writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "invalid_totp_code"})
			return
		}
	}

	tokens, err := GenerateTokenPair(member, h.PrivateKey)
	if err != nil {
		slog.Error("failed to generate token pair", "error", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	resp := LoginResponse{
		TokenPair: *tokens,
		Member: memberInfo{
			ID:        member.ID.String(),
			TeamID:    member.TeamID.String(),
			Name:      member.Name,
			Email:     member.Email,
			Role:      member.Role,
			CreatedAt: member.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
		},
	}

	writeJSON(w, http.StatusOK, resp)
}

// totpCodeRequest is the JSON body for TOTP verify and disable endpoints.
type totpCodeRequest struct {
	Code string `json:"code"`
}

// totpSetupResponse is returned by HandleTOTPSetup with the secret and QR URI.
type totpSetupResponse struct {
	Secret string `json:"secret"`
	URI    string `json:"uri"`
}

// totpEnabledResponse is returned by HandleTOTPVerify and HandleTOTPDisable.
type totpEnabledResponse struct {
	Enabled bool `json:"enabled"`
}

// HandleTOTPSetup generates a new TOTP secret for the authenticated member
// and stores it (without enabling TOTP).  The client should display the
// returned URI as a QR code so the member can enroll their authenticator app.
//
// POST /api/v1/auth/totp/setup (requires authentication)
func (h *Handler) HandleTOTPSetup(w http.ResponseWriter, r *http.Request) {
	claims := ClaimsFromContext(r.Context())
	if claims == nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "unauthorized"})
		return
	}

	memberID, err := parseUUID(claims.MemberID)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_member_id"})
		return
	}

	member, err := h.Members.GetByID(r.Context(), memberID)
	if err != nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	if member.TotpEnabled {
		writeJSON(w, http.StatusConflict, map[string]string{"error": "totp_already_enabled"})
		return
	}

	secret, err := GenerateTOTPSecret()
	if err != nil {
		slog.Error("failed to generate TOTP secret", "error", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	// Store the secret but do NOT enable TOTP yet — the member must verify
	// a code first via HandleTOTPVerify.
	member.TotpSecret = &secret
	if err := h.Members.Update(r.Context(), member); err != nil {
		slog.Error("failed to store TOTP secret", "error", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	uri := GenerateTOTPURI(secret, member.Email, h.Issuer)
	writeJSON(w, http.StatusOK, totpSetupResponse{
		Secret: secret,
		URI:    uri,
	})
}

// HandleTOTPVerify validates a 6-digit TOTP code against the member's stored
// secret.  On success, TOTP two-factor authentication is enabled for the member.
//
// POST /api/v1/auth/totp/verify (requires authentication)
// Request body: {"code": "123456"}
func (h *Handler) HandleTOTPVerify(w http.ResponseWriter, r *http.Request) {
	claims := ClaimsFromContext(r.Context())
	if claims == nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "unauthorized"})
		return
	}

	var req totpCodeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	if req.Code == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "code is required"})
		return
	}

	memberID, err := parseUUID(claims.MemberID)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_member_id"})
		return
	}

	member, err := h.Members.GetByID(r.Context(), memberID)
	if err != nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	if member.TotpSecret == nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "totp_not_setup"})
		return
	}

	if member.TotpEnabled {
		writeJSON(w, http.StatusConflict, map[string]string{"error": "totp_already_enabled"})
		return
	}

	if !ValidateTOTPCode(*member.TotpSecret, req.Code) {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "invalid_totp_code"})
		return
	}

	member.TotpEnabled = true
	if err := h.Members.Update(r.Context(), member); err != nil {
		slog.Error("failed to enable TOTP", "error", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	writeJSON(w, http.StatusOK, totpEnabledResponse{Enabled: true})
}

// HandleTOTPDisable turns off TOTP two-factor authentication for the
// authenticated member.  The member must supply a current valid TOTP code
// to confirm the operation.
//
// POST /api/v1/auth/totp/disable (requires authentication)
// Request body: {"code": "123456"}
func (h *Handler) HandleTOTPDisable(w http.ResponseWriter, r *http.Request) {
	claims := ClaimsFromContext(r.Context())
	if claims == nil {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "unauthorized"})
		return
	}

	var req totpCodeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	if req.Code == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "code is required"})
		return
	}

	memberID, err := parseUUID(claims.MemberID)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_member_id"})
		return
	}

	member, err := h.Members.GetByID(r.Context(), memberID)
	if err != nil {
		writeJSON(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	if !member.TotpEnabled {
		writeJSON(w, http.StatusConflict, map[string]string{"error": "totp_not_enabled"})
		return
	}

	if member.TotpSecret == nil || !ValidateTOTPCode(*member.TotpSecret, req.Code) {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "invalid_totp_code"})
		return
	}

	member.TotpEnabled = false
	member.TotpSecret = nil
	if err := h.Members.Update(r.Context(), member); err != nil {
		slog.Error("failed to disable TOTP", "error", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	writeJSON(w, http.StatusOK, totpEnabledResponse{Enabled: false})
}

// parseUUID converts a string to a uuid.UUID, returning an error if the
// string is not a valid UUID.
func parseUUID(s string) (uuid.UUID, error) {
	return uuid.Parse(s)
}

// writeJSON encodes v as JSON and writes it to w with the given status code.
func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(v); err != nil {
		slog.Error("failed to write json response", "error", err)
	}
}
