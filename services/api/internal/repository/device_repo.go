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

	"github.com/google/uuid"
	"github.com/jmoiron/sqlx"

	"github.com/rdcs/rdcs-api/internal/model"
)

// DeviceFilter specifies optional filters for listing devices.
type DeviceFilter struct {
	Status   string // "online" or "offline"
	Platform string // "windows", "macos", or "linux"
	Search   string // substring match on device_name
	Limit    int
	Offset   int
}

// DeviceRepository defines operations on the devices table.
type DeviceRepository interface {
	GetByID(ctx context.Context, id uuid.UUID) (*model.Device, error)
	GetByCode(ctx context.Context, code string) (*model.Device, error)
	ListByTeam(ctx context.Context, teamID uuid.UUID, filter DeviceFilter) ([]*model.Device, error)
	CountByTeam(ctx context.Context, teamID uuid.UUID, filter DeviceFilter) (int, error)
	Create(ctx context.Context, device *model.Device) error
	Update(ctx context.Context, device *model.Device) error
	Delete(ctx context.Context, id uuid.UUID) error

	// Dashboard extensions
	CountByStatus(ctx context.Context, teamID uuid.UUID, status string) (int, error)
	CountInSession(ctx context.Context, teamID uuid.UUID) (int, error)
}

type deviceRepo struct {
	db *sqlx.DB
}

// NewDeviceRepository creates a new DeviceRepository backed by sqlx.
func NewDeviceRepository(db *sqlx.DB) DeviceRepository {
	return &deviceRepo{db: db}
}

func (r *deviceRepo) GetByID(ctx context.Context, id uuid.UUID) (*model.Device, error) {
	var device model.Device
	err := r.db.GetContext(ctx, &device,
		`SELECT id, team_id, device_code, device_name, platform, os_version,
		        client_version, status, last_seen, ip_address, created_at
		 FROM devices WHERE id = $1`, id)
	if err != nil {
		return nil, fmt.Errorf("get device by id: %w", err)
	}
	return &device, nil
}

func (r *deviceRepo) GetByCode(ctx context.Context, code string) (*model.Device, error) {
	var device model.Device
	err := r.db.GetContext(ctx, &device,
		`SELECT id, team_id, device_code, device_name, platform, os_version,
		        client_version, status, last_seen, ip_address, created_at
		 FROM devices WHERE device_code = $1`, code)
	if err != nil {
		return nil, fmt.Errorf("get device by code: %w", err)
	}
	return &device, nil
}

func (r *deviceRepo) ListByTeam(ctx context.Context, teamID uuid.UUID, filter DeviceFilter) ([]*model.Device, error) {
	var args []interface{}
	var conditions []string

	args = append(args, teamID)
	conditions = append(conditions, fmt.Sprintf("team_id = $%d", len(args)))

	if filter.Status != "" {
		args = append(args, filter.Status)
		conditions = append(conditions, fmt.Sprintf("status = $%d", len(args)))
	}
	if filter.Platform != "" {
		args = append(args, filter.Platform)
		conditions = append(conditions, fmt.Sprintf("platform = $%d", len(args)))
	}
	if filter.Search != "" {
		args = append(args, "%"+filter.Search+"%")
		conditions = append(conditions, fmt.Sprintf("device_name ILIKE $%d", len(args)))
	}

	query := `SELECT id, team_id, device_code, device_name, platform, os_version,
	                   client_version, status, last_seen, ip_address, created_at
	            FROM devices
	            WHERE ` + strings.Join(conditions, " AND ") +
		` ORDER BY created_at DESC`

	limit := filter.Limit
	if limit <= 0 {
		limit = 50
	}
	offset := filter.Offset
	if offset < 0 {
		offset = 0
	}
	args = append(args, limit, offset)
	query += fmt.Sprintf(" LIMIT $%d OFFSET $%d", len(args)-1, len(args))

	var devices []*model.Device
	err := r.db.SelectContext(ctx, &devices, query, args...)
	if err != nil {
		return nil, fmt.Errorf("list devices by team: %w", err)
	}
	return devices, nil
}

func (r *deviceRepo) CountByTeam(ctx context.Context, teamID uuid.UUID, filter DeviceFilter) (int, error) {
	var args []interface{}
	var conditions []string

	args = append(args, teamID)
	conditions = append(conditions, fmt.Sprintf("team_id = $%d", len(args)))

	if filter.Status != "" {
		args = append(args, filter.Status)
		conditions = append(conditions, fmt.Sprintf("status = $%d", len(args)))
	}
	if filter.Platform != "" {
		args = append(args, filter.Platform)
		conditions = append(conditions, fmt.Sprintf("platform = $%d", len(args)))
	}
	if filter.Search != "" {
		args = append(args, "%"+filter.Search+"%")
		conditions = append(conditions, fmt.Sprintf("device_name ILIKE $%d", len(args)))
	}

	query := `SELECT COUNT(*) FROM devices WHERE ` + strings.Join(conditions, " AND ")

	var count int
	err := r.db.GetContext(ctx, &count, query, args...)
	if err != nil {
		return 0, fmt.Errorf("count devices by team: %w", err)
	}
	return count, nil
}

func (r *deviceRepo) Create(ctx context.Context, device *model.Device) error {
	err := r.db.QueryRowxContext(ctx,
		`INSERT INTO devices (team_id, device_code, device_name, platform, os_version,
		                      client_version, status, ip_address)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
		 RETURNING id, created_at`,
		device.TeamID, device.DeviceCode, device.DeviceName, device.Platform,
		device.OsVersion, device.ClientVersion, device.Status, device.IPAddress,
	).Scan(&device.ID, &device.CreatedAt)
	if err != nil {
		return fmt.Errorf("create device: %w", err)
	}
	return nil
}

func (r *deviceRepo) Update(ctx context.Context, device *model.Device) error {
	result, err := r.db.ExecContext(ctx,
		`UPDATE devices
		 SET device_name = $1, platform = $2, os_version = $3, client_version = $4,
		     status = $5, last_seen = $6, ip_address = $7
		 WHERE id = $8`,
		device.DeviceName, device.Platform, device.OsVersion, device.ClientVersion,
		device.Status, device.LastSeen, device.IPAddress, device.ID,
	)
	if err != nil {
		return fmt.Errorf("update device: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("update device rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("update device: not found")
	}
	return nil
}

func (r *deviceRepo) Delete(ctx context.Context, id uuid.UUID) error {
	result, err := r.db.ExecContext(ctx,
		`DELETE FROM devices WHERE id = $1`, id)
	if err != nil {
		return fmt.Errorf("delete device: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("delete device rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("delete device: not found")
	}
	return nil
}
