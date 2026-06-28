-- Copyright 2026 RDCS Contributors
-- SPDX-License-Identifier: Apache-2.0

-- RDCS Seed Data (002)
-- Development/test data. Do NOT run in production.
-- Executed automatically after 001_init_schema.sql via docker-entrypoint-initdb.d

-- ===========================================
-- Test Team
-- ===========================================
INSERT INTO teams (id, name, plan, max_concurrent) VALUES
    ('a0000000-0000-0000-0000-000000000001', '测试团队 Alpha', 'pro', 10);

-- ===========================================
-- Members (password: test123, bcrypt hash)
-- $2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy
-- ===========================================
INSERT INTO members (id, team_id, name, email, role, password_hash) VALUES
    ('b0000000-0000-0000-0000-000000000001',
     'a0000000-0000-0000-0000-000000000001',
     '张管理员',
     'admin@rdcs-test.local',
     'owner',
     '$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy'),

    ('b0000000-0000-0000-0000-000000000002',
     'a0000000-0000-0000-0000-000000000001',
     '李经理',
     'manager@rdcs-test.local',
     'manager',
     '$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy'),

    ('b0000000-0000-0000-0000-000000000003',
     'a0000000-0000-0000-0000-000000000001',
     '王工程师',
     'member@rdcs-test.local',
     'member',
     '$2a$10$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy');

-- ===========================================
-- Devices (5 devices: 3 macOS, 1 Windows, 1 Linux)
-- device_code: 9-digit numeric, displayed as "XXX XXX XXX"
-- ===========================================
INSERT INTO devices (id, team_id, device_code, device_name, platform, os_version, client_version, status, ip_address) VALUES
    ('c0000000-0000-0000-0000-000000000001',
     'a0000000-0000-0000-0000-000000000001',
     '100200301',
     'MacBook-Pro-张三',
     'macos',
     '15.0',
     '0.1.0',
     'online',
     '192.168.1.101'),

    ('c0000000-0000-0000-0000-000000000002',
     'a0000000-0000-0000-0000-000000000001',
     '100200302',
     'MacBook-Air-李四',
     'macos',
     '14.5',
     '0.1.0',
     'online',
     '192.168.1.102'),

    ('c0000000-0000-0000-0000-000000000003',
     'a0000000-0000-0000-0000-000000000001',
     '100200303',
     'iMac-会议室A',
     'macos',
     '15.0',
     '0.1.0',
     'offline',
     '192.168.1.103'),

    ('c0000000-0000-0000-0000-000000000004',
     'a0000000-0000-0000-0000-000000000001',
     '100200304',
     'DESKTOP-王五',
     'windows',
     '11 23H2',
     '0.1.0',
     'online',
     '192.168.1.104'),

    ('c0000000-0000-0000-0000-000000000005',
     'a0000000-0000-0000-0000-000000000001',
     '100200305',
     'ubuntu-server-01',
     'linux',
     'Ubuntu 24.04 LTS',
     '0.1.0',
     'offline',
     '192.168.1.105');

-- ===========================================
-- Connection Records (3 sessions with different paths)
-- ===========================================
INSERT INTO connection_records (id, team_id, controller_code, controlled_code, path, started_at, ended_at, duration_sec, bytes_transferred) VALUES
    ('d0000000-0000-0000-0000-000000000001',
     'a0000000-0000-0000-0000-000000000001',
     '100200301',
     '100200304',
     'L1',
     NOW() - INTERVAL '2 hours',
     NOW() - INTERVAL '1 hour 30 minutes',
     1800,
     524288000),

    ('d0000000-0000-0000-0000-000000000002',
     'a0000000-0000-0000-0000-000000000001',
     '100200302',
     '100200305',
     'L2',
     NOW() - INTERVAL '5 hours',
     NOW() - INTERVAL '4 hours',
     3600,
     1073741824),

    ('d0000000-0000-0000-0000-000000000003',
     'a0000000-0000-0000-0000-000000000001',
     '100200301',
     '100200303',
     'L3',
     NOW() - INTERVAL '1 day',
     NOW() - INTERVAL '1 day' + INTERVAL '45 minutes',
     2700,
     805306368);
