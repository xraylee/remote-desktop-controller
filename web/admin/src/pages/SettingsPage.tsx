// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

export default function SettingsPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900">系统设置</h1>
      <p className="mt-2 text-gray-600">配置系统参数和安全策略（开发中）</p>

      {/* Placeholder sections */}
      <div className="mt-6 space-y-4">
        {['基本设置', '安全策略', '通知配置', '许可证'].map((section) => (
          <div
            key={section}
            className="rounded-xl border border-gray-200 bg-white p-6"
          >
            <h2 className="text-base font-medium text-gray-900">{section}</h2>
            <p className="mt-1 text-sm text-gray-400">此功能尚在开发中</p>
          </div>
        ))}
      </div>
    </div>
  )
}
