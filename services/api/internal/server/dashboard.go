// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package server

import (
	"encoding/json"
	"net/http"
	"strconv"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"
	"github.com/rdcs/rdcs-api/internal/repository"
)

// DashboardStats represents real-time dashboard statistics.
type DashboardStats struct {
	OnlineDevices     int `json:"online_devices"`
	ActiveSessions    int `json:"active_sessions"`
	TotalMembers      int `json:"total_members"`
	TodayConnections  int `json:"today_connections"`
}

// ConnectionTrend represents daily connection count trend.
type ConnectionTrend struct {
	Date  string `json:"date"`
	Count int    `json:"count"`
}

// RecentActivity represents a recent activity event.
type RecentActivity struct {
	ID           string `json:"id"`
	Type         string `json:"type"`
	DeviceCode   string `json:"device_code"`
	DeviceName   string `json:"device_name"`
	Timestamp    int64  `json:"timestamp"`
	UserName     string `json:"user_name,omitempty"`
}

// handleGetDashboardStats returns current dashboard statistics.
func (s *Server) handleGetDashboardStats(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	ctx := r.Context()

	// Count online devices
	onlineDevices, err := s.Devices.CountByStatus(ctx, teamID, "online")
	if err != nil {
		http.Error(w, "failed to count online devices", http.StatusInternalServerError)
		return
	}

	// Count active sessions (devices currently in_session)
	activeSessions, err := s.Devices.CountInSession(ctx, teamID)
	if err != nil {
		http.Error(w, "failed to count active sessions", http.StatusInternalServerError)
		return
	}

	// Count total members
	totalMembers, err := s.Members.CountByTeam(ctx, teamID)
	if err != nil {
		http.Error(w, "failed to count members", http.StatusInternalServerError)
		return
	}

	// Count today's connections
	startOfDay := time.Now().Truncate(24 * time.Hour)
	todayConnections, err := s.Connections.Count(ctx, teamID, repository.ConnFilter{
		StartedAfter: &startOfDay,
	})
	if err != nil {
		http.Error(w, "failed to count today connections", http.StatusInternalServerError)
		return
	}

	stats := DashboardStats{
		OnlineDevices:    onlineDevices,
		ActiveSessions:   activeSessions,
		TotalMembers:     totalMembers,
		TodayConnections: todayConnections,
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(stats)
}

// handleGetConnectionTrends returns connection count trends for the past N days.
func (s *Server) handleGetConnectionTrends(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	daysStr := r.URL.Query().Get("days")
	days := 7
	if daysStr != "" {
		if parsed, err := strconv.Atoi(daysStr); err == nil && parsed > 0 && parsed <= 90 {
			days = parsed
		}
	}

	ctx := r.Context()
	trends := make([]ConnectionTrend, 0, days)

	for i := days - 1; i >= 0; i-- {
		day := time.Now().AddDate(0, 0, -i).Truncate(24 * time.Hour)
		nextDay := day.Add(24 * time.Hour)

		count, err := s.Connections.Count(ctx, teamID, repository.ConnFilter{
			StartedAfter:  &day,
			StartedBefore: &nextDay,
		})
		if err != nil {
			http.Error(w, "failed to query connection trends", http.StatusInternalServerError)
			return
		}

		trends = append(trends, ConnectionTrend{
			Date:  day.Format("2006-01-02"),
			Count: count,
		})
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(trends)
}

// handleGetRecentActivities returns recent activity events.
func (s *Server) handleGetRecentActivities(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	limitStr := r.URL.Query().Get("limit")
	limit := 10
	if limitStr != "" {
		if parsed, err := strconv.Atoi(limitStr); err == nil && parsed > 0 && parsed <= 100 {
			limit = parsed
		}
	}

	ctx := r.Context()

	// Get recent audit logs for activity feed
	logs, err := s.AuditLogs.List(ctx, teamID, repository.AuditFilter{
		Limit: limit,
	})
	if err != nil {
		http.Error(w, "failed to list recent activities", http.StatusInternalServerError)
		return
	}

	activities := make([]RecentActivity, 0, len(logs))
	for _, log := range logs {
		activityType := mapActionToActivityType(log.Action)
		if activityType == "" {
			continue // Skip unmapped actions
		}

		activity := RecentActivity{
			ID:         log.ID.String(),
			Type:       activityType,
			Timestamp:  log.CreatedAt.Unix(),
		}

		// Extract device info from details JSON if available
		if log.Details != nil {
			var details map[string]interface{}
			if err := json.Unmarshal(log.Details, &details); err == nil {
				if deviceCode, ok := details["device_code"].(string); ok {
					activity.DeviceCode = deviceCode
				}
				if deviceName, ok := details["device_name"].(string); ok {
					activity.DeviceName = deviceName
				}
				if userName, ok := details["user_name"].(string); ok {
					activity.UserName = userName
				}
			}
		}

		activities = append(activities, activity)
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(activities)
}

// mapActionToActivityType maps audit log actions to dashboard activity types.
func mapActionToActivityType(action string) string {
	switch action {
	case "device.connect", "session.start":
		return "connection"
	case "device.disconnect", "session.end":
		return "disconnection"
	case "device.register":
		return "device_register"
	default:
		return ""
	}
}

// handleListConnectionRecords returns paginated connection records with filters.
func (s *Server) handleListConnectionRecords(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	// Parse query parameters
	page := 1
	if pageStr := r.URL.Query().Get("page"); pageStr != "" {
		if parsed, err := strconv.Atoi(pageStr); err == nil && parsed > 0 {
			page = parsed
		}
	}

	pageSize := 20
	if sizeStr := r.URL.Query().Get("page_size"); sizeStr != "" {
		if parsed, err := strconv.Atoi(sizeStr); err == nil && parsed > 0 && parsed <= 100 {
			pageSize = parsed
		}
	}

	filter := repository.ConnFilter{
		Limit:  pageSize,
		Offset: (page - 1) * pageSize,
	}

	// Time range filter
	timeRange := r.URL.Query().Get("time_range")
	now := time.Now()
	switch timeRange {
	case "today":
		startOfDay := now.Truncate(24 * time.Hour)
		filter.StartedAfter = &startOfDay
	case "week":
		weekAgo := now.AddDate(0, 0, -7)
		filter.StartedAfter = &weekAgo
	case "month":
		monthAgo := now.AddDate(0, -1, 0)
		filter.StartedAfter = &monthAgo
	}

	// Search filter (controller or controlled device code)
	if search := r.URL.Query().Get("search"); search != "" {
		// Note: This is a simple implementation. For proper search across multiple fields,
		// you'd need to enhance the repository layer.
		filter.ControllerCode = search
	}

	ctx := r.Context()

	// Get records
	records, err := s.Connections.List(ctx, teamID, filter)
	if err != nil {
		http.Error(w, "failed to list connection records", http.StatusInternalServerError)
		return
	}

	// Get total count
	total, err := s.Connections.Count(ctx, teamID, filter)
	if err != nil {
		http.Error(w, "failed to count connection records", http.StatusInternalServerError)
		return
	}

	response := map[string]interface{}{
		"records":   records,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(response)
}

// handleExportConnectionRecords exports connection records as CSV.
func (s *Server) handleExportConnectionRecords(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	filter := repository.ConnFilter{}

	// Time range filter
	timeRange := r.URL.Query().Get("time_range")
	now := time.Now()
	switch timeRange {
	case "today":
		startOfDay := now.Truncate(24 * time.Hour)
		filter.StartedAfter = &startOfDay
	case "week":
		weekAgo := now.AddDate(0, 0, -7)
		filter.StartedAfter = &weekAgo
	case "month":
		monthAgo := now.AddDate(0, -1, 0)
		filter.StartedAfter = &monthAgo
	}

	if search := r.URL.Query().Get("search"); search != "" {
		filter.ControllerCode = search
	}

	ctx := r.Context()

	w.Header().Set("Content-Type", "text/csv")
	w.Header().Set("Content-Disposition", "attachment; filename=connection_records.csv")

	if err := s.Connections.ExportCSV(ctx, teamID, filter, w); err != nil {
		// Can't change headers after writing has started, so log the error
		http.Error(w, "failed to export CSV", http.StatusInternalServerError)
		return
	}
}

// handleListAuditLogs returns paginated audit logs.
func (s *Server) handleListAuditLogsAPI(w http.ResponseWriter, r *http.Request) {
	teamIDStr := chi.URLParam(r, "teamID")
	teamID, err := uuid.Parse(teamIDStr)
	if err != nil {
		http.Error(w, "invalid team_id", http.StatusBadRequest)
		return
	}

	page := 1
	if pageStr := r.URL.Query().Get("page"); pageStr != "" {
		if parsed, err := strconv.Atoi(pageStr); err == nil && parsed > 0 {
			page = parsed
		}
	}

	pageSize := 20
	if sizeStr := r.URL.Query().Get("page_size"); sizeStr != "" {
		if parsed, err := strconv.Atoi(sizeStr); err == nil && parsed > 0 && parsed <= 100 {
			pageSize = parsed
		}
	}

	filter := repository.AuditFilter{
		Limit:  pageSize,
		Offset: (page - 1) * pageSize,
	}

	// Action filter
	if action := r.URL.Query().Get("action"); action != "" {
		filter.Action = action
	}

	// Time range filter
	if startStr := r.URL.Query().Get("start_time"); startStr != "" {
		if startTime, err := time.Parse(time.RFC3339, startStr); err == nil {
			filter.StartTime = &startTime
		}
	}
	if endStr := r.URL.Query().Get("end_time"); endStr != "" {
		if endTime, err := time.Parse(time.RFC3339, endStr); err == nil {
			filter.EndTime = &endTime
		}
	}

	ctx := r.Context()

	logs, err := s.AuditLogs.List(ctx, teamID, filter)
	if err != nil {
		http.Error(w, "failed to list audit logs", http.StatusInternalServerError)
		return
	}

	total, err := s.AuditLogs.Count(ctx, teamID, filter)
	if err != nil {
		http.Error(w, "failed to count audit logs", http.StatusInternalServerError)
		return
	}

	response := map[string]interface{}{
		"logs":      logs,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(response)
}
