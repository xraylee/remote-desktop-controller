// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithRouter } from '@/test/utils'
import { useAuthStore } from '@/stores/authStore'
import LoginPage from '../LoginPage'

// Mock the API layer so the real store can run its logic
vi.mock('@/api/auth', () => ({
  loginRequest: vi.fn(),
  logoutRequest: vi.fn().mockResolvedValue(undefined),
}))

import { loginRequest } from '@/api/auth'

const mockMember = { id: '1', email: 'admin@test.com', name: 'Admin', role: 'admin' as const }

describe('LoginPage', () => {
  beforeEach(() => {
    useAuthStore.setState({ isAuthenticated: false, accessToken: null, refreshToken: null, member: null })
    vi.mocked(loginRequest).mockReset()
  })

  it('renders email, password fields and submit button', () => {
    renderWithRouter(<LoginPage />, { route: '/login' })

    expect(screen.getByLabelText('Email')).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeInTheDocument()
  })

  it('calls loginRequest with email and password on submit', async () => {
    const user = userEvent.setup()
    vi.mocked(loginRequest).mockResolvedValue({
      access_token: 'tok', refresh_token: 'ref', expires_in: 3600, member: mockMember,
    })

    renderWithRouter(<LoginPage />, { route: '/login' })

    await user.type(screen.getByLabelText('Email'), 'admin@test.com')
    await user.type(screen.getByLabelText('Password'), 'secret')
    await user.click(screen.getByRole('button', { name: 'Sign in' }))

    await waitFor(() => {
      expect(loginRequest).toHaveBeenCalledWith('admin@test.com', 'secret', undefined)
    })
  })

  it('shows loading state while submitting', async () => {
    const user = userEvent.setup()
    vi.mocked(loginRequest).mockImplementation(() => new Promise(() => {})) // never resolves

    renderWithRouter(<LoginPage />, { route: '/login' })

    await user.type(screen.getByLabelText('Email'), 'admin@test.com')
    await user.type(screen.getByLabelText('Password'), 'password')
    await user.click(screen.getByRole('button', { name: 'Sign in' }))

    expect(screen.getByRole('button', { name: 'Signing in...' })).toBeDisabled()
  })

  it('displays error message on login failure', async () => {
    const user = userEvent.setup()
    vi.mocked(loginRequest).mockRejectedValue(new Error('Invalid email or password'))

    renderWithRouter(<LoginPage />, { route: '/login' })

    await user.type(screen.getByLabelText('Email'), 'bad@test.com')
    await user.type(screen.getByLabelText('Password'), 'wrong')
    await user.click(screen.getByRole('button', { name: 'Sign in' }))

    await waitFor(() => {
      expect(screen.getByText('Invalid email or password')).toBeInTheDocument()
    })
    // Button re-enabled after failure
    expect(screen.getByRole('button', { name: 'Sign in' })).not.toBeDisabled()
  })

  it('shows TOTP field after clicking "Enable Two-Factor Authentication"', async () => {
    const user = userEvent.setup()
    renderWithRouter(<LoginPage />, { route: '/login' })

    expect(screen.queryByLabelText('Two-Factor Authentication Code')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Enable Two-Factor Authentication' }))
    expect(screen.getByLabelText('Two-Factor Authentication Code')).toBeInTheDocument()
  })
})
