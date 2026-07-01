// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithRouter } from '@/test/utils'
import { useAuthStore } from '@/stores/authStore'
import DevicesPage from '../DevicesPage'

vi.mock('@/api/client', () => ({
  apiClient: {
    get: vi.fn(),
    post: vi.fn(),
  },
}))

import { apiClient } from '@/api/client'

const mockDevices = [
  {
    device_code: 'DEV-001',
    device_name: 'MacBook Pro',
    device_type: 'desktop',
    os: 'macOS 14',
    version: '0.1.0',
    status: 'online',
    last_seen: Math.floor(Date.now() / 1000),
    register_time: Math.floor(Date.now() / 1000) - 86400,
    ip_address: '192.168.1.100',
    in_session: false,
  },
  {
    device_code: 'DEV-002',
    device_name: 'Windows PC',
    device_type: 'desktop',
    os: 'Windows 11',
    version: '0.1.0',
    status: 'offline',
    last_seen: Math.floor(Date.now() / 1000) - 3600,
    register_time: Math.floor(Date.now() / 1000) - 172800,
    in_session: false,
  },
]

describe('DevicesPage', () => {
  beforeEach(() => {
    useAuthStore.setState({ isAuthenticated: true })
    vi.mocked(apiClient.get).mockResolvedValue({ data: mockDevices })
    vi.mocked(apiClient.post).mockResolvedValue({ data: {} })
    vi.spyOn(window, 'alert').mockImplementation(() => {})
  })

  it('renders page heading', () => {
    renderWithRouter(<DevicesPage />, { route: '/devices' })
    expect(screen.getByText('Device Management')).toBeInTheDocument()
  })

  it('shows device list after data loads', async () => {
    renderWithRouter(<DevicesPage />, { route: '/devices' })

    await waitFor(() => {
      expect(screen.getByText('MacBook Pro')).toBeInTheDocument()
      expect(screen.getByText('Windows PC')).toBeInTheDocument()
    })
  })

  it('filters devices by search query', async () => {
    const user = userEvent.setup()
    renderWithRouter(<DevicesPage />, { route: '/devices' })

    await waitFor(() => screen.getByText('MacBook Pro'))

    await user.type(screen.getByPlaceholderText(/Search/i), 'MacBook')

    expect(screen.getByText('MacBook Pro')).toBeInTheDocument()
    expect(screen.queryByText('Windows PC')).not.toBeInTheDocument()
  })

  it('calls GET /api/devices on mount', async () => {
    renderWithRouter(<DevicesPage />, { route: '/devices' })

    await waitFor(() => {
      expect(apiClient.get).toHaveBeenCalledWith(expect.stringContaining('/api/devices'))
    })
  })
})
