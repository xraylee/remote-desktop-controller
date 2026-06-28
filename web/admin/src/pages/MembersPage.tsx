// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

export default function MembersPage() {
  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">成员管理</h1>
      </div>
      <p className="mt-2 text-gray-600">管理组织成员和权限分配（开发中）</p>

      {/* Placeholder table */}
      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-white">
        <div className="px-6 py-12 text-center text-sm text-gray-400">
          暂无成员数据
        </div>
      </div>
    </div>
  )
}
