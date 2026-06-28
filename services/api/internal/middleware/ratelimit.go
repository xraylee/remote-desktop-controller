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

package middleware

import (
	"net/http"
	"sync"
	"time"
)

// RateLimit returns middleware that enforces a per-IP requests-per-second
// limit using a simple token-bucket algorithm.  When the limit is exceeded
// the middleware responds with HTTP 429 Too Many Requests.
func RateLimit(rps int) func(http.Handler) http.Handler {
	var mu sync.Mutex
	visitors := make(map[string]*bucket)

	// Reclaim stale entries every minute.
	go func() {
		ticker := time.NewTicker(time.Minute)
		defer ticker.Stop()
		for range ticker.C {
			mu.Lock()
			now := time.Now()
			for ip, b := range visitors {
				if now.Sub(b.lastSeen) > 3*time.Minute {
					delete(visitors, ip)
				}
			}
			mu.Unlock()
		}
	}()

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := realIP(r)

			mu.Lock()
			b, ok := visitors[ip]
			if !ok {
				b = &bucket{tokens: float64(rps), lastRefill: time.Now()}
				visitors[ip] = b
			}

			// Refill tokens based on elapsed time.
			now := time.Now()
			elapsed := now.Sub(b.lastRefill).Seconds()
			b.tokens += elapsed * float64(rps)
			if b.tokens > float64(rps) {
				b.tokens = float64(rps)
			}
			b.lastRefill = now
			b.lastSeen = now

			if b.tokens < 1 {
				mu.Unlock()
				http.Error(w, "rate limit exceeded", http.StatusTooManyRequests)
				return
			}

			b.tokens--
			mu.Unlock()

			next.ServeHTTP(w, r)
		})
	}
}

type bucket struct {
	tokens     float64
	lastRefill time.Time
	lastSeen   time.Time
}

// realIP extracts the client IP, respecting X-Forwarded-For and
// X-Real-IP headers set by reverse proxies.
func realIP(r *http.Request) string {
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		// Take the first entry (left-most) which is the original client.
		if idx := indexByte(xff, ','); idx >= 0 {
			return xff[:idx]
		}
		return xff
	}
	if xri := r.Header.Get("X-Real-IP"); xri != "" {
		return xri
	}
	return r.RemoteAddr
}

func indexByte(s string, b byte) int {
	for i := 0; i < len(s); i++ {
		if s[i] == b {
			return i
		}
	}
	return -1
}
