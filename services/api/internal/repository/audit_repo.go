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
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jmoiron/sqlx"

	"github.com/rdcs/rdcs-api/internal/model"
)

// AuditFilter specifies optional filters for listing audit logs.
type AuditFilter struct {
	ActorID   *uuid.UUID
	Action    string
	StartTime *time.Time
	EndTime   *time.Time
	Limit     int
	Offset    int
}

// AuditLogRepository defines operations on the audit_logs table.
type AuditLogRepository interface {
	List(ctx context.Context, teamID uuid.UUID, filter AuditFilter) ([]*model.AuditLog, error)
	Count(ctx context.Context, teamID uuid.UUID, filter AuditFilter) (int, error)
	Create(ctx context.Context, log *model.AuditLog) error
}

type auditLogRepo struct {
	db *sqlx.DB
}

// NewAuditLogRepository creates a new AuditLogRepository backed by sqlx.
func NewAuditLogRepository(db *sqlx.DB) AuditLogRepository {
	return &auditLogRepo{db: db}
}

func (r *auditLogRepo) List(ctx context.Context, teamID uuid.UUID, filter AuditFilter) ([]*model.AuditLog, error) {
	var args []interface{}
	var conditions []string

	args = append(args, teamID)
	conditions = append(conditions, fmt.Sprintf("team_id = $%d", len(args)))

	if filter.ActorID != nil {
		args = append(args, *filter.ActorID)
		conditions = append(conditions, fmt.Sprintf("actor_id = $%d", len(args)))
	}
	if filter.Action != "" {
		args = append(args, filter.Action)
		conditions = append(conditions, fmt.Sprintf("action = $%d", len(args)))
	}
	if filter.StartTime != nil {
		args = append(args, *filter.StartTime)
		conditions = append(conditions, fmt.Sprintf("created_at >= $%d", len(args)))
	}
	if filter.EndTime != nil {
		args = append(args, *filter.EndTime)
		conditions = append(conditions, fmt.Sprintf("created_at <= $%d", len(args)))
	}

	limit := filter.Limit
	if limit <= 0 {
		limit = 50
	}
	offset := filter.Offset
	if offset < 0 {
		offset = 0
	}
	args = append(args, limit, offset)

	query := `SELECT id, team_id, actor_id, action, target_type, target_id,
	                   details, ip_address, created_at
	            FROM audit_logs
	            WHERE ` + strings.Join(conditions, " AND ") +
		fmt.Sprintf(` ORDER BY created_at DESC LIMIT $%d OFFSET $%d`, len(args)-1, len(args))

	var logs []*model.AuditLog
	err := r.db.SelectContext(ctx, &logs, query, args...)
	if err != nil {
		return nil, fmt.Errorf("list audit logs: %w", err)
	}
	return logs, nil
}

func (r *auditLogRepo) Count(ctx context.Context, teamID uuid.UUID, filter AuditFilter) (int, error) {
	var args []interface{}
	var conditions []string

	args = append(args, teamID)
	conditions = append(conditions, fmt.Sprintf("team_id = $%d", len(args)))

	if filter.ActorID != nil {
		args = append(args, *filter.ActorID)
		conditions = append(conditions, fmt.Sprintf("actor_id = $%d", len(args)))
	}
	if filter.Action != "" {
		args = append(args, filter.Action)
		conditions = append(conditions, fmt.Sprintf("action = $%d", len(args)))
	}
	if filter.StartTime != nil {
		args = append(args, *filter.StartTime)
		conditions = append(conditions, fmt.Sprintf("created_at >= $%d", len(args)))
	}
	if filter.EndTime != nil {
		args = append(args, *filter.EndTime)
		conditions = append(conditions, fmt.Sprintf("created_at <= $%d", len(args)))
	}

	query := `SELECT COUNT(*) FROM audit_logs WHERE ` + strings.Join(conditions, " AND ")

	var count int
	err := r.db.GetContext(ctx, &count, query, args...)
	if err != nil {
		return 0, fmt.Errorf("count audit logs: %w", err)
	}
	return count, nil
}

func (r *auditLogRepo) Create(ctx context.Context, log *model.AuditLog) error {
	err := r.db.QueryRowxContext(ctx,
		`INSERT INTO audit_logs (team_id, actor_id, action, target_type, target_id,
		                         details, ip_address)
		 VALUES ($1, $2, $3, $4, $5, $6, $7)
		 RETURNING id, created_at`,
		log.TeamID, log.ActorID, log.Action, log.TargetType, log.TargetID,
		log.Details, log.IPAddress,
	).Scan(&log.ID, &log.CreatedAt)
	if err != nil {
		return fmt.Errorf("create audit log: %w", err)
	}
	return nil
}
