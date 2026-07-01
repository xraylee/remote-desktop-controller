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

function saveAccessToken(token: string | null): void {
  try {
    if (token) {
      localStorage.setItem('rdcs_access_token', token)
    } else {
      localStorage.removeItem('rdcs_access_token')
    }
  } catch {
    // localStorage may be unavailable in some environments
  }
}

// ---------- Store ----------

export const useAuthStore = create<AuthState>()((set, get) => {
  // Initialize state by checking localStorage synchronously
  const storedRefreshToken = loadRefreshToken()
  const storedAccessToken = storedRefreshToken ? localStorage.getItem('rdcs_access_token') : null

  // If tokens exist, restore session immediately
  if (storedAccessToken && storedRefreshToken) {
    setAccessToken(storedAccessToken)
  }

  return {
    accessToken: storedAccessToken,
    refreshToken: storedRefreshToken,
    member: null,
    isAuthenticated: !!storedRefreshToken,  // Key: restore auth state on init

    login: async (email: string, password: string, totpCode?: string) => {
      const res = await loginRequest(email, password, totpCode)

      // Sync token with the axios interceptor
      setAccessToken(res.access_token)
      saveAccessToken(res.access_token)  // Persist access token
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
      saveAccessToken(null)  // Clear access token
      saveRefreshToken(null)

      set({
        accessToken: null,
        refreshToken: null,
        member: null,
        isAuthenticated: false,
      })
    },

    restoreSession: () => {
      console.log('[authStore] restoreSession() called')

      // On app startup, if we have a stored refresh token, restore the session.
      // We mark isAuthenticated as true so the user can access protected routes.
      const stored = loadRefreshToken()
      console.log('[authStore] stored refresh_token:', stored ? 'exists' : 'null')

      if (stored) {
        // Try to restore access token from localStorage as well
        const storedAccessToken = localStorage.getItem('rdcs_access_token')
        console.log('[authStore] stored access_token:', storedAccessToken ? 'exists' : 'null')

        if (storedAccessToken) {
          setAccessToken(storedAccessToken)
        }

        set({
          accessToken: storedAccessToken,
          refreshToken: stored,
          isAuthenticated: true  // Key fix: restore authenticated state
        })

        console.log('[authStore] Session restored, isAuthenticated = true')
        // Note: if access token is expired, API calls will get 401 and redirect to login
      } else {
        console.log('[authStore] No refresh token found, session NOT restored')
      }
    },
  }
})
