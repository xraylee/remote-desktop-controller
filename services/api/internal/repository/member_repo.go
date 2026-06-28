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

package repository

import (
	"context"
	"fmt"

	"github.com/google/uuid"
	"github.com/jmoiron/sqlx"

	"github.com/rdcs/rdcs-api/internal/model"
)

// MemberRepository defines operations on the members table.
type MemberRepository interface {
	GetByID(ctx context.Context, id uuid.UUID) (*model.Member, error)
	GetByEmail(ctx context.Context, email string) (*model.Member, error)
	ListByTeam(ctx context.Context, teamID uuid.UUID) ([]*model.Member, error)
	Create(ctx context.Context, member *model.Member) error
	Update(ctx context.Context, member *model.Member) error
	Delete(ctx context.Context, id uuid.UUID) error

	// Dashboard extensions
	CountByTeam(ctx context.Context, teamID uuid.UUID) (int, error)
}

type memberRepo struct {
	db *sqlx.DB
}

// NewMemberRepository creates a new MemberRepository backed by sqlx.
func NewMemberRepository(db *sqlx.DB) MemberRepository {
	return &memberRepo{db: db}
}

func (r *memberRepo) GetByID(ctx context.Context, id uuid.UUID) (*model.Member, error) {
	var member model.Member
	err := r.db.GetContext(ctx, &member,
		`SELECT id, team_id, name, email, role, password_hash, totp_secret,
		        totp_enabled, last_login, created_at
		 FROM members WHERE id = $1`, id)
	if err != nil {
		return nil, fmt.Errorf("get member by id: %w", err)
	}
	return &member, nil
}

func (r *memberRepo) GetByEmail(ctx context.Context, email string) (*model.Member, error) {
	var member model.Member
	err := r.db.GetContext(ctx, &member,
		`SELECT id, team_id, name, email, role, password_hash, totp_secret,
		        totp_enabled, last_login, created_at
		 FROM members WHERE email = $1`, email)
	if err != nil {
		return nil, fmt.Errorf("get member by email: %w", err)
	}
	return &member, nil
}

func (r *memberRepo) ListByTeam(ctx context.Context, teamID uuid.UUID) ([]*model.Member, error) {
	var members []*model.Member
	err := r.db.SelectContext(ctx, &members,
		`SELECT id, team_id, name, email, role, password_hash, totp_secret,
		        totp_enabled, last_login, created_at
		 FROM members
		 WHERE team_id = $1
		 ORDER BY created_at ASC`, teamID)
	if err != nil {
		return nil, fmt.Errorf("list members by team: %w", err)
	}
	return members, nil
}

func (r *memberRepo) Create(ctx context.Context, member *model.Member) error {
	err := r.db.QueryRowxContext(ctx,
		`INSERT INTO members (team_id, name, email, role, password_hash, totp_secret, totp_enabled)
		 VALUES ($1, $2, $3, $4, $5, $6, $7)
		 RETURNING id, created_at`,
		member.TeamID, member.Name, member.Email, member.Role,
		member.PasswordHash, member.TotpSecret, member.TotpEnabled,
	).Scan(&member.ID, &member.CreatedAt)
	if err != nil {
		return fmt.Errorf("create member: %w", err)
	}
	return nil
}

func (r *memberRepo) Update(ctx context.Context, member *model.Member) error {
	result, err := r.db.ExecContext(ctx,
		`UPDATE members
		 SET name = $1, email = $2, role = $3, password_hash = $4,
		     totp_secret = $5, totp_enabled = $6, last_login = $7
		 WHERE id = $8`,
		member.Name, member.Email, member.Role, member.PasswordHash,
		member.TotpSecret, member.TotpEnabled, member.LastLogin, member.ID,
	)
	if err != nil {
		return fmt.Errorf("update member: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("update member rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("update member: not found")
	}
	return nil
}

func (r *memberRepo) Delete(ctx context.Context, id uuid.UUID) error {
	result, err := r.db.ExecContext(ctx, `DELETE FROM members WHERE id = $1`, id)
	if err != nil {
		return fmt.Errorf("delete member: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("delete member rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("delete member: not found")
	}
	return nil
}
