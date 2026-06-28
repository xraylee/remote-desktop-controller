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

package config

import (
	"fmt"

	"github.com/caarlos0/env/v11"
)

// Config holds all runtime configuration for the API server.
// Values are read from environment variables with sensible defaults.
type Config struct {
	Port          int    `env:"API_PORT"          envDefault:"8080"`
	DatabaseURL   string `env:"DATABASE_URL"      envDefault:"postgres://rdcs:rdcs_dev@localhost:5432/rdcs?sslmode=disable"`
	RedisURL      string `env:"REDIS_URL"         envDefault:"redis://localhost:6379"`
	JWTPrivateKey string `env:"JWT_PRIVATE_KEY"`
	JWTPublicKey  string `env:"JWT_PUBLIC_KEY"`
	CORSOrigins   string `env:"CORS_ORIGINS"      envDefault:"http://localhost:3000"`
	RateLimitRPS  int    `env:"RATE_LIMIT_RPS"    envDefault:"100"`
	TOTPIssuer    string `env:"TOTP_ISSUER"       envDefault:"RDCS"`
}

// Load parses environment variables into a Config struct.
// It returns an error if required variables are malformed.
func Load() (*Config, error) {
	cfg := &Config{}
	if err := env.Parse(cfg); err != nil {
		return nil, fmt.Errorf("parse config: %w", err)
	}
	return cfg, nil
}
