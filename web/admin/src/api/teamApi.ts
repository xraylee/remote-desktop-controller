// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { useAuthStore } from '@/stores/authStore'

/**
 * Get the current user's team ID from the auth store.
 * Throws an error if the user is not authenticated or team_id is missing.
 */
export function getCurrentTeamId(): string {
  const member = useAuthStore.getState().member
  if (!member || !member.team_id) {
    throw new Error('User not authenticated or team_id missing')
  }
  return member.team_id
}

/**
 * Build a team-scoped API path.
 * Example: buildTeamPath('/dashboard/stats') -> '/teams/{teamID}/dashboard/stats'
 */
export function buildTeamPath(path: string): string {
  const teamId = getCurrentTeamId()
  // Remove leading slash if present
  const cleanPath = path.startsWith('/') ? path.slice(1) : path
  return `/teams/${teamId}/${cleanPath}`
}
