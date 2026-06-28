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

// TeamRepository defines operations on the teams table.
type TeamRepository interface {
	GetByID(ctx context.Context, id uuid.UUID) (*model.Team, error)
	Create(ctx context.Context, team *model.Team) error
	Update(ctx context.Context, team *model.Team) error
}

type teamRepo struct {
	db *sqlx.DB
}

// NewTeamRepository creates a new TeamRepository backed by sqlx.
func NewTeamRepository(db *sqlx.DB) TeamRepository {
	return &teamRepo{db: db}
}

func (r *teamRepo) GetByID(ctx context.Context, id uuid.UUID) (*model.Team, error) {
	var team model.Team
	err := r.db.GetContext(ctx, &team,
		`SELECT id, name, plan, max_concurrent, created_at, updated_at
		 FROM teams WHERE id = $1`, id)
	if err != nil {
		return nil, fmt.Errorf("get team by id: %w", err)
	}
	return &team, nil
}

func (r *teamRepo) Create(ctx context.Context, team *model.Team) error {
	err := r.db.QueryRowxContext(ctx,
		`INSERT INTO teams (name, plan, max_concurrent)
		 VALUES ($1, $2, $3)
		 RETURNING id, created_at, updated_at`,
		team.Name, team.Plan, team.MaxConcurrent,
	).Scan(&team.ID, &team.CreatedAt, &team.UpdatedAt)
	if err != nil {
		return fmt.Errorf("create team: %w", err)
	}
	return nil
}

func (r *teamRepo) Update(ctx context.Context, team *model.Team) error {
	result, err := r.db.ExecContext(ctx,
		`UPDATE teams
		 SET name = $1, plan = $2, max_concurrent = $3, updated_at = NOW()
		 WHERE id = $4`,
		team.Name, team.Plan, team.MaxConcurrent, team.ID,
	)
	if err != nil {
		return fmt.Errorf("update team: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("update team rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("update team: not found")
	}
	return nil
}
