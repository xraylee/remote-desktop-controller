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
)

// DeviceRepositoryExtensions adds helper methods for dashboard statistics.
type DeviceRepositoryExtensions interface {
	CountByStatus(ctx context.Context, teamID uuid.UUID, status string) (int, error)
	CountInSession(ctx context.Context, teamID uuid.UUID) (int, error)
}

// CountByStatus returns the number of devices with a specific status.
func (r *deviceRepo) CountByStatus(ctx context.Context, teamID uuid.UUID, status string) (int, error) {
	var count int
	err := r.db.GetContext(ctx, &count,
		`SELECT COUNT(*) FROM devices WHERE team_id = $1 AND status = $2`,
		teamID, status)
	if err != nil {
		return 0, fmt.Errorf("count devices by status: %w", err)
	}
	return count, nil
}

// CountInSession returns the number of devices currently in an active session.
func (r *deviceRepo) CountInSession(ctx context.Context, teamID uuid.UUID) (int, error) {
	var count int
	err := r.db.GetContext(ctx, &count,
		`SELECT COUNT(*) FROM devices WHERE team_id = $1 AND in_session = true`,
		teamID)
	if err != nil {
		return 0, fmt.Errorf("count devices in session: %w", err)
	}
	return count, nil
}
