// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

export default function SessionsPage() {
  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">Sessions</h1>
      </div>
      <p className="mt-2 text-gray-600">View remote connection history and audit recordings (coming soon)</p>

      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-white">
        <div className="px-6 py-12 text-center text-sm text-gray-400">
          No session records
        </div>
      </div>
    </div>
  )
}
