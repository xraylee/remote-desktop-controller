// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { useQuery } from '@tanstack/react-query'
import { apiClient } from '@/api/client'
import { useEffect, useState } from 'react'

interface DashboardStats {
  online_devices: number
  active_sessions: number
  total_members: number
  today_connections: number
}

interface ConnectionTrend {
  date: string
  count: number
}

interface RecentActivity {
  id: string
  type: 'connection' | 'disconnection' | 'device_register'
  device_code: string
  device_name: string
  timestamp: number
  user_name?: string
}

export default function DashboardPage() {
  const [currentTime, setCurrentTime] = useState(new Date())

  useEffect(() => {
    const timer = setInterval(() => setCurrentTime(new Date()), 1000)
    return () => clearInterval(timer)
  }, [])

  const { data: stats, isLoading: statsLoading } = useQuery<DashboardStats>({
    queryKey: ['dashboard-stats'],
    queryFn: async () => {
      const res = await apiClient.get('/api/dashboard/stats')
      return res.data
    },
    refetchInterval: 5000,
  })

  const { data: trends } = useQuery<ConnectionTrend[]>({
    queryKey: ['connection-trends'],
    queryFn: async () => {
      const res = await apiClient.get('/api/dashboard/trends?days=7')
      return res.data
    },
    refetchInterval: 30000,
  })

  const { data: activities } = useQuery<RecentActivity[]>({
    queryKey: ['recent-activities'],
    queryFn: async () => {
      const res = await apiClient.get('/api/dashboard/activities?limit=10')
      return res.data
    },
    refetchInterval: 3000,
  })

  const statCards = [
    { label: 'Online Devices', value: stats?.online_devices ?? 0, color: 'text-amber-600' },
    { label: 'Active Sessions', value: stats?.active_sessions ?? 0, color: 'text-green-600' },
    { label: 'Total Members', value: stats?.total_members ?? 0, color: 'text-blue-600' },
    { label: 'Today Connections', value: stats?.today_connections ?? 0, color: 'text-purple-600' },
  ]

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'connection':
        return '🔗'
      case 'disconnection':
        return '❌'
      case 'device_register':
        return '📱'
      default:
        return '•'
    }
  }

  const getActivityText = (activity: RecentActivity) => {
    switch (activity.type) {
      case 'connection':
        return `${activity.user_name || 'User'} connected to ${activity.device_name}`
      case 'disconnection':
        return `${activity.device_name} disconnected`
      case 'device_register':
        return `New device registered: ${activity.device_name}`
      default:
        return 'Unknown activity'
    }
  }

  const formatTimestamp = (ts: number) => {
    const date = new Date(ts * 1000)
    const now = new Date()
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000)

    if (diff < 60) return `${diff}s ago`
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    return date.toLocaleDateString('en-US')
  }

  const maxTrendValue = trends ? Math.max(...trends.map(t => t.count)) : 0

  return (
    <div>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="mt-1 text-sm text-gray-500">
            {currentTime.toLocaleString('en-US', {
              year: 'numeric',
              month: 'long',
              day: 'numeric',
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit'
            })}
          </p>
        </div>
        <div className="rounded-lg bg-amber-50 px-3 py-1.5 text-sm text-amber-700 border border-amber-200">
          Live Monitoring
        </div>
      </div>

      {/* Stats cards */}
      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {statCards.map(({ label, value, color }) => (
          <div
            key={label}
            className="rounded-xl border border-gray-200 bg-white p-5 shadow-sm hover:shadow-md transition-shadow"
          >
            <p className="text-sm text-gray-500">{label}</p>
            <p className={`mt-2 text-3xl font-bold ${color}`}>
              {statsLoading ? '—' : value.toLocaleString()}
            </p>
          </div>
        ))}
      </div>

      {/* Connection trend chart */}
      <div className="mt-6 rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="text-lg font-semibold text-gray-900">Connection Trends (Last 7 Days)</h2>
        <div className="mt-4 h-48 flex items-end gap-2">
          {trends && trends.length > 0 ? (
            trends.map((trend, idx) => {
              const height = maxTrendValue > 0 ? (trend.count / maxTrendValue) * 100 : 0
              return (
                <div key={idx} className="flex-1 flex flex-col items-center">
                  <div
                    className="w-full bg-amber-500 rounded-t hover:bg-amber-600 transition-colors"
                    style={{ height: `${Math.max(height, 2)}%` }}
                    title={`${trend.count} connections`}
                  />
                  <p className="mt-2 text-xs text-gray-500 rotate-0">
                    {new Date(trend.date).toLocaleDateString('en-US', { month: 'numeric', day: 'numeric' })}
                  </p>
                </div>
              )
            })
          ) : (
            <div className="w-full h-full flex items-center justify-center text-gray-400 text-sm">
              No data available
            </div>
          )}
        </div>
      </div>

      {/* Recent activities */}
      <div className="mt-6 rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="border-b border-gray-200 px-6 py-4">
          <h2 className="text-lg font-semibold text-gray-900">Recent Activities</h2>
        </div>
        <div className="divide-y divide-gray-100">
          {activities && activities.length > 0 ? (
            activities.map((activity) => (
              <div key={activity.id} className="px-6 py-3 hover:bg-gray-50 transition-colors">
                <div className="flex items-center gap-3">
                  <span className="text-lg">{getActivityIcon(activity.type)}</span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-gray-900">{getActivityText(activity)}</p>
                    <p className="text-xs text-gray-500 mt-0.5">
                      Device Code: {activity.device_code}
                    </p>
                  </div>
                  <span className="text-xs text-gray-400 whitespace-nowrap">
                    {formatTimestamp(activity.timestamp)}
                  </span>
                </div>
              </div>
            ))
          ) : (
            <div className="px-6 py-12 text-center text-sm text-gray-400">
              No activity records
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
