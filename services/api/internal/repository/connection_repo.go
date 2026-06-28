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
	"encoding/csv"
	"fmt"
	"io"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jmoiron/sqlx"

	"github.com/rdcs/rdcs-api/internal/model"
)

// ConnFilter specifies optional filters for listing connection records.
type ConnFilter struct {
	ControllerCode string
	ControlledCode string
	Path           string // "L1", "L2", or "L3"
	StartedAfter   *time.Time
	StartedBefore  *time.Time
	Limit          int
	Offset         int
}

// ConnectionRecordRepository defines operations on the connection_records table.
type ConnectionRecordRepository interface {
	List(ctx context.Context, teamID uuid.UUID, filter ConnFilter) ([]*model.ConnectionRecord, error)
	Count(ctx context.Context, teamID uuid.UUID, filter ConnFilter) (int, error)
	Create(ctx context.Context, record *model.ConnectionRecord) error
	ExportCSV(ctx context.Context, teamID uuid.UUID, filter ConnFilter, w io.Writer) error
}

type connectionRecordRepo struct {
	db *sqlx.DB
}

// NewConnectionRecordRepository creates a new ConnectionRecordRepository backed by sqlx.
func NewConnectionRecordRepository(db *sqlx.DB) ConnectionRecordRepository {
	return &connectionRecordRepo{db: db}
}

func (r *connectionRecordRepo) buildWhereClause(teamID uuid.UUID, filter ConnFilter) (string, []interface{}) {
	var args []interface{}
	var conditions []string

	args = append(args, teamID)
	conditions = append(conditions, fmt.Sprintf("team_id = $%d", len(args)))

	if filter.ControllerCode != "" {
		args = append(args, filter.ControllerCode)
		conditions = append(conditions, fmt.Sprintf("controller_code = $%d", len(args)))
	}
	if filter.ControlledCode != "" {
		args = append(args, filter.ControlledCode)
		conditions = append(conditions, fmt.Sprintf("controlled_code = $%d", len(args)))
	}
	if filter.Path != "" {
		args = append(args, filter.Path)
		conditions = append(conditions, fmt.Sprintf("path = $%d", len(args)))
	}
	if filter.StartedAfter != nil {
		args = append(args, *filter.StartedAfter)
		conditions = append(conditions, fmt.Sprintf("started_at >= $%d", len(args)))
	}
	if filter.StartedBefore != nil {
		args = append(args, *filter.StartedBefore)
		conditions = append(conditions, fmt.Sprintf("started_at <= $%d", len(args)))
	}

	return strings.Join(conditions, " AND "), args
}

func (r *connectionRecordRepo) List(ctx context.Context, teamID uuid.UUID, filter ConnFilter) ([]*model.ConnectionRecord, error) {
	where, args := r.buildWhereClause(teamID, filter)

	limit := filter.Limit
	if limit <= 0 {
		limit = 50
	}
	offset := filter.Offset
	if offset < 0 {
		offset = 0
	}
	args = append(args, limit, offset)

	query := `SELECT id, team_id, controller_code, controlled_code, path,
	                   started_at, ended_at, duration_sec, bytes_transferred,
	                   recording_path, created_at
	            FROM connection_records
	            WHERE ` + where +
		fmt.Sprintf(` ORDER BY started_at DESC LIMIT $%d OFFSET $%d`, len(args)-1, len(args))

	var records []*model.ConnectionRecord
	err := r.db.SelectContext(ctx, &records, query, args...)
	if err != nil {
		return nil, fmt.Errorf("list connection records: %w", err)
	}
	return records, nil
}

func (r *connectionRecordRepo) Count(ctx context.Context, teamID uuid.UUID, filter ConnFilter) (int, error) {
	where, args := r.buildWhereClause(teamID, filter)

	query := `SELECT COUNT(*) FROM connection_records WHERE ` + where

	var count int
	err := r.db.GetContext(ctx, &count, query, args...)
	if err != nil {
		return 0, fmt.Errorf("count connection records: %w", err)
	}
	return count, nil
}

func (r *connectionRecordRepo) Create(ctx context.Context, record *model.ConnectionRecord) error {
	err := r.db.QueryRowxContext(ctx,
		`INSERT INTO connection_records (team_id, controller_code, controlled_code, path,
		                                 started_at, ended_at, duration_sec, bytes_transferred,
		                                 recording_path)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
		 RETURNING id, created_at`,
		record.TeamID, record.ControllerCode, record.ControlledCode, record.Path,
		record.StartedAt, record.EndedAt, record.DurationSec, record.BytesTransferred,
		record.RecordingPath,
	).Scan(&record.ID, &record.CreatedAt)
	if err != nil {
		return fmt.Errorf("create connection record: %w", err)
	}
	return nil
}

func (r *connectionRecordRepo) ExportCSV(ctx context.Context, teamID uuid.UUID, filter ConnFilter, w io.Writer) error {
	where, args := r.buildWhereClause(teamID, filter)

	query := `SELECT id, team_id, controller_code, controlled_code, path,
	                   started_at, ended_at, duration_sec, bytes_transferred,
	                   recording_path, created_at
	            FROM connection_records
	            WHERE ` + where + ` ORDER BY started_at DESC`

	rows, err := r.db.QueryxContext(ctx, query, args...)
	if err != nil {
		return fmt.Errorf("export csv query: %w", err)
	}
	defer rows.Close()

	cw := csv.NewWriter(w)
	defer cw.Flush()

	// Write header row.
	header := []string{
		"id", "team_id", "controller_code", "controlled_code", "path",
		"started_at", "ended_at", "duration_sec", "bytes_transferred",
		"recording_path", "created_at",
	}
	if err := cw.Write(header); err != nil {
		return fmt.Errorf("write csv header: %w", err)
	}

	for rows.Next() {
		var rec model.ConnectionRecord
		if err := rows.StructScan(&rec); err != nil {
			return fmt.Errorf("scan connection record: %w", err)
		}

		endedAt := ""
		if rec.EndedAt != nil {
			endedAt = rec.EndedAt.Format(time.RFC3339)
		}
		durationSec := ""
		if rec.DurationSec != nil {
			durationSec = fmt.Sprintf("%d", *rec.DurationSec)
		}
		recordingPath := ""
		if rec.RecordingPath != nil {
			recordingPath = *rec.RecordingPath
		}

		record := []string{
			rec.ID.String(),
			rec.TeamID.String(),
			rec.ControllerCode,
			rec.ControlledCode,
			rec.Path,
			rec.StartedAt.Format(time.RFC3339),
			endedAt,
			durationSec,
			fmt.Sprintf("%d", rec.BytesTransferred),
			recordingPath,
			rec.CreatedAt.Format(time.RFC3339),
		}
		if err := cw.Write(record); err != nil {
			return fmt.Errorf("write csv record: %w", err)
		}
	}

	if err := rows.Err(); err != nil {
		return fmt.Errorf("iterate csv rows: %w", err)
	}
	return nil
}
