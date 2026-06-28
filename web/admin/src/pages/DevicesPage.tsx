// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/api/client'
import { useState } from 'react'

interface Device {
  device_code: string
  device_name: string
  device_type: 'desktop' | 'mobile'
  os: string
  version: string
  status: 'online' | 'offline' | 'disabled'
  last_seen: number
  register_time: number
  ip_address?: string
  in_session: boolean
  session_id?: string
  user_id?: string
  user_name?: string
}

export default function DevicesPage() {
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<'all' | 'online' | 'offline' | 'disabled'>('all')
  const [selectedDevice, setSelectedDevice] = useState<Device | null>(null)
  const queryClient = useQueryClient()

  const { data: devices, isLoading } = useQuery({
    queryKey: ['devices', statusFilter],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (statusFilter !== 'all') {
        params.append('status', statusFilter)
      }
      const res = await apiClient.get(`/api/devices?${params}`)
      return res.data as Device[]
    },
    refetchInterval: 3000,
  })

  const kickDeviceMutation = useMutation({
    mutationFn: async (deviceCode: string) => {
      await apiClient.post(`/api/devices/${deviceCode}/kick`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['devices'] })
      alert('设备已断开连接')
      setSelectedDevice(null)
    },
    onError: () => {
      alert('操作失败，请重试')
    },
  })

  const disableDeviceMutation = useMutation({
    mutationFn: async (deviceCode: string) => {
      await apiClient.post(`/api/devices/${deviceCode}/disable`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['devices'] })
      alert('设备已禁用')
      setSelectedDevice(null)
    },
    onError: () => {
      alert('操作失败，请重试')
    },
  })

  const enableDeviceMutation = useMutation({
    mutationFn: async (deviceCode: string) => {
      await apiClient.post(`/api/devices/${deviceCode}/enable`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['devices'] })
      alert('设备已启用')
      setSelectedDevice(null)
    },
    onError: () => {
      alert('操作失败，请重试')
    },
  })

  const filteredDevices = devices?.filter(device => {
    if (!searchQuery) return true
    const query = searchQuery.toLowerCase()
    return (
      device.device_name.toLowerCase().includes(query) ||
      device.device_code.toLowerCase().includes(query) ||
      device.user_name?.toLowerCase().includes(query) ||
      device.os.toLowerCase().includes(query)
    )
  })

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'online':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-700">在线</span>
      case 'offline':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-700">离线</span>
      case 'disabled':
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-700">已禁用</span>
      default:
        return <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-500">未知</span>
    }
  }

  const getDeviceIcon = (type: string) => {
    return type === 'desktop' ? '🖥️' : '📱'
  }

  const formatTimestamp = (ts: number) => {
    const date = new Date(ts * 1000)
    const now = new Date()
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000)

    if (diff < 60) return `${diff}秒前`
    if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`
    if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`
    return date.toLocaleDateString('zh-CN')
  }

  const stats = {
    total: devices?.length || 0,
    online: devices?.filter(d => d.status === 'online').length || 0,
    offline: devices?.filter(d => d.status === 'offline').length || 0,
    disabled: devices?.filter(d => d.status === 'disabled').length || 0,
    inSession: devices?.filter(d => d.in_session).length || 0,
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">设备管理</h1>
      </div>

      {/* Stats */}
      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <div className="rounded-xl border border-gray-200 bg-white p-4">
          <p className="text-sm text-gray-500">总设备数</p>
          <p className="mt-1 text-2xl font-bold text-gray-900">{stats.total}</p>
        </div>
        <div className="rounded-xl border border-green-200 bg-green-50 p-4">
          <p className="text-sm text-green-700">在线</p>
          <p className="mt-1 text-2xl font-bold text-green-700">{stats.online}</p>
        </div>
        <div className="rounded-xl border border-gray-200 bg-white p-4">
          <p className="text-sm text-gray-500">离线</p>
          <p className="mt-1 text-2xl font-bold text-gray-900">{stats.offline}</p>
        </div>
        <div className="rounded-xl border border-red-200 bg-red-50 p-4">
          <p className="text-sm text-red-700">已禁用</p>
          <p className="mt-1 text-2xl font-bold text-red-700">{stats.disabled}</p>
        </div>
        <div className="rounded-xl border border-amber-200 bg-amber-50 p-4">
          <p className="text-sm text-amber-700">会话中</p>
          <p className="mt-1 text-2xl font-bold text-amber-700">{stats.inSession}</p>
        </div>
      </div>

      {/* Filters */}
      <div className="mt-6 flex flex-col sm:flex-row gap-4">
        <div className="flex-1">
          <input
            type="text"
            placeholder="搜索设备名称、设备码、用户或系统..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full rounded-lg border border-gray-300 px-4 py-2 text-sm focus:border-amber-500 focus:outline-none focus:ring-2 focus:ring-amber-200"
          />
        </div>
        <div className="flex gap-2">
          {(['all', 'online', 'offline', 'disabled'] as const).map((status) => (
            <button
              key={status}
              onClick={() => setStatusFilter(status)}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                statusFilter === status
                  ? 'bg-amber-600 text-white'
                  : 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50'
              }`}
            >
              {status === 'all' ? '全部' : status === 'online' ? '在线' : status === 'offline' ? '离线' : '已禁用'}
            </button>
          ))}
        </div>
      </div>

      {/* Devices table */}
      <div className="mt-6 overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  设备
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  设备码
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  系统
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  用户
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  最后在线
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {isLoading ? (
                <tr>
                  <td colSpan={7} className="px-6 py-12 text-center text-sm text-gray-400">
                    加载中...
                  </td>
                </tr>
              ) : filteredDevices && filteredDevices.length > 0 ? (
                filteredDevices.map((device) => (
                  <tr key={device.device_code} className="hover:bg-gray-50 transition-colors">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl">{getDeviceIcon(device.device_type)}</span>
                        <div>
                          <div className="text-sm font-medium text-gray-900">{device.device_name}</div>
                          {device.in_session && (
                            <div className="text-xs text-amber-600 mt-0.5">🔗 会话中</div>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm font-mono text-gray-900">
                      {device.device_code}
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-gray-900">{device.os}</div>
                      <div className="text-xs text-gray-500">{device.version}</div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900">
                      {device.user_name || '—'}
                    </td>
                    <td className="px-6 py-4">
                      {getStatusBadge(device.status)}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-500">
                      {formatTimestamp(device.last_seen)}
                    </td>
                    <td className="px-6 py-4">
                      <button
                        onClick={() => setSelectedDevice(device)}
                        className="text-sm text-amber-600 hover:text-amber-700 font-medium"
                      >
                        详情
                      </button>
                    </td>
                  </tr>
                ))
              ) : (
                <tr>
                  <td colSpan={7} className="px-6 py-12 text-center text-sm text-gray-400">
                    {searchQuery ? '未找到匹配的设备' : '暂无设备数据'}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Device detail modal */}
      {selectedDevice && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" onClick={() => setSelectedDevice(null)}>
          <div className="bg-white rounded-xl p-6 max-w-lg w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-gray-900">设备详情</h2>
              <button
                onClick={() => setSelectedDevice(null)}
                className="text-gray-400 hover:text-gray-600"
              >
                ✕
              </button>
            </div>

            <div className="space-y-3 text-sm">
              <div>
                <span className="text-gray-500">设备名称：</span>
                <span className="font-medium text-gray-900">{selectedDevice.device_name}</span>
              </div>
              <div>
                <span className="text-gray-500">设备码：</span>
                <span className="font-mono text-gray-900">{selectedDevice.device_code}</span>
              </div>
              <div>
                <span className="text-gray-500">类型：</span>
                <span className="text-gray-900">{getDeviceIcon(selectedDevice.device_type)} {selectedDevice.device_type}</span>
              </div>
              <div>
                <span className="text-gray-500">系统：</span>
                <span className="text-gray-900">{selectedDevice.os} {selectedDevice.version}</span>
              </div>
              <div>
                <span className="text-gray-500">状态：</span>
                {getStatusBadge(selectedDevice.status)}
              </div>
              {selectedDevice.user_name && (
                <div>
                  <span className="text-gray-500">用户：</span>
                  <span className="text-gray-900">{selectedDevice.user_name}</span>
                </div>
              )}
              {selectedDevice.ip_address && (
                <div>
                  <span className="text-gray-500">IP 地址：</span>
                  <span className="font-mono text-gray-900">{selectedDevice.ip_address}</span>
                </div>
              )}
              <div>
                <span className="text-gray-500">注册时间：</span>
                <span className="text-gray-900">{new Date(selectedDevice.register_time * 1000).toLocaleString('zh-CN')}</span>
              </div>
              <div>
                <span className="text-gray-500">最后在线：</span>
                <span className="text-gray-900">{formatTimestamp(selectedDevice.last_seen)}</span>
              </div>
              {selectedDevice.in_session && selectedDevice.session_id && (
                <div>
                  <span className="text-gray-500">当前会话：</span>
                  <span className="font-mono text-gray-900">{selectedDevice.session_id.slice(0, 8)}...</span>
                </div>
              )}
            </div>

            <div className="mt-6 flex gap-3">
              {selectedDevice.status === 'online' && selectedDevice.in_session && (
                <button
                  onClick={() => kickDeviceMutation.mutate(selectedDevice.device_code)}
                  disabled={kickDeviceMutation.isPending}
                  className="flex-1 px-4 py-2 text-sm font-medium rounded-lg bg-red-600 text-white hover:bg-red-700 disabled:opacity-50 transition-colors"
                >
                  {kickDeviceMutation.isPending ? '处理中...' : '断开连接'}
                </button>
              )}
              {selectedDevice.status !== 'disabled' ? (
                <button
                  onClick={() => disableDeviceMutation.mutate(selectedDevice.device_code)}
                  disabled={disableDeviceMutation.isPending}
                  className="flex-1 px-4 py-2 text-sm font-medium rounded-lg bg-gray-600 text-white hover:bg-gray-700 disabled:opacity-50 transition-colors"
                >
                  {disableDeviceMutation.isPending ? '处理中...' : '禁用设备'}
                </button>
              ) : (
                <button
                  onClick={() => enableDeviceMutation.mutate(selectedDevice.device_code)}
                  disabled={enableDeviceMutation.isPending}
                  className="flex-1 px-4 py-2 text-sm font-medium rounded-lg bg-green-600 text-white hover:bg-green-700 disabled:opacity-50 transition-colors"
                >
                  {enableDeviceMutation.isPending ? '处理中...' : '启用设备'}
                </button>
              )}
              <button
                onClick={() => setSelectedDevice(null)}
                className="flex-1 px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 transition-colors"
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
