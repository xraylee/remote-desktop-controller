# Documentation

This directory contains all project documentation organized by category.

## Structure

```
docs/
├── README.md                  ← This file (documentation index)
├── ROADMAP.md                 ← Project roadmap and milestones
├── DEVELOPMENT.md             ← Development guide
├── research/                  ← Market research and product analysis
│   ├── market-analysis.md     ← 9-product deep comparison report
│   ├── product-brainstorming.md ← Product brainstorming v1
│   └── product-brainstorming-v2.md ← Product brainstorming v2
├── specs/                     ← Feature specification documents
│   ├── architecture-design.md ← System architecture design
│   ├── prd-v1.md             ← Product Requirements Document v1
│   ├── prd-review-report.md  ← PRD review report
│   └── wave-migration-plan.md ← Wave migration plan
├── architecture/              ← Technical architecture documents
│   └── (architecture docs)
├── decisions/                 ← Architecture Decision Records (ADRs)
│   ├── WEBRTC_ARCHITECTURE.md
│   ├── WEBRTC_CODEC_INTEGRATION_DECISION.md
│   └── WEBRTC_SOLUTION_COMPARISON.md
├── plans/                     ← Development phase plans
│   ├── plan-a-core-engine.md
│   ├── plan-b-signaling.md
│   ├── plan-c-relay.md
│   ├── plan-d-flutter-client.md
│   ├── plan-e-web-console.md
│   └── plan-f-dev-environment.md
├── progress/                  ← Development progress reports
│   ├── NEXT_STEPS.md
│   ├── final-project-status.md
│   └── (various completion reports)
├── testing/                   ← Testing documentation
│   ├── TESTING_GUIDELINES.md
│   ├── PHASE1_COMPLETION_REPORT.md
│   └── VIDEOTOOLBOX_CRASH_DIAGNOSIS.md
├── installation/              ← Installation and deployment guides
│   ├── README.md
│   ├── INSTALLATION_CHECKLIST.md
│   ├── APPLE_SILICON_FIX.md
│   ├── BEST_MIRRORS.md
│   └── CHINA_MIRROR_GUIDE.md
├── reviews/                   ← Project and technical reviews
│   ├── README.md
│   ├── PROJECT_REVIEW.md
│   ├── WebRTC_Integration_Review.md
│   └── architecture-review-report.md
├── archived/                  ← Archived/historical documents
│   └── README.md
├── prototypes/                ← Prototype code and experiments
└── images/                    ← Screenshots and diagrams
    └── (product screenshots used in research docs)
```

## Quick Links

### 📚 Research & Planning
| Document | Description | Status |
|----------|-------------|--------|
| [Market Analysis](research/market-analysis.md) | In-depth comparison of 9 remote desktop products across 8 dimensions with 60+ feature items and 19 UI screenshots | ✅ Complete |
| [Product Brainstorming](research/product-brainstorming.md) | 28 creative ideas evaluated through SCAMPER, 5-Why, competitor analysis, and constraint innovation frameworks | ✅ Complete |
| [Project Roadmap](ROADMAP.md) | 5-phase development plan from research to v1.0 release | ✅ Complete |
| [PRD v1](specs/prd-v1.md) | Product Requirements Document | ✅ Complete |

### 🏗️ Architecture & Design
| Document | Description | Status |
|----------|-------------|--------|
| [Architecture Design](specs/architecture-design.md) | System architecture design | ✅ Complete |
| [WebRTC Architecture](decisions/WEBRTC_ARCHITECTURE.md) | WebRTC integration architecture decision | ✅ Complete |
| [Codec Integration](decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md) | Video codec integration decision | ✅ Complete |
| [WebRTC Comparison](decisions/WEBRTC_SOLUTION_COMPARISON.md) | WebRTC solution comparison | ✅ Complete |

### 🔧 Development
| Document | Description | Status |
|----------|-------------|--------|
| [Development Guide](DEVELOPMENT.md) | Development setup and workflow | ✅ Complete |
| [Testing Guidelines](testing/TESTING_GUIDELINES.md) | Testing standards and procedures | ✅ Complete |
| [Phase Plans](plans/) | Detailed implementation plans for each phase | ✅ Complete |

### 📦 Installation & Deployment
| Document | Description | Status |
|----------|-------------|--------|
| [Installation Guide](installation/README.md) | Installation documentation index | ✅ Complete |
| [Setup Guide](../SETUP.md) | Basic setup instructions | ✅ Complete |
| [Apple Silicon Fix](installation/APPLE_SILICON_FIX.md) | Apple Silicon specific fixes | ✅ Complete |
| [China Mirror Guide](installation/CHINA_MIRROR_GUIDE.md) | Mirror configuration for China | ✅ Complete |

### 📊 Progress & Reviews
| Document | Description | Status |
|----------|-------------|--------|
| [Final Project Status](progress/final-project-status.md) | Overall project status | ✅ Complete |
| [Next Steps](progress/NEXT_STEPS.md) | Immediate next actions | ✅ Complete |
| [Project Review](reviews/PROJECT_REVIEW.md) | Project review report | ✅ Complete |
| [WebRTC Integration Review](reviews/WebRTC_Integration_Review.md) | WebRTC integration review | ✅ Complete |

## For Contributors

- Start with the [main README](../README.md) for project overview
- Read [CONTRIBUTING.md](../CONTRIBUTING.md) before making changes
- Check the [Roadmap](ROADMAP.md) to see what's being worked on
- Feature specs (when available) will be the source of truth for requirements
