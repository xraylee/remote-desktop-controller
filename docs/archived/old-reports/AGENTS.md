# AGENTS.md

> This file provides context and instructions for AI coding agents (Claude, Cursor, Copilot, Windsurf, etc.) working on this project.

## Project Overview

**RDCS** (Remote Desktop Control System) is a self-hosted, open-source remote desktop solution for small and medium businesses (SMBs). It aims to combine the ease of use of TeamViewer with the data sovereignty of RustDesk, at 1/5 the cost.

- **Repository**: remote-desktop-controller
- **License**: Apache 2.0
- **Status**: Planning & Design Phase (Q2 2026)
- **Target Users**: SMB IT administrators (50-500 employees) and their remote workers
- **Primary Language**: Chinese-first, with English documentation

## Current Phase

The project is in **Phase 2: Design & Architecture**. Completed work includes:

1. ✅ Market analysis of 9 remote desktop products (TeamViewer, AnyDesk, Splashtop, ScreenConnect, Sunlogin, ToDesk, RustDesk, Chrome RD, Windows RDP) across 8 dimensions with 60+ feature items
2. ✅ Product brainstorming: 28 ideas evaluated via SCAMPER, 5-Why, competitor analysis, constraint innovation, and reverse brainstorming
3. ✅ Top 3 product concepts identified: one-line deployment, concurrent session pricing, compliance audit recording

## Key Documents to Read First

| Document | Path | Purpose |
|----------|------|---------|
| README | `README.md` | Project overview and status |
| Market Analysis | `docs/research/market-analysis.md` | Competitive analysis with feature matrix |
| Product Brainstorming | `docs/research/product-brainstorming.md` | 28 evaluated product ideas |
| Roadmap | `docs/ROADMAP.md` | 5-phase development plan |
| Contributing Guide | `CONTRIBUTING.md` | Commit conventions and PR process |

## Planned Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Relay/Signaling Server | **Rust** | High-performance media relay, memory safety |
| Management API | **Go** | Fast HTTP server, strong concurrency, simple deployment |
| Desktop Client UI | **Flutter** (Rust backend) | Cross-platform from single codebase |
| Client Core Engine | **Rust** | Shared screen capture/codec/transport logic |
| Web Admin Console | **React + TypeScript** | Modern SPA, rich component ecosystem |
| Database | **PostgreSQL** | Relational data (users, devices, audit logs) |
| Cache | **Redis** | Session state, real-time presence |
| File Storage | **MinIO / S3-compatible** | Session recordings, file transfers |
| Deployment | **Docker Compose** | One-command self-hosted deployment |
| Reverse Proxy | **Caddy** | Automatic TLS via Let's Encrypt |
| Codecs | **H.264 / H.265 / VP9 / AV1** | GPU-accelerated encoding/decoding |
| Encryption | **NaCl (libsodium)** | XSalsa20-Poly1305 + X25519 key exchange |

## Architecture Decisions Made

### 1. Client-Server with Rendezvous + Relay Pattern
Similar to RustDesk but with an enterprise management layer. Clients connect via a Rendezvous server for signaling, then attempt P2P (UDP hole-punching). If P2P fails, traffic routes through a Relay server.

### 2. Dual-Mode Client UI
- **Employee mode**: Minimal interface — only a "Connect" button and device code input
- **Admin mode**: Full management dashboard — device list, user permissions, audit logs, session recordings
- The mode is determined by the user's role (RBAC), not a separate binary

### 3. Device-Code Auth (No Registration for End Users)
Terminal employees do NOT register accounts. The admin binds device codes to employees in the management console. Employees open the client and see an "Authorized" state immediately.

### 4. Concurrent Session Pricing Model
Price by concurrent sessions (e.g., 5 concurrent = covers 50-person company), NOT by per-user seats. This reduces cost by ~80% compared to TeamViewer.

### 5. 100% Self-Hosted by Default
All data (connections, file transfers, recordings, audit logs) stays on the customer's infrastructure. No telemetry to vendor servers. Optional managed hosting for customers without server expertise.

## Product Requirements Summary (from Brainstorming)

### Quick Wins (High Impact, Low Effort)
- One-line Docker deployment: `curl | bash` completes full setup including TLS
- Progressive UI: role-based interface complexity
- Device-code auth: no end-user registration
- Single-file green client (<10MB): no install, no admin rights needed
- Cloud trial → self-hosted migration path

### Big Bets (High Impact, High Effort)
- Remote desktop + lightweight IT asset management unified
- AI-assisted operations panel (device anomaly detection)
- Compliance audit: automatic session recording with encrypted storage
- Web management console (no desktop admin client needed)
- Reverse-install model (agent-first, controller-second)

## Coding Conventions

### General
- Follow `.editorconfig`: 2-space indentation, UTF-8, LF line endings, trim trailing whitespace
- Exception: Markdown files preserve trailing whitespace; Makefiles use tabs
- All code must include license headers (Apache 2.0)

