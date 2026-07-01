// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { useQuery } from '@tanstack/react-query'
import { apiClient } from '@/api/client'
import { useState } from 'react'

interface ConnectionRecord {
  id: string
  session_id: string
  controller_device_code: string
  controller_device_name: string
  controlled_device_code: string
  controlled_device_name: string
  start_time: number
  end_time?: number
  duration_seconds?: number
  status: 'active' | 'completed' | 'failed'
  disconnect_reason?: string
  user_name?: string
}

export default function ConnectionRecordsPage() {
  const [page, setPage] = useState(1)
  const [timeRange, setTimeRange] = useState<'today' | 'week' | 'month' | 'all'>('week')
  const [searchQuery, setSearchQuery] = useState('')
  const pageSize = 20

  const { data, isLoading } = useQuery({
    queryKey: ['connection-records', page, timeRange, searchQuery],
    queryFn: async () => {
      const params = new URLSearchParams({
        page: page.toString(),
        page_size: pageSize.toString(),
        time_range: timeRange,
      })
      if (searchQuery) {
        params.append('search', searchQuery)
      }
      const res = await apiClient.get(`/api/records?${params}`)
      return res.data as {
        records: ConnectionRecord[]
        total: number
        page: number
        page_size: number
      }
    },
  })

  const handleExportCSV = async () => {
    try {
      const params = new URLSearchParams({ time_range: timeRange })
      if (searchQuery) params.append('search', searchQuery)

      const res = await apiClient.get(`/api/records/export?${params}`, {
        responseType: 'blob',
      })

      const blob = new Blob([res.data], { type: 'text/csv' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `connection_records_${new Date().toISOString().split('T')[0]}.csv`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
    } catch (error) {
      console.error('Export failed:', error)
      alert('Export failed, please try again')
    }
  }

  const formatDuration = (seconds?: number) => {
    if (!seconds) return '—'
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = seconds % 60
    if (h > 0) return `${h}h ${m}m`
    if (m > 0) return `${m}m ${s}s`
    return `${s}s`
  }

  const formatTimestamp = (ts: number) => {
    return new Date(ts * 1000).toLocaleString('en-US')
  }

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-700">Active</span>
      case 'completed':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-700">Completed</span>
      case 'failed':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-700">Failed</span>
      default:
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-500">Unknown</span>
    }
  }

  const totalPages = data ? Math.ceil(data.total / pageSize) : 0

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">Connection Records</h1>
        <button
          onClick={handleExportCSV}
          className="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-700 transition-colors"
        >
          Export CSV
        </button>
      </div>

      {/* Filters */}
      <div className="mt-6 flex flex-col sm:flex-row gap-4">
        <div className="flex-1">
          <input
            type="text"
            placeholder="Search by device name, code or user..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setPage(1)
            }}
            className="w-full rounded-lg border border-gray-300 px-4 py-2 text-sm focus:border-amber-500 focus:outline-none focus:ring-2 focus:ring-amber-200"
          />
        </div>
        <div className="flex gap-2">
          {(['today', 'week', 'month', 'all'] as const).map((range) => (
            <button
              key={range}
              onClick={() => {
                setTimeRange(range)
                setPage(1)
              }}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                timeRange === range
                  ? 'bg-amber-600 text-white'
                  : 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50'
              }`}
            >
              {range === 'today' ? 'Today' : range === 'week' ? 'This Week' : range === 'month' ? 'This Month' : 'All'}
            </button>
          ))}
        </div>
      </div>

      {/* Records table */}
      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Session ID</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Controller</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Controlled</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Start Time</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Duration</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {isLoading ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-sm text-gray-400">
                    加载中...
                  </td>
                </tr>
              ) : data && data.records.length > 0 ? (
                data.records.map((record) => (
                  <tr key={record.id} className="hover:bg-gray-50 transition-colors">
                    <td className="px-6 py-4 text-sm font-mono text-gray-900">
                      {record.session_id.slice(0, 8)}...
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm">
                        <div className="font-medium text-gray-900">{record.controller_device_name}</div>
                        <div className="text-gray-500 font-mono text-xs">{record.controller_device_code}</div>
                        {record.user_name && (
                          <div className="text-gray-500 text-xs mt-0.5">👤 {record.user_name}</div>
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm">
                        <div className="font-medium text-gray-900">{record.controlled_device_name}</div>
                        <div className="text-gray-500 font-mono text-xs">{record.controlled_device_code}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-500 whitespace-nowrap">
                      {formatTimestamp(record.start_time)}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900 font-medium">
                      {formatDuration(record.duration_seconds)}
                    </td>
                    <td className="px-6 py-4">
                      {getStatusBadge(record.status)}
                      {record.disconnect_reason && (
                        <div className="text-xs text-gray-500 mt-1">{record.disconnect_reason}</div>
                      )}
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-sm text-gray-400">
                    {searchQuery ? '未找到匹配的记录' : '暂无连接记录'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Pagination */}
      {data && data.total > pageSize && (
        <div className="mt-6 flex items-center justify-between">
          <p className="text-sm text-gray-700">
            显示 <span className="font-medium">{(page - 1) * pageSize + 1}</span> 到{' '}
            <span className="font-medium">{Math.min(page * pageSize, data.total)}</span>，共{' '}
            <span className="font-medium">{data.total}</span> 条记录
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => setPage(p => Math.max(1, p - 1))}
              disabled={page === 1}
              className="px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              上一页
            </button>
            <span className="px-3 py-1.5 text-sm text-gray-700">
              {page} / {totalPages}
            </span>
            <button
              onClick={() => setPage(p => Math.min(totalPages, p + 1))}
              disabled={page === totalPages}
              className="px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              下一页
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
