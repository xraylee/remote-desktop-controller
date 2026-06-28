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
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"log/slog"
	"net/http"
	"strings"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/model"
)

// memberListResponse is the JSON envelope for member listings.
type memberListResponse struct {
	Members []memberJSON `json:"members"`
	Total   int          `json:"total"`
}

// memberJSON is the API representation of a member (excludes sensitive fields).
type memberJSON struct {
	ID        string  `json:"id"`
	TeamID    string  `json:"team_id"`
	Name      string  `json:"name"`
	Email     string  `json:"email"`
	Role      string  `json:"role"`
	LastLogin *string `json:"last_login,omitempty"`
	CreatedAt string  `json:"created_at"`
}

// inviteMemberRequest is the JSON body expected on POST /api/v1/teams/{teamID}/invite.
type inviteMemberRequest struct {
	Name  string `json:"name"`
	Email string `json:"email"`
	Role  string `json:"role"`
}

// updateMemberRequest is the JSON body expected on PUT /api/v1/teams/{teamID}/members/{memberID}.
type updateMemberRequest struct {
	Name *string `json:"name,omitempty"`
	Role *string `json:"role,omitempty"`
}

// toMemberJSON converts a model.Member into its API representation.
func toMemberJSON(m *model.Member) memberJSON {
	mj := memberJSON{
		ID:        m.ID.String(),
		TeamID:    m.TeamID.String(),
		Name:      m.Name,
		Email:     m.Email,
		Role:      m.Role,
		CreatedAt: m.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
	if m.LastLogin != nil {
		s := m.LastLogin.UTC().Format("2006-01-02T15:04:05Z")
		mj.LastLogin = &s
	}
	return mj
}

// handleListMembers returns all members belonging to the specified team.
//
// GET /api/v1/teams/{teamID}/members
func (s *Server) handleListMembers(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	members, err := s.Members.ListByTeam(r.Context(), teamID)
	if err != nil {
		slog.Error("failed to list members", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	items := make([]memberJSON, 0, len(members))
	for _, m := range members {
		items = append(items, toMemberJSON(m))
	}

	writeJSONResponse(w, http.StatusOK, memberListResponse{
		Members: items,
		Total:   len(members),
	})
}

// handleInviteMember creates a new member in the specified team with a random
// initial password hash. The member is expected to set their password on first login.
//
// POST /api/v1/teams/{teamID}/invite
// Request body: {"name": "...", "email": "...", "role": "member"|"manager"}
func (s *Server) handleInviteMember(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	var req inviteMemberRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	if req.Name == "" || req.Email == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "name and email are required"})
		return
	}

	if req.Role == "" {
		req.Role = "member"
	}
	validRoles := map[string]bool{"member": true, "manager": true}
	if !validRoles[req.Role] {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "role must be member or manager"})
		return
	}

	// Generate a random initial password hash placeholder.
	randomHash := generateRandomHash()

	member := &model.Member{
		TeamID:       teamID,
		Name:         req.Name,
		Email:        req.Email,
		Role:         req.Role,
		PasswordHash: randomHash,
		TotpEnabled:  false,
	}

	if err := s.Members.Create(r.Context(), member); err != nil {
		// Handle unique constraint violation on email.
		if strings.Contains(err.Error(), "duplicate key") || strings.Contains(err.Error(), "unique constraint") {
			writeJSONResponse(w, http.StatusConflict, map[string]string{"error": "email_already_exists"})
			return
		}
		slog.Error("failed to invite member", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	// Create audit log entry for the invitation.
	claims := auth.ClaimsFromContext(r.Context())
	var actorID *uuid.UUID
	if claims != nil {
		if parsed, parseErr := uuid.Parse(claims.MemberID); parseErr == nil {
			actorID = &parsed
		}
	}

	targetType := "member"
	details, _ := json.Marshal(map[string]string{
		"invited_name":  member.Name,
		"invited_email": member.Email,
		"invited_role":  member.Role,
	})

	auditLog := &model.AuditLog{
		TeamID:     teamID,
		ActorID:    actorID,
		Action:     "member.invite",
		TargetType: &targetType,
		TargetID:   &member.ID,
		Details:    details,
	}
	if err := s.AuditLogs.Create(r.Context(), auditLog); err != nil {
		slog.Error("failed to create audit log for member invitation", "error", err)
		// Do not fail the response — the member was already created.
	}

	writeJSONResponse(w, http.StatusCreated, toMemberJSON(member))
}

// handleUpdateMember updates the name and/or role of an existing member.
// It creates an audit log entry for the change.
//
// PUT /api/v1/teams/{teamID}/members/{memberID}
// Request body: {"name": "...", "role": "member"|"manager"}
// Restrictions:
//   - Cannot change own role
//   - Cannot change the owner's role
func (s *Server) handleUpdateMember(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	memberIDStr := chi.URLParam(r, "memberID")
	if memberIDStr == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "missing_member_id"})
		return
	}
	memberID, err := uuid.Parse(memberIDStr)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_member_id"})
		return
	}

	var req updateMemberRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_request_body"})
		return
	}

	// Nothing to update.
	if req.Name == nil && req.Role == nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "at least one of name or role must be provided"})
		return
	}

	// Validate role if provided.
	if req.Role != nil {
		validRoles := map[string]bool{"member": true, "manager": true}
		if !validRoles[*req.Role] {
			writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "role must be member or manager"})
			return
		}
	}

	// Fetch the target member.
	member, err := s.Members.GetByID(r.Context(), memberID)
	if err != nil {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	// Verify the member belongs to the requested team.
	if member.TeamID != teamID {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	// Cannot change the owner's role.
	if member.Role == "owner" && req.Role != nil {
		writeJSONResponse(w, http.StatusForbidden, map[string]string{"error": "cannot_change_owner_role"})
		return
	}

	// Cannot change own role.
	claims := auth.ClaimsFromContext(r.Context())
	if claims != nil && claims.MemberID == memberID.String() && req.Role != nil {
		writeJSONResponse(w, http.StatusForbidden, map[string]string{"error": "cannot_change_own_role"})
		return
	}

	// Apply updates.
	if req.Name != nil {
		member.Name = *req.Name
	}
	if req.Role != nil {
		member.Role = *req.Role
	}

	if err := s.Members.Update(r.Context(), member); err != nil {
		slog.Error("failed to update member", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	// Create audit log entry for the update.
	var actorID *uuid.UUID
	if claims != nil {
		if parsed, parseErr := uuid.Parse(claims.MemberID); parseErr == nil {
			actorID = &parsed
		}
	}

	targetType := "member"
	detailMap := map[string]string{
		"updated_name": member.Name,
		"updated_role": member.Role,
	}
	details, _ := json.Marshal(detailMap)

	auditLog := &model.AuditLog{
		TeamID:     teamID,
		ActorID:    actorID,
		Action:     "member.update",
		TargetType: &targetType,
		TargetID:   &member.ID,
		Details:    details,
	}
	if err := s.AuditLogs.Create(r.Context(), auditLog); err != nil {
		slog.Error("failed to create audit log for member update", "error", err)
		// Do not fail the response — the member was already updated.
	}

	writeJSONResponse(w, http.StatusOK, toMemberJSON(member))
}

// handleRemoveMember deletes a member from the team and creates an audit log entry.
//
// DELETE /api/v1/teams/{teamID}/members/{memberID}
// Restrictions:
//   - Cannot remove self
//   - Cannot remove the owner
func (s *Server) handleRemoveMember(w http.ResponseWriter, r *http.Request) {
	teamID, err := parseTeamID(r)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_team_id"})
		return
	}

	memberIDStr := chi.URLParam(r, "memberID")
	if memberIDStr == "" {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "missing_member_id"})
		return
	}
	memberID, err := uuid.Parse(memberIDStr)
	if err != nil {
		writeJSONResponse(w, http.StatusBadRequest, map[string]string{"error": "invalid_member_id"})
		return
	}

	// Fetch the target member.
	member, err := s.Members.GetByID(r.Context(), memberID)
	if err != nil {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	// Verify the member belongs to the requested team.
	if member.TeamID != teamID {
		writeJSONResponse(w, http.StatusNotFound, map[string]string{"error": "member_not_found"})
		return
	}

	// Cannot remove the owner.
	if member.Role == "owner" {
		writeJSONResponse(w, http.StatusForbidden, map[string]string{"error": "cannot_remove_owner"})
		return
	}

	// Cannot remove self.
	claims := auth.ClaimsFromContext(r.Context())
	if claims != nil && claims.MemberID == memberID.String() {
		writeJSONResponse(w, http.StatusForbidden, map[string]string{"error": "cannot_remove_self"})
		return
	}

	if err := s.Members.Delete(r.Context(), member.ID); err != nil {
		slog.Error("failed to remove member", "error", err)
		writeJSONResponse(w, http.StatusInternalServerError, map[string]string{"error": "internal_server_error"})
		return
	}

	// Create audit log entry for the removal.
	var actorID *uuid.UUID
	if claims != nil {
		if parsed, parseErr := uuid.Parse(claims.MemberID); parseErr == nil {
			actorID = &parsed
		}
	}

	targetType := "member"
	details, _ := json.Marshal(map[string]string{
		"removed_name":  member.Name,
		"removed_email": member.Email,
		"removed_role":  member.Role,
	})

	auditLog := &model.AuditLog{
		TeamID:     teamID,
		ActorID:    actorID,
		Action:     "member.remove",
		TargetType: &targetType,
		TargetID:   &member.ID,
		Details:    details,
	}
	if err := s.AuditLogs.Create(r.Context(), auditLog); err != nil {
		slog.Error("failed to create audit log for member removal", "error", err)
		// Do not fail the response — the member was already removed.
	}

	w.WriteHeader(http.StatusNoContent)
}

// generateRandomHash creates a random 32-byte hex string used as an initial
// password hash placeholder for newly invited members.
func generateRandomHash() string {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		// Fallback: should never happen in practice.
		return uuid.New().String()
	}
	return hex.EncodeToString(b)
}
