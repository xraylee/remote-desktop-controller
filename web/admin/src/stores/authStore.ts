// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { create } from 'zustand'
import { setAccessToken } from '@/api/client'
import { loginRequest, logoutRequest, type MemberDto } from '@/api/auth'

// ---------- Public types ----------

export interface Member {
  id: string
  email: string
  name: string
  role: 'super_admin' | 'admin' | 'operator' | 'viewer'
}

export interface AuthState {
  accessToken: string | null
  refreshToken: string | null
  member: Member | null
  isAuthenticated: boolean

  // Actions
  login: (email: string, password: string, totpCode?: string) => Promise<void>
  logout: () => void
  restoreSession: () => void
}

// ---------- Helpers ----------

function dtoToMember(dto: MemberDto): Member {
  return { id: dto.id, email: dto.email, name: dto.name, role: dto.role }
}

function loadRefreshToken(): string | null {
  try {
    return localStorage.getItem('rdcs_refresh_token')
  } catch {
    return null
  }
}

function saveRefreshToken(token: string | null): void {
  try {
    if (token) {
      localStorage.setItem('rdcs_refresh_token', token)
    } else {
      localStorage.removeItem('rdcs_refresh_token')
    }
  } catch {
    // localStorage may be unavailable in some environments
  }
}

// ---------- Store ----------

export const useAuthStore = create<AuthState>()((set, get) => ({
  accessToken: null,
  refreshToken: loadRefreshToken(),
  member: null,
  isAuthenticated: false,

  login: async (email: string, password: string, totpCode?: string) => {
    const res = await loginRequest(email, password, totpCode)

    // Sync token with the axios interceptor
    setAccessToken(res.access_token)
    saveRefreshToken(res.refresh_token)

    set({
      accessToken: res.access_token,
      refreshToken: res.refresh_token,
      member: dtoToMember(res.member),
      isAuthenticated: true,
    })
  },

  logout: () => {
    const { refreshToken } = get()

    // Best-effort server-side logout
    if (refreshToken) {
      logoutRequest(refreshToken).catch(() => {
        // Ignore errors during logout
      })
    }

    setAccessToken(null)
    saveRefreshToken(null)

    set({
      accessToken: null,
      refreshToken: null,
      member: null,
      isAuthenticated: false,
    })
  },

  restoreSession: () => {
    // On app startup, if we have a stored refresh token we can mark the
    // session as needing restoration. The actual token refresh is handled
    // by the app's root component or a dedicated hook.
    const stored = loadRefreshToken()
    if (stored) {
      set({ refreshToken: stored })
    }
  },
}))
