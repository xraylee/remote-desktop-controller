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

package model

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
)

// Team represents an organization/tenant in the system.
type Team struct {
	ID            uuid.UUID `db:"id"`
	Name          string    `db:"name"`
	Plan          string    `db:"plan"`
	MaxConcurrent int       `db:"max_concurrent"`
	CreatedAt     time.Time `db:"created_at"`
	UpdatedAt     time.Time `db:"updated_at"`
}

// Member represents an admin or operator account within a team.
type Member struct {
	ID           uuid.UUID  `db:"id"`
	TeamID       uuid.UUID  `db:"team_id"`
	Name         string     `db:"name"`
	Email        string     `db:"email"`
	Role         string     `db:"role"`
	PasswordHash string     `db:"password_hash"`
	TotpSecret   *string    `db:"totp_secret"`
	TotpEnabled  bool       `db:"totp_enabled"`
	LastLogin    *time.Time `db:"last_login"`
	CreatedAt    time.Time  `db:"created_at"`
}

// Device represents a remote machine with the RDCS agent installed.
type Device struct {
	ID            uuid.UUID  `db:"id"`
	TeamID        uuid.UUID  `db:"team_id"`
	DeviceCode    string     `db:"device_code"`
	DeviceName    string     `db:"device_name"`
	Platform      string     `db:"platform"`
	OsVersion     *string    `db:"os_version"`
	ClientVersion *string    `db:"client_version"`
	Status        string     `db:"status"`
	LastSeen      *time.Time `db:"last_seen"`
	IPAddress     *string    `db:"ip_address"`
	CreatedAt     time.Time  `db:"created_at"`
}

// ConnectionRecord represents a remote desktop session between two devices.
type ConnectionRecord struct {
	ID               uuid.UUID  `db:"id"`
	TeamID           uuid.UUID  `db:"team_id"`
	ControllerCode   string     `db:"controller_code"`
	ControlledCode   string     `db:"controlled_code"`
	Path             string     `db:"path"`
	StartedAt        time.Time  `db:"started_at"`
	EndedAt          *time.Time `db:"ended_at"`
	DurationSec      *int       `db:"duration_sec"`
	BytesTransferred int64      `db:"bytes_transferred"`
	RecordingPath    *string    `db:"recording_path"`
	CreatedAt        time.Time  `db:"created_at"`
}

// AuditLog represents a record of an administrative action.
type AuditLog struct {
	ID         uuid.UUID       `db:"id"`
	TeamID     uuid.UUID       `db:"team_id"`
	ActorID    *uuid.UUID      `db:"actor_id"`
	Action     string          `db:"action"`
	TargetType *string         `db:"target_type"`
	TargetID   *uuid.UUID      `db:"target_id"`
	Details    json.RawMessage `db:"details"`
	IPAddress  *string         `db:"ip_address"`
	CreatedAt  time.Time       `db:"created_at"`
}

// Recording represents an encrypted session recording stored on disk.
type Recording struct {
	ID            uuid.UUID  `db:"id"`
	ConnectionID  uuid.UUID  `db:"connection_id"`
	TeamID        uuid.UUID  `db:"team_id"`
	EncryptionKey []byte     `db:"encryption_key"`
	StoragePath   string     `db:"storage_path"`
	DurationSec   *int       `db:"duration_sec"`
	FileSize      int64      `db:"file_size"`
	CreatedAt     time.Time  `db:"created_at"`
	ExpiresAt     *time.Time `db:"expires_at"`
}
