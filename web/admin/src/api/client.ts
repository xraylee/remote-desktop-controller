// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import axios from 'axios'
import type { AxiosInstance, InternalAxiosRequestConfig, AxiosResponse } from 'axios'

const apiClient: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api/v1',
  timeout: 15_000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Shared token accessor — avoids circular dependency with authStore.
// authStore calls setAccessToken() whenever the token changes.
let _accessToken: string | null = null

export function getAccessToken(): string | null {
  return _accessToken
}

export function setAccessToken(token: string | null): void {
  _accessToken = token
}

// Request interceptor: attach Authorization header
apiClient.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    if (_accessToken && config.headers) {
      config.headers.Authorization = `Bearer ${_accessToken}`
    }
    return config
  },
  (error) => Promise.reject(error),
)

// Response interceptor: handle 401 by clearing auth and redirecting to login
apiClient.interceptors.response.use(
  (response: AxiosResponse) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Clear token and redirect; authStore.logout() will be called by the
      // ProtectedRoute / app-level logic when the user lands on /login.
      _accessToken = null
      window.location.href = '/login'
    }
    return Promise.reject(error)
  },
)

export { apiClient }
export default apiClient
