-- Copyright 2026 RDCS Contributors
-- SPDX-License-Identifier: Apache-2.0

-- RDCS Initial Schema Migration (001)
-- Creates 6 core tables for the remote desktop control system.
-- Executed automatically on first PostgreSQL startup via docker-entrypoint-initdb.d

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ===========================================
-- Teams (organization / tenant)
-- ===========================================
CREATE TABLE IF NOT EXISTS teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(128) NOT NULL,
    plan VARCHAR(16) NOT NULL DEFAULT 'free' CHECK (plan IN ('free', 'basic', 'pro')),
    max_concurrent INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================
-- Members (admin/operator accounts)
-- ===========================================
CREATE TABLE IF NOT EXISTS members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(16) NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'manager', 'member')),
    password_hash VARCHAR(255) NOT NULL,
    totp_secret VARCHAR(255),
    totp_enabled BOOLEAN NOT NULL DEFAULT false,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================
-- Devices (remote machines with agent installed)
-- ===========================================
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    device_code CHAR(9) NOT NULL UNIQUE,
    device_name VARCHAR(255) NOT NULL,
    platform VARCHAR(32) NOT NULL CHECK (platform IN ('windows', 'macos', 'linux')),
    os_version VARCHAR(64),
    client_version VARCHAR(32),
    status VARCHAR(16) NOT NULL DEFAULT 'offline' CHECK (status IN ('online', 'offline')),
    last_seen TIMESTAMPTZ,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================
-- Connection Records (remote desktop sessions)
-- ===========================================
CREATE TABLE IF NOT EXISTS connection_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    controller_code CHAR(9) NOT NULL,
    controlled_code CHAR(9) NOT NULL,
    path VARCHAR(8) NOT NULL CHECK (path IN ('L1', 'L2', 'L3')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    duration_sec INTEGER,
    bytes_transferred BIGINT DEFAULT 0,
    recording_path VARCHAR(512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================
-- Audit Logs (all admin actions)
-- ===========================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    actor_id UUID REFERENCES members(id) ON DELETE SET NULL,
    action VARCHAR(64) NOT NULL,
    target_type VARCHAR(64),
    target_id UUID,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================
-- Recordings (encrypted session recordings)
-- ===========================================
CREATE TABLE IF NOT EXISTS recordings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    connection_id UUID NOT NULL REFERENCES connection_records(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    encryption_key BYTEA NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    duration_sec INTEGER,
    file_size BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

-- ===========================================
-- Indexes
-- ===========================================
CREATE INDEX IF NOT EXISTS idx_devices_team_status ON devices(team_id, status);
CREATE INDEX IF NOT EXISTS idx_devices_device_code ON devices(device_code);
CREATE INDEX IF NOT EXISTS idx_connection_records_team_started ON connection_records(team_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_team_created ON audit_logs(team_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recordings_team_created ON recordings(team_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_members_team_id ON members(team_id);

-- ===========================================
-- Trigger: auto-update updated_at on teams
-- ===========================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_teams_updated_at
    BEFORE UPDATE ON teams
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
