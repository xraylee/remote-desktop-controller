// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'

export default function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const location = useLocation()

  if (!isAuthenticated) {
    // Redirect to login, preserving the page they tried to visit
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  return <>{children}</>
}
