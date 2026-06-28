// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import apiClient from './client'

// ---------- Response types ----------

export interface LoginResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  member: MemberDto
}

export interface MemberDto {
  id: string
  email: string
  name: string
  role: 'super_admin' | 'admin' | 'operator' | 'viewer'
}

export interface RefreshResponse {
  access_token: string
  refresh_token: string
  expires_in: number
}

// ---------- Raw API functions ----------

export async function loginRequest(
  email: string,
  password: string,
  totpCode?: string,
): Promise<LoginResponse> {
  const { data } = await apiClient.post<LoginResponse>('/auth/login', {
    email,
    password,
    totp_code: totpCode,
  })
  return data
}

export async function logoutRequest(refreshToken: string): Promise<void> {
  await apiClient.post('/auth/logout', { refresh_token: refreshToken })
}

export async function refreshRequest(refreshToken: string): Promise<RefreshResponse> {
  const { data } = await apiClient.post<RefreshResponse>('/auth/refresh', {
    refresh_token: refreshToken,
  })
  return data
}
