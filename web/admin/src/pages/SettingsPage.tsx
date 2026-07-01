// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

export default function SettingsPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
      <p className="mt-2 text-gray-600">Configure system parameters and security policies (coming soon)</p>

      <div className="mt-6 space-y-4">
        {['General', 'Security', 'Notifications', 'License'].map((section) => (
          <div key={section} className="rounded-xl border border-gray-200 bg-white p-6">
            <h2 className="text-base font-medium text-gray-900">{section}</h2>
            <p className="mt-1 text-sm text-gray-400">This feature is under development</p>
          </div>
        ))}
      </div>
    </div>
  )
}