### Commit Messages (Conventional Commits)
```
type(scope): description

Types: feat, fix, docs, style, refactor, perf, test, chore, ci, revert
Scopes: server, client, admin, relay, deploy, docs
```
Examples:
```
feat(server): implement rendezvous signaling endpoint
fix(client): resolve screen capture permission on macOS 15
docs(admin): add deployment guide for Ubuntu 24.04
```

### Rust Code
- Use `clippy` lints with zero warnings
- Prefer `thiserror` for error types, `anyhow` for application-level errors
- Use `tracing` for structured logging
- Async runtime: `tokio`
- Serialization: `serde` with `serde_json`

### Go Code
- Follow standard Go project layout
- Use `slog` for structured logging
- Error handling: wrap with context using `fmt.Errorf("context: %w", err)`
- HTTP framework: `net/http` with `chi` router

### TypeScript/React Code
- Strict mode enabled
- Use functional components with hooks
- State management: Zustand (lightweight) or React Context
- Styling: Tailwind CSS
- API calls: TanStack Query (React Query)

### Flutter/Dart Code
- Follow official Flutter style guide
- State management: Riverpod
- Use `freezed` for immutable data classes
- Navigation: GoRouter

## File Structure Guide

```
remote-desktop-controller/
├── README.md                    # Project entry point (bilingual)
├── AGENTS.md                    # ← This file (AI agent instructions)
├── LICENSE                      # Apache 2.0
├── CONTRIBUTING.md              # Contribution guidelines
├── CODE_OF_CONDUCT.md           # Community behavior standards
├── CHANGELOG.md                 # Version history
├── .editorconfig                # Editor formatting rules
├── .gitignore                   # Ignored files/patterns
├── .github/                     # GitHub templates
│   ├── ISSUE_TEMPLATE/          # Bug reports, feature requests
│   └── PULL_REQUEST_TEMPLATE/   # PR template
└── docs/                        # All documentation
    ├── README.md                # Documentation index
    ├── ROADMAP.md               # Milestone timeline
    ├── research/                # Market research & analysis
    │   ├── market-analysis.md   # 9-product comparison
    │   └── product-brainstorming.md  # Idea evaluation
    ├── specs/                   # Feature specifications (TBD)
    ├── architecture/            # Technical design docs (TBD)
    └── images/                  # Screenshots & diagrams
```

## Do's and Don'ts for AI Agents

### Do
- Read `docs/research/market-analysis.md` before making product decisions — it contains detailed competitive intelligence
- Reference the brainstorming document for validated product ideas
- Follow the planned tech stack unless you have strong justification for alternatives
- Use Chinese for user-facing text (UI strings, error messages) and English for code/docs
- Consider the target user: an IT administrator at a 100-person company, not a developer
- Design for self-hosted deployment — avoid external service dependencies
- Keep the employee-mode client as simple as possible (one button to connect)
- Write tests for all new functionality

### Don't
- Don't add features that require routing user data through third-party servers
- Don't make the free tier artificially limited in quality (no 720P caps, no ads)
- Don't over-engineer the MVP — focus on core remote connection + device management + audit
- Don't use AGPL-licensed code in the core relay server (Apache 2.0 compatibility)
- Don't create separate binaries for employee vs admin modes (use role-based UI)
- Don't add mobile client features until the desktop MVP is stable
- Don't ignore the SMB budget constraint — every feature should be evaluated against cost sensitivity

## Key Metrics to Optimize For

| Metric | Target | Rationale |
|--------|--------|-----------|
| Time to first connection | < 5 minutes | Beat industry average of 15-30 min |
| Deployment time | < 30 minutes | From `docker compose up` to first device connected |
| Client binary size | < 10MB (green), < 30MB (installer) | Easy distribution |
| LAN latency | < 10ms | Match AnyDesk's best-in-class 16.5ms |
| LAN frame rate | 120+ FPS | Exceed TeamViewer's 60 FPS |
| Free tier resolution | 1080P / 60FPS | Beat Sunlogin (720P) and match ToDesk free |

## Security Requirements (Non-Negotiable)

- All connections MUST use end-to-end encryption (AES-256 or NaCl XSalsa20-Poly1305)
- The relay server MUST NOT be able to decrypt session content (zero-knowledge relay)
- Authentication MUST support TOTP two-factor authentication
- All administrative operations MUST be recorded in audit logs
- Session recordings MUST be encrypted at rest
- No telemetry data sent to vendor servers without explicit opt-in
- TLS 1.3 for all signaling/API channels

## OpenSpec Workflow

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) for spec-driven development. Changes are tracked in `openspec/changes/` (gitignored — local workspace only).

Key commands:
```bash
npx @fission-ai/openspec@latest new change "<name>"     # Create a new change
npx @fission-ai/openspec@latest status --change "<name>" # Check progress
npx @fission-ai/openspec@latest instructions apply --change "<name>" --json  # Get implementation tasks
```

Active changes:
- `remote-desktop-system` — Core system proposal with 10 capability specs (connection, auth, file-transfer, device-management, security, collaboration, server, client, mobile, system-tools)
- `screenshot-analysis` — Completed: added product UI screenshots to market analysis
