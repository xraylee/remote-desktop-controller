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
	"log/slog"
	"net/http"

	"github.com/gorilla/websocket"

	"github.com/rdcs/rdcs-api/internal/auth"
)

// upgrader is the WebSocket upgrader with permissive origin checking.
// In production, CheckOrigin should be restricted to known domains.
var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		// Allow all origins for now; tighten in production.
		return true
	},
}

// HandleWebSocket returns an http.HandlerFunc that upgrades an HTTP
// connection to WebSocket after validating a JWT token provided as a
// query parameter (?token=xxx).
//
// The handler extracts team_id from the token claims and registers the
// new client with the provided Hub. ReadPump and WritePump goroutines
// are spawned to manage the connection lifecycle.
func HandleWebSocket(hub *Hub, publicKey string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// 1. Extract JWT token from query parameter.
		tokenString := r.URL.Query().Get("token")
		if tokenString == "" {
			http.Error(w, `{"error":"missing token parameter"}`, http.StatusUnauthorized)
			return
		}

		// 2. Validate the JWT token.
		claims, err := auth.ValidateToken(tokenString, publicKey)
		if err != nil {
			slog.Debug("ws jwt validation failed", "error", err)
			http.Error(w, `{"error":"invalid or expired token"}`, http.StatusUnauthorized)
			return
		}

		// 3. Upgrade the HTTP connection to WebSocket.
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			slog.Error("ws upgrade failed", "error", err)
			return
		}

		// 4. Create a Client and register with the Hub.
		client := &Client{
			hub:    hub,
			conn:   conn,
			teamID: claims.TeamID,
			send:   make(chan []byte, sendBufSize),
		}
		hub.register <- client

		// 5. Spawn ReadPump and WritePump goroutines.
		go client.WritePump()
		go client.ReadPump()
	}
}
