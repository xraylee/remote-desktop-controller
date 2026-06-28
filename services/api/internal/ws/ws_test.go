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
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/google/uuid"

	"github.com/rdcs/rdcs-api/internal/auth"
	"github.com/rdcs/rdcs-api/internal/model"
)

// testKeyPair generates an RSA key pair for test JWT signing.
func testKeyPair(t *testing.T) (privPEM, pubPEM string) {
	t.Helper()
	priv, pub, err := auth.GenerateRSAKeyPair()
	if err != nil {
		t.Fatalf("GenerateRSAKeyPair() error = %v", err)
	}
	return priv, pub
}

// testToken creates a valid JWT access token for a test member.
func testToken(t *testing.T, privPEM string, teamID string) string {
	t.Helper()
	tid, _ := uuid.Parse(teamID)
	member := &model.Member{
		ID:     uuid.New(),
		TeamID: tid,
		Name:   "Test User",
		Email:  "test@example.com",
		Role:   "admin",
	}
	pair, err := auth.GenerateTokenPair(member, privPEM)
	if err != nil {
		t.Fatalf("GenerateTokenPair() error = %v", err)
	}
	return pair.AccessToken
}

// startTestHub creates and starts a Hub, returning a cancel function to stop it.
func startTestHub(t *testing.T) (*Hub, context.CancelFunc) {
	t.Helper()
	hub := NewHub()
	ctx, cancel := context.WithCancel(context.Background())
	go hub.Run(ctx)
	t.Cleanup(cancel)
	return hub, cancel
}

// startWSServer creates an httptest.Server with the WebSocket handler.
func startWSServer(t *testing.T, hub *Hub, pubPEM string) *httptest.Server {
	t.Helper()
	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/ws", HandleWebSocket(hub, pubPEM))
	srv := httptest.NewServer(mux)
	t.Cleanup(srv.Close)
	return srv
}

// wsURL converts an http test server URL to a ws:// URL with a token query param.
func wsURL(srv *httptest.Server, token string) string {
	return fmt.Sprintf("ws%s/api/v1/ws?token=%s", strings.TrimPrefix(srv.URL, "http"), token)
}

// connectWS dials a WebSocket connection to the test server.
func connectWS(t *testing.T, url string) *websocket.Conn {
	t.Helper()
	conn, resp, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		if resp != nil {
			t.Fatalf("Dial error: %v, HTTP status: %d", err, resp.StatusCode)
		}
		t.Fatalf("Dial error: %v", err)
	}
	return conn
}

// waitForRegistration polls until the hub has the expected client count.
func waitForRegistration(t *testing.T, hub *Hub, teamID string, expected int) {
	t.Helper()
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if hub.ClientCount(teamID) == expected {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for %d clients on team %s, got %d", expected, teamID, hub.ClientCount(teamID))
}

// readMessage reads a message from a WebSocket connection with a timeout.
func readMessage(t *testing.T, conn *websocket.Conn, timeout time.Duration) (int, []byte) {
	t.Helper()
	conn.SetReadDeadline(time.Now().Add(timeout))
	msgType, data, err := conn.ReadMessage()
	if err != nil {
		t.Fatalf("ReadMessage error: %v", err)
	}
	return msgType, data
}

// --- Acceptance Criterion Tests ---

// TestWSUpgradeSucceeds verifies that a valid JWT + Upgrade header results
// in a successful WebSocket handshake (101 Switching Protocols).
func TestWSUpgradeSucceeds(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	conn := connectWS(t, wsURL(srv, token))
	defer conn.Close()

	// Verify the client was registered in the hub.
	waitForRegistration(t, hub, teamID, 1)

	if hub.ClientCount(teamID) != 1 {
		t.Errorf("expected 1 client, got %d", hub.ClientCount(teamID))
	}
}

// TestWSInvalidJWTRejected verifies that an invalid or missing JWT token
// results in a 401 Unauthorized response.
func TestWSInvalidJWTRejected(t *testing.T) {
	_, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	tests := []struct {
		name  string
		token string
	}{
		{"missing token", ""},
		{"invalid token", "not-a-valid-jwt"},
		{"tampered token", "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.e30.invalid"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			url := wsURL(srv, tt.token)
			_, resp, err := websocket.DefaultDialer.Dial(url, nil)
			if err == nil {
				t.Fatal("expected dial error, got nil")
			}
			if resp != nil && resp.StatusCode != http.StatusUnauthorized {
				t.Errorf("expected status 401, got %d", resp.StatusCode)
			}
		})
	}
}

