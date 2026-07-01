// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useAuthStore } from '../authStore'

// Mock the auth API
vi.mock('@/api/auth', () => ({
  loginRequest: vi.fn(),
  logoutRequest: vi.fn().mockResolvedValue(undefined),
}))

import { loginRequest } from '@/api/auth'

const mockLoginResponse = {
  access_token: 'access-token-xyz',
  refresh_token: 'refresh-token-xyz',
  expires_in: 3600,
  member: { id: '1', email: 'admin@test.com', name: 'Admin', role: 'admin' as const },
}

describe('authStore', () => {
  beforeEach(() => {
    useAuthStore.setState({
      accessToken: null,
      refreshToken: null,
      member: null,
      isAuthenticated: false,
    })
  })

  describe('login', () => {
    it('stores tokens and sets isAuthenticated on success', async () => {
      vi.mocked(loginRequest).mockResolvedValue(mockLoginResponse)

      const { result } = renderHook(() => useAuthStore())
      await act(async () => {
        await result.current.login('admin@test.com', 'password')
      })

      expect(result.current.isAuthenticated).toBe(true)
      expect(result.current.accessToken).toBe('access-token-xyz')
      expect(result.current.refreshToken).toBe('refresh-token-xyz')
      expect(result.current.member?.email).toBe('admin@test.com')
    })

    it('persists tokens to localStorage on success', async () => {
      vi.mocked(loginRequest).mockResolvedValue(mockLoginResponse)

      const { result } = renderHook(() => useAuthStore())
      await act(async () => {
        await result.current.login('admin@test.com', 'password')
      })

      expect(localStorage.getItem('rdcs_access_token')).toBe('access-token-xyz')
      expect(localStorage.getItem('rdcs_refresh_token')).toBe('refresh-token-xyz')
    })

    it('throws and keeps isAuthenticated false on failure', async () => {
      vi.mocked(loginRequest).mockRejectedValue(new Error('Invalid credentials'))

      const { result } = renderHook(() => useAuthStore())
      await expect(
        act(async () => { await result.current.login('bad@test.com', 'wrong') })
      ).rejects.toThrow('Invalid credentials')

      expect(result.current.isAuthenticated).toBe(false)
      expect(result.current.accessToken).toBeNull()
    })
  })

  describe('logout', () => {
    it('clears all auth state and localStorage', async () => {
      // Seed authenticated state
      useAuthStore.setState({
        accessToken: 'tok',
        refreshToken: 'ref',
        member: mockLoginResponse.member,
        isAuthenticated: true,
      })
      localStorage.setItem('rdcs_access_token', 'tok')
      localStorage.setItem('rdcs_refresh_token', 'ref')

      const { result } = renderHook(() => useAuthStore())
      act(() => { result.current.logout() })

      expect(result.current.isAuthenticated).toBe(false)
      expect(result.current.accessToken).toBeNull()
      expect(result.current.member).toBeNull()
      expect(localStorage.getItem('rdcs_access_token')).toBeNull()
      expect(localStorage.getItem('rdcs_refresh_token')).toBeNull()
    })
  })

  describe('restoreSession', () => {
    it('restores isAuthenticated when tokens exist in localStorage', () => {
      localStorage.setItem('rdcs_refresh_token', 'stored-refresh')
      localStorage.setItem('rdcs_access_token', 'stored-access')

      const { result } = renderHook(() => useAuthStore())
      act(() => { result.current.restoreSession() })

      expect(result.current.isAuthenticated).toBe(true)
      expect(result.current.accessToken).toBe('stored-access')
    })

    it('leaves isAuthenticated false when no refresh token in localStorage', () => {
      const { result } = renderHook(() => useAuthStore())
      act(() => { result.current.restoreSession() })

      expect(result.current.isAuthenticated).toBe(false)
    })
  })
})
