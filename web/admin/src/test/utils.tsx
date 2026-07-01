// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { render, type RenderOptions } from '@testing-library/react'
import { BrowserRouter, MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
}

export function renderWithRouter(ui: ReactNode, { route = '/' } = {}) {
  return render(
    <MemoryRouter initialEntries={[route]}>
      <QueryClientProvider client={makeQueryClient()}>{ui}</QueryClientProvider>
    </MemoryRouter>,
  )
}

export function renderWithProviders(ui: ReactNode, options?: RenderOptions) {
  return render(
    <BrowserRouter>
      <QueryClientProvider client={makeQueryClient()}>{ui}</QueryClientProvider>
    </BrowserRouter>,
    options,
  )
}
