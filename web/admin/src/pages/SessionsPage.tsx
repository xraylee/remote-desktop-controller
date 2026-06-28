// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

export default function SessionsPage() {
  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">会话记录</h1>
      </div>
      <p className="mt-2 text-gray-600">查看远程连接历史和审计录像（开发中）</p>

      {/* Placeholder table */}
      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-white">
        <div className="px-6 py-12 text-center text-sm text-gray-400">
          暂无会话记录
        </div>
      </div>
    </div>
  )
}