// TestBroadcastToTeam verifies that an event is delivered to all connected
// clients belonging to the target team.
func TestBroadcastToTeam(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	// Connect two clients for the same team.
	conn1 := connectWS(t, wsURL(srv, token))
	defer conn1.Close()
	conn2 := connectWS(t, wsURL(srv, token))
	defer conn2.Close()

	waitForRegistration(t, hub, teamID, 2)

	// Broadcast an event to the team.
	payload, _ := json.Marshal(map[string]string{"device_code": "ABC123"})
	hub.BroadcastToTeam(Event{
		Type:      "device_online",
		TeamID:    teamID,
		Payload:   payload,
		Timestamp: time.Now(),
	})

	// Both clients should receive the event.
	for i, conn := range []*websocket.Conn{conn1, conn2} {
		_, data := readMessage(t, conn, 2*time.Second)
		var event Event
		if err := json.Unmarshal(data, &event); err != nil {
			t.Fatalf("client %d: failed to unmarshal event: %v", i+1, err)
		}
		if event.Type != "device_online" {
			t.Errorf("client %d: event type = %q, want %q", i+1, event.Type, "device_online")
		}
		if event.TeamID != teamID {
			t.Errorf("client %d: team_id = %q, want %q", i+1, event.TeamID, teamID)
		}
	}
}

// TestBroadcastToTeamDoesNotLeakToOtherTeam verifies that events for one
// team are not delivered to clients of another team.
func TestBroadcastToTeamDoesNotLeakToOtherTeam(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamA := "33333333-3333-3333-3333-333333333333"
	teamB := "44444444-4444-4444-4444-444444444444"

	tokenA := testToken(t, privPEM, teamA)
	tokenB := testToken(t, privPEM, teamB)

	connA := connectWS(t, wsURL(srv, tokenA))
	defer connA.Close()
	connB := connectWS(t, wsURL(srv, tokenB))
	defer connB.Close()

	waitForRegistration(t, hub, teamA, 1)
	waitForRegistration(t, hub, teamB, 1)

	// Broadcast only to team A.
	payload, _ := json.Marshal(map[string]string{"device_code": "XYZ"})
	hub.BroadcastToTeam(Event{
		Type:      "device_offline",
		TeamID:    teamA,
		Payload:   payload,
		Timestamp: time.Now(),
	})

	// Team A client should receive it.
	_, data := readMessage(t, connA, 2*time.Second)
	var event Event
	if err := json.Unmarshal(data, &event); err != nil {
		t.Fatalf("team A: failed to unmarshal event: %v", err)
	}
	if event.Type != "device_offline" {
		t.Errorf("team A: event type = %q, want %q", event.Type, "device_offline")
	}

	// Team B client should NOT receive anything within a short window.
	connB.SetReadDeadline(time.Now().Add(200 * time.Millisecond))
	_, _, err := connB.ReadMessage()
	if err == nil {
		t.Error("team B: unexpectedly received a message meant for team A")
	}
}

