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

// RecordingRepository defines operations on the recordings table.
type RecordingRepository interface {
	List(ctx context.Context, teamID uuid.UUID, limit, offset int) ([]*model.Recording, error)
	GetByID(ctx context.Context, id uuid.UUID) (*model.Recording, error)
}

type recordingRepo struct {
	db *sqlx.DB
}

// NewRecordingRepository creates a new RecordingRepository backed by sqlx.
func NewRecordingRepository(db *sqlx.DB) RecordingRepository {
	return &recordingRepo{db: db}
}

func (r *recordingRepo) List(ctx context.Context, teamID uuid.UUID, limit, offset int) ([]*model.Recording, error) {
	if limit <= 0 {
		limit = 50
	}
	if offset < 0 {
		offset = 0
	}

	var recordings []*model.Recording
	err := r.db.SelectContext(ctx, &recordings,
		`SELECT id, connection_id, team_id, encryption_key, storage_path,
		        duration_sec, file_size, created_at, expires_at
		 FROM recordings
		 WHERE team_id = $1
		 ORDER BY created_at DESC
		 LIMIT $2 OFFSET $3`,
		teamID, limit, offset)
	if err != nil {
		return nil, fmt.Errorf("list recordings: %w", err)
	}
	return recordings, nil
}

func (r *recordingRepo) GetByID(ctx context.Context, id uuid.UUID) (*model.Recording, error) {
	var recording model.Recording
	err := r.db.GetContext(ctx, &recording,
		`SELECT id, connection_id, team_id, encryption_key, storage_path,
		        duration_sec, file_size, created_at, expires_at
		 FROM recordings WHERE id = $1`, id)
	if err != nil {
		return nil, fmt.Errorf("get recording by id: %w", err)
	}
	return &recording, nil
}
