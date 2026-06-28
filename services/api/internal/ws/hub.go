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

package ws

import (
	"context"
	"encoding/json"
	"log/slog"
	"sync"
	"time"
)

// Event represents a real-time event to be pushed to WebSocket clients.
type Event struct {
	Type      string          `json:"type"`      // "device_online", "device_offline", "session_started", etc.
	TeamID    string          `json:"team_id"`
	Payload   json.RawMessage `json:"payload"`
	Timestamp time.Time       `json:"timestamp"`
}

// Hub manages a set of WebSocket clients, handling registration,
// unregistration, and broadcasting of events to connected clients.
type Hub struct {
	mu         sync.RWMutex
	clients    map[string]map[*Client]struct{} // teamID -> set of clients
	register   chan *Client
	unregister chan *Client
	broadcast  chan Event
}

// NewHub creates a new Hub ready to manage WebSocket connections.
func NewHub() *Hub {
	return &Hub{
		clients:    make(map[string]map[*Client]struct{}),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		broadcast:  make(chan Event, 256),
	}
}

// Run starts the Hub's main event loop. It processes client registrations,
// unregistrations, and broadcasts. It exits when ctx is cancelled.
func (h *Hub) Run(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			// Close all client send channels on shutdown.
			h.mu.Lock()
			for teamID, teamClients := range h.clients {
				for client := range teamClients {
					close(client.send)
				}
				delete(h.clients, teamID)
			}
			h.mu.Unlock()
			return

		case client := <-h.register:
			h.mu.Lock()
			if h.clients[client.teamID] == nil {
				h.clients[client.teamID] = make(map[*Client]struct{})
			}
			h.clients[client.teamID][client] = struct{}{}
			h.mu.Unlock()
			slog.Debug("ws client registered", "team_id", client.teamID)

		case client := <-h.unregister:
			h.mu.Lock()
			if teamClients, ok := h.clients[client.teamID]; ok {
				if _, exists := teamClients[client]; exists {
					delete(teamClients, client)
					close(client.send)
					if len(teamClients) == 0 {
						delete(h.clients, client.teamID)
					}
				}
			}
			h.mu.Unlock()
			slog.Debug("ws client unregistered", "team_id", client.teamID)

		case event := <-h.broadcast:
			h.BroadcastToTeam(event)
		}
	}
}

// BroadcastToTeam sends an event to all connected clients belonging
// to the specified team. It serializes the event to JSON once and
// delivers it via each client's send channel.
func (h *Hub) BroadcastToTeam(event Event) {
	data, err := json.Marshal(event)
	if err != nil {
		slog.Error("failed to marshal ws event", "error", err, "type", event.Type)
		return
	}

	h.mu.RLock()
	teamClients, ok := h.clients[event.TeamID]
	if !ok {
		h.mu.RUnlock()
		return
	}
	// Collect clients to avoid holding the lock during channel sends.
	targets := make([]*Client, 0, len(teamClients))
	for c := range teamClients {
		targets = append(targets, c)
	}
	h.mu.RUnlock()

	for _, c := range targets {
		select {
		case c.send <- data:
		default:
			// Client's send buffer is full; skip to avoid blocking.
			slog.Warn("ws client send buffer full, dropping event", "team_id", event.TeamID, "type", event.Type)
		}
	}
}

// ClientCount returns the number of currently connected clients for a team.
// This is primarily useful for testing and monitoring.
func (h *Hub) ClientCount(teamID string) int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients[teamID])
}

// TotalClientCount returns the total number of connected clients across all teams.
func (h *Hub) TotalClientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	total := 0
	for _, teamClients := range h.clients {
		total += len(teamClients)
	}
	return total
}
