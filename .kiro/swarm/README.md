# Hermes Data Pipeline Swarm Agent System

Multi-agent coordination untuk hermes-data-pipeline development dengan Hermes orchestration + Kiro workers.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    HERMES ORCHESTRATOR                       │
│              start-feature → dispatch → process              │
└──────────────────────────┬──────────────────────────────────┘
                           │ delegate_task (kiro-cli acp)
    ┌──────────────────────┼──────────────────────────────────┐
    ▼                      ▼                      ▼           ▼
┌──────────┐      ┌──────────────┐      ┌─────────┐   ┌───────────┐
│Developer │      │Data Engineer │      │   QA    │   │ DevSecOps │
│ (Rust)   │      │  (Python)    │      │         │   │           │
└────┬─────┘      └──────┬───────┘      └────┬────┘   └─────┬─────┘
     │                   │                   │              │
     └───────────────────┴───────────────────┴──────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   .kiro/swarm/        │
                    │   (shared state)      │
                    └───────────────────────┘
```

## Agents

| Agent | Focus | Tech Stack |
|-------|-------|------------|
| Developer | Rust backend, news collectors | Rust, tokio, reqwest |
| Data Engineer | Python pipelines, ETL | Python, pandas, qdrant |
| QA | Tests, validation | cargo test, pytest |
| DevSecOps | Security, CI/CD | cargo audit, safety |

## Usage

### CLI Commands

```bash
# Start feature workflow
python .kiro/swarm/swarm.py start-feature ".kiro/specs/feature-name/"

# Check status
python .kiro/swarm/swarm.py status

# Process completed events
python .kiro/swarm/swarm.py process-events

# Write event (from agent)
python .kiro/swarm/swarm.py write-event "developer" "task_complete" '{"result": "success", "next_suggested": "qa"}'
```

### Via Hermes

```
Mulai swarm untuk spec news-source-resilience
```

## Directory Structure

```
.kiro/swarm/
├── state.json              # Shared workflow state
├── swarm.py                # CLI orchestration tool
├── events/                 # Agent → Orchestrator messages
│   └── processed/          # Archived events
├── inbox/                  # Orchestrator → Agent tasks
│   ├── developer/
│   ├── data-engineer/
│   ├── qa/
│   ├── devsecops/
│   └── orchestrator/
├── artifacts/              # Shared work outputs
└── logs/                   # Execution logs
```

## Quality Gates

- `unit_tests` — cargo test / pytest
- `lint` — clippy / ruff
- `security_scan` — cargo audit / safety
- `integration_tests` — end-to-end pipeline tests