// TestClientDisconnectCleanup verifies that when a WebSocket client
// disconnects, it is properly removed from the hub.
func TestClientDisconnectCleanup(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	conn := connectWS(t, wsURL(srv, token))
	waitForRegistration(t, hub, teamID, 1)

	// Close the client connection.
	conn.Close()

	// Wait for the hub to unregister the client.
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if hub.ClientCount(teamID) == 0 {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("expected 0 clients after disconnect, got %d", hub.ClientCount(teamID))
}

// TestHubConcurrentClients verifies that 10 concurrent WebSocket clients
// all successfully connect and receive broadcast events.
func TestHubConcurrentClients(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	const numClients = 10
	conns := make([]*websocket.Conn, numClients)
	var mu sync.Mutex
	var errs []error

	// Connect all clients concurrently.
	var wg sync.WaitGroup
	for i := 0; i < numClients; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			conn, resp, err := websocket.DefaultDialer.Dial(wsURL(srv, token), nil)
			mu.Lock()
			defer mu.Unlock()
			if err != nil {
				if resp != nil {
					errs = append(errs, fmt.Errorf("client %d: dial error: %v, status: %d", idx, err, resp.StatusCode))
				} else {
					errs = append(errs, fmt.Errorf("client %d: dial error: %v", idx, err))
				}
				return
			}
			conns[idx] = conn
		}(i)
	}
	wg.Wait()

	if len(errs) > 0 {
		for _, e := range errs {
			t.Error(e)
		}
		t.Fatal("some clients failed to connect")
	}

	// Clean up all connections at the end.
	defer func() {
		for _, c := range conns {
			if c != nil {
				c.Close()
			}
		}
	}()

	// Wait for all clients to register.
	waitForRegistration(t, hub, teamID, numClients)

	if count := hub.ClientCount(teamID); count != numClients {
		t.Fatalf("expected %d clients, got %d", numClients, count)
	}

	// Broadcast an event and verify all clients receive it.
	payload, _ := json.Marshal(map[string]string{"status": "connected"})
	hub.BroadcastToTeam(Event{
		Type:      "session_started",
		TeamID:    teamID,
		Payload:   payload,
		Timestamp: time.Now(),
	})

	var receivedMu sync.Mutex
	received := 0

	var readWg sync.WaitGroup
	for i := 0; i < numClients; i++ {
		readWg.Add(1)
		go func(idx int) {
			defer readWg.Done()
			conns[idx].SetReadDeadline(time.Now().Add(3 * time.Second))
			_, data, err := conns[idx].ReadMessage()
			if err != nil {
				t.Errorf("client %d: ReadMessage error: %v", idx, err)
				return
			}
			var event Event
			if err := json.Unmarshal(data, &event); err != nil {
				t.Errorf("client %d: unmarshal error: %v", idx, err)
				return
			}
			if event.Type != "session_started" {
				t.Errorf("client %d: type = %q, want %q", idx, event.Type, "session_started")
				return
			}
			receivedMu.Lock()
			received++
			receivedMu.Unlock()
		}(i)
	}
	readWg.Wait()

	if received != numClients {
		t.Errorf("expected %d clients to receive broadcast, got %d", numClients, received)
	}
}

// TestHubRunShutdown verifies that cancelling the hub context cleanly shuts
// down the event loop without panicking.
func TestHubRunShutdown(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, cancel := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	conn := connectWS(t, wsURL(srv, token))
	defer conn.Close()

	waitForRegistration(t, hub, teamID, 1)

	// Cancel the hub context; this should not panic.
	cancel()
	time.Sleep(100 * time.Millisecond)

	// The hub should have zero clients after shutdown.
	if total := hub.TotalClientCount(); total != 0 {
		t.Errorf("expected 0 total clients after shutdown, got %d", total)
	}
}

// TestEventJSONFormat verifies that events are serialized with the correct
// JSON structure expected by web console clients.
func TestEventJSONFormat(t *testing.T) {
	privPEM, pubPEM := testKeyPair(t)
	hub, _ := startTestHub(t)
	srv := startWSServer(t, hub, pubPEM)

	teamID := "33333333-3333-3333-3333-333333333333"
	token := testToken(t, privPEM, teamID)

	conn := connectWS(t, wsURL(srv, token))
	defer conn.Close()

	waitForRegistration(t, hub, teamID, 1)

	now := time.Now().Truncate(time.Millisecond)
	payload, _ := json.Marshal(map[string]string{"device_code": "DEV001", "platform": "windows"})
	hub.BroadcastToTeam(Event{
		Type:      "device_online",
		TeamID:    teamID,
		Payload:   payload,
		Timestamp: now,
	})

	_, data := readMessage(t, conn, 2*time.Second)

	// Verify the JSON contains all expected fields.
	var raw map[string]interface{}
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("failed to unmarshal raw JSON: %v", err)
	}

	if raw["type"] != "device_online" {
		t.Errorf("type = %v, want %q", raw["type"], "device_online")
	}
	if raw["team_id"] != teamID {
		t.Errorf("team_id = %v, want %q", raw["team_id"], teamID)
	}
	if raw["timestamp"] == nil {
		t.Error("timestamp is missing from event JSON")
	}
	if raw["payload"] == nil {
		t.Error("payload is missing from event JSON")
	}
}
