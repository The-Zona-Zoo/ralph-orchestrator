# Ralph Orchestrator

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)](https://www.rust-lang.org/)
[![Build](https://img.shields.io/github/actions/workflow/status/mikeyobrien/ralph-orchestrator/ci.yml?branch=main&label=CI)](https://github.com/mikeyobrien/ralph-orchestrator/actions)
[![Coverage](https://img.shields.io/badge/coverage-65%25-yellowgreen)](coverage/index.html)
[![Mentioned in Awesome Claude Code](https://awesome.re/mentioned-badge.svg)](https://github.com/hesreallyhim/awesome-claude-code)


A hat-based orchestration framework that keeps Ralph in a loop until the task is done.

> "Me fail English? That's unpossible!" - Ralph Wiggum

**Notice:** Ralph Orchestrator is under active development. It works today, but expect rough edges and breaking changes between releases.

v1.0.0 was ralphed into existence with little oversight and guidance. v2.0.0 is a simpler, more-structured implementation. Looking for the old version? See [v1.2.3](https://github.com/mikeyobrien/ralph-orchestrator/tree/v1.2.3). 

<img width="912" height="712" alt="Screenshot 2026-01-20 at 10 27 57 AM" src="https://github.com/user-attachments/assets/91b08b47-8b0a-4e2c-b66e-88551c2c5cc6" />

## Table of Contents

- [What is Ralph?](#what-is-ralph)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Custom Backends and Per-Hat Configuration](#custom-backends-and-per-hat-configuration)
- [Presets](#presets)
- [Key Concepts](#key-concepts)
- [CLI Reference](#cli-reference)
- [Architecture](#architecture)
- [Building & Testing](#building--testing)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## What is Ralph?

Ralph implements the [Ralph Wiggum technique](https://ghuntley.com/ralph/) — autonomous task completion through continuous iteration.

> "The orchestrator is a thin coordination layer, not a platform. Ralph is smart; let Ralph do the work."

### Two Modes of Operation

Ralph supports two orchestration styles:

| Mode | Description | Best For |
|------|-------------|----------|
| **Traditional** | Simple loop — Ralph iterates until done | Quick tasks, simple automation, minimal config |
| **Hat-Based** | Ralph can wear many hats — specialized personas coordinate through events | Complex workflows, multi-step processes, role separation |

**Traditional mode** is the original Ralph Wiggum approach: start a loop, let it run until it outputs the completion promise. No roles, no events, just iteration.

**Hat-based mode** adds structure: specialized personas coordinate through events. You define the roles that fit your workflow — reviewers, testers, documenters, whatever makes sense. Presets provide ready-made patterns like TDD or spec-driven development.

### The Ralph Tenets

1. **Fresh Context Is Reliability** — Each iteration clears context. Re-read specs, plan, code every cycle.
2. **Backpressure Over Prescription** — Don't prescribe how; create gates that reject bad work.
3. **The Plan Is Disposable** — Regeneration costs one planning loop. Cheap.
4. **Disk Is State, Git Is Memory** — Files are the handoff mechanism.
5. **Steer With Signals, Not Scripts** — Add signs, not scripts.
6. **Let Ralph Ralph** — Sit *on* the loop, not *in* it.

See [AGENTS.md](AGENTS.md) for the full philosophy.

## Features

- **Multi-Backend Support** — Works with Claude Code, Kiro, Gemini CLI, Codex, Amp, Copilot CLI, and OpenCode
- **Hat System** — Specialized Ralph personas with distinct behaviors
- **Event-Driven Coordination** — Hats communicate through typed events with glob pattern matching
- **Backpressure Enforcement** — Gates that reject incomplete work (tests, lint, typecheck)
- **Presets Library** — 20+ pre-configured workflows for common development patterns
- **Interactive TUI** — Real-time terminal UI for monitoring Ralph's activity (enabled by default)
- **Memories** — Persistent learning across sessions stored in `.agent/memories.md`
- **Tasks** — Runtime work tracking stored in `.agent/tasks.jsonl`
- **Session Recording** — Record and replay sessions for debugging and testing (experimental)

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- At least one AI CLI:
  - [Claude Code](https://github.com/anthropics/claude-code) (recommended)
  - [Kiro](https://kiro.dev/)
  - [Gemini CLI](https://github.com/google-gemini/gemini-cli)
  - [Codex](https://github.com/openai/codex)
  - [Amp](https://github.com/sourcegraph/amp)
  - [Copilot CLI](https://docs.github.com/copilot) (`npm install -g @github/copilot`)
  - [OpenCode](https://opencode.ai/) (`curl -fsSL https://opencode.ai/install | bash`)

### Via npm (Recommended)

```bash
# Install globally
npm install -g @ralph-orchestrator/ralph-cli

# Or run directly with npx
npx @ralph-orchestrator/ralph-cli --version
```

### Via Homebrew (macOS)

```bash
brew install ralph-orchestrator
```

### Via Cargo

```bash
cargo install ralph-cli
```

### From Source

```bash
git clone https://github.com/mikeyobrien/ralph-orchestrator.git
cd ralph-orchestrator
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"

# Or create symlink
sudo ln -s $(pwd)/target/release/ralph /usr/local/bin/ralph
```

### Verify Installation

```bash
ralph --version
ralph --help
```

### Migrating from v1 (Python)

If you have the old Python-based Ralph v1 installed, uninstall it first to avoid conflicts:

```bash
# If installed via pip
pip uninstall ralph-orchestrator

# If installed via pipx
pipx uninstall ralph-orchestrator

# If installed via uv
uv tool uninstall ralph-orchestrator

# Verify removal
which ralph  # Should return nothing or point to new Rust version
```

The v1 Python version is no longer maintained. See [v1.2.3](https://github.com/mikeyobrien/ralph-orchestrator/tree/v1.2.3) for historical reference.

## Quick Start

### 1. Initialize a Project

```bash
# Traditional mode — simple loop, no hats (recommended for getting started)
ralph init --backend claude

# Hat-based mode — use a preset workflow with specialized personas
ralph init --preset tdd-red-green

# Combine preset with different backend
ralph init --preset spec-driven --backend kiro

# See all available presets
ralph init --list-presets
```

This creates `ralph.yml` in your current directory. Without a preset, you get traditional mode (no hats). With a preset, you get hat-based orchestration.

### 2. Define Your Task

**Option A:** Create a `PROMPT.md` file:

```bash
cat > PROMPT.md << 'EOF'
Build a REST API with the following endpoints:
- POST /users - Create a new user
- GET /users/:id - Get user by ID
- PUT /users/:id - Update user
- DELETE /users/:id - Delete user

Use Express.js with TypeScript. Include input validation
and proper error handling.
EOF
```

**Option B:** Pass inline prompt when running:

```bash
ralph run -p "Add input validation to the user API endpoints"
```

### 3. Run Ralph

```bash
# TUI mode (default) — real-time terminal UI for monitoring
ralph run

# With inline prompt
ralph run -p "Implement the login endpoint with JWT authentication"

# Headless mode (no TUI)
ralph run --no-tui

# Resume interrupted session
ralph run --continue

# Dry run (show what would execute)
ralph run --dry-run
```

### Alternative: SOP-Driven Sessions

For standalone planning and task generation (without Ralph's event loop), use these commands:

```bash
# Start an interactive PDD planning session
ralph plan                           # SOP prompts for input
ralph plan "build a REST API"        # Provide idea inline
ralph plan --backend kiro "my idea"  # Use specific backend

# Generate code task files from descriptions
ralph task                           # SOP prompts for input
ralph task "add authentication"      # From description
ralph task specs/feature/plan.md     # From PDD plan file
```

These commands spawn an interactive AI session with bundled SOPs — perfect for one-off planning without configuring a full workflow.

## Configuration

Ralph uses a YAML configuration file (`ralph.yml` by default).

### Traditional Mode (No Hats)

The simplest configuration — just a loop that runs until completion:

```yaml
# ralph.yml — Traditional mode
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
```

This runs Ralph in a loop. No hats, no events, no role switching. Ralph iterates until it outputs `LOOP_COMPLETE` or hits the iteration limit.

### Hat-Based Mode (Specialized Personas)

> Ralph can wear many hats.

Add a `hats` section to enable role-based orchestration. Hats subscribe to events (triggers) and publish events when done:

```yaml
# ralph.yml — Hat-based mode (example structure)
cli:
  backend: "claude"

event_loop:
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  starting_event: "task.start"

hats:
  my_hat:
    name: "🎯 My Hat"
    triggers: ["task.start"]        # Events that activate this hat
    publishes: ["work.done"]        # Events this hat can emit
    instructions: |
      Your instructions here...
```

With hats, Ralph publishes a starting event, which triggers the matching hat. That hat does its work and publishes an event, potentially triggering other hats. This event-driven handoff continues until completion.

> **Tip:** Use `ralph init --preset <name>` to get pre-configured hat workflows. See [Presets](#presets) for ready-made patterns like TDD, spec-driven development, and more.

### Full Configuration Reference

```yaml
# Event loop settings
event_loop:
  completion_promise: "LOOP_COMPLETE"  # Output that signals completion
  max_iterations: 100                   # Maximum orchestration loops
  max_runtime_seconds: 14400            # 4 hours max runtime
  idle_timeout_secs: 1800               # 30 min idle timeout
  starting_event: "task.start"          # First event published

# CLI backend settings
cli:
  backend: "claude"                     # claude, kiro, gemini, codex, amp, copilot, opencode, custom
  prompt_mode: "arg"                    # arg (CLI argument) or stdin

# Core behaviors (always injected into prompts)
core:
  scratchpad: ".agent/scratchpad.md"    # Shared memory across iterations
  specs_dir: "./specs/"                 # Directory for specifications
  guardrails:                           # Rules injected into every prompt
    - "Fresh context each iteration - scratchpad is memory"
    - "Don't assume 'not implemented' - search first"
    - "Backpressure is law - tests/typecheck/lint must pass"

# Memories — persistent learning across sessions (enabled by default)
memories:
  enabled: true                         # Set false to disable
  inject: auto                          # auto, manual, or none

# Tasks — runtime work tracking (enabled by default)
tasks:
  enabled: true                         # Set false to use scratchpad-only mode

# Custom hats (omit to use default planner/builder)
hats:
  my_hat:
    name: "My Hat Name"                 # Display name
    triggers: ["some.event"]            # Events that activate this hat
    publishes: ["other.event"]          # Events this hat can emit
    instructions: |                     # Prompt instructions
      What this hat should do...
```


## Custom Backends and Per-Hat Configuration

### Custom Backends

Beyond the built-in backends (Claude, Kiro, Gemini, Codex, Amp, Copilot, OpenCode), you can define custom backends to integrate any CLI-based AI agent:

```yaml
cli:
  backend: "custom"
  command: "my-agent"
  args: ["--headless", "--auto-approve"]
  prompt_mode: "arg"        # "arg" or "stdin"
  prompt_flag: "-p"         # Optional: flag for prompt argument
```

| Field | Description |
|-------|-------------|
| `command` | The CLI command to execute |
| `args` | Arguments inserted before the prompt |
| `prompt_mode` | How to pass the prompt: `arg` (command-line argument) or `stdin` |
| `prompt_flag` | Flag preceding the prompt (e.g., `-p`, `--prompt`). If omitted, prompt is positional. |

### Per-Hat Backend Configuration

Different hats can use different backends, enabling specialized tools for specialized tasks:

```yaml
cli:
  backend: "claude"  # Default for Ralph and hats without explicit backend

hats:
  builder:
    name: "🔨 Builder"
    description: "Implements code"
    triggers: ["build.task"]
    publishes: ["build.done"]
    backend: "claude"        # Explicit: Claude for coding

  researcher:
    name: "🔍 Researcher"
    description: "Researches technical questions"
    triggers: ["research.task"]
    publishes: ["research.done"]
    backend:                 # Kiro with custom agent (has MCP tools)
      type: "kiro"
      agent: "researcher"

  reviewer:
    name: "👀 Reviewer"
    description: "Reviews code changes"
    triggers: ["review.task"]
    publishes: ["review.done"]
    backend: "gemini"        # Different model for fresh perspective
```

**Backend Types:**

| Type | Syntax | Invocation |
|------|--------|------------|
| Named | `backend: "claude"` | Uses standard backend configuration |
| Kiro Agent | `backend: { type: "kiro", agent: "builder" }` | `kiro-cli --agent builder ...` |
| Custom | `backend: { command: "...", args: [...] }` | Your custom command |

**When to mix backends:**

| Scenario | Recommended Backend |
|----------|---------------------|
| Complex coding | Claude (best reasoning) |
| AWS/cloud tasks | Kiro with agent (MCP tools) |
| Code review | Different model (fresh perspective) |
| Internal tools | Custom backend |
| Cost optimization | Faster/cheaper model for simple tasks |

Hats without explicit `backend` inherit from `cli.backend`.

## Presets

Presets are pre-configured workflows for common development patterns.

### Development Workflows

| Preset | Pattern | Description |
|--------|---------|-------------|
| `feature` | Planner-Builder | Standard feature development |
| `feature-minimal` | Single hat | Minimal feature development |
| `tdd-red-green` | Test-Implement-Refactor | TDD with red-green-refactor cycle |
| `spec-driven` | Spec-Build-Verify | Specification-first development |
| `refactor` | Analyze-Plan-Execute | Code refactoring workflow |

### Debugging & Investigation

| Preset | Pattern | Description |
|--------|---------|-------------|
| `debug` | Investigate-Fix-Verify | Bug investigation and fixing |
| `incident-response` | Triage-Fix-Postmortem | Production incident handling |
| `code-archaeology` | Explore-Document-Present | Legacy code understanding |

### Review & Quality

| Preset | Pattern | Description |
|--------|---------|-------------|
| `review` | Analyze-Critique-Suggest | Code review workflow |
| `pr-review` | Multi-Perspective | PR review with specialized reviewers |
| `adversarial-review` | Critic-Defender | Devil's advocate review style |

### Documentation

| Preset | Pattern | Description |
|--------|---------|-------------|
| `docs` | Write-Review-Publish | Documentation writing |
| `documentation-first` | Doc-Implement-Sync | Doc-first development |

### Specialized

| Preset | Pattern | Description |
|--------|---------|-------------|
| `api-design` | Design-Implement-Document | API-first development |
| `migration-safety` | Analyze-Migrate-Verify | Safe code migrations |
| `performance-optimization` | Profile-Optimize-Benchmark | Performance tuning |
| `scientific-method` | Hypothesis-Experiment-Conclude | Experimental approach |
| `mob-programming` | Rotate roles | Simulated mob programming |
| `socratic-learning` | Question-Answer-Synthesize | Learning through dialogue |
| `research` | Gather-Analyze-Synthesize | Research and analysis |
| `gap-analysis` | Current-Target-Plan | Gap identification |

### Using Presets

```bash
# List all available presets
ralph init --list-presets

# Initialize with a preset
ralph init --preset tdd-red-green

# Use preset with different backend
ralph init --preset spec-driven --backend gemini

# Override existing config
ralph init --preset debug --force
```

## Key Concepts

### Hats

Hats are specialized Ralph personas. Each hat has:

- **Triggers**: Events that activate this hat
- **Publishes**: Events this hat can emit
- **Instructions**: Prompt injected when hat is active

View event history:

```bash
ralph events
```

### Memories and Tasks

Ralph uses two complementary systems for persistent state (both enabled by default):

**Memories** (`.agent/memories.md`) — Accumulated wisdom across sessions:
- Codebase patterns and conventions discovered
- Architectural decisions and rationale
- Recurring problem solutions (fixes)
- Project-specific context

**Tasks** (`.agent/tasks.jsonl`) — Runtime work tracking:
- Create, list, and close tasks during orchestration
- Track dependencies between tasks
- Used for loop completion verification

When memories and tasks are enabled, they replace the scratchpad for state management. Set `memories.enabled: false` and `tasks.enabled: false` to use the legacy scratchpad-only mode.

### Scratchpad (Legacy Mode)

When memories/tasks are disabled, all hats share `.agent/scratchpad.md` — persistent memory across iterations. This enables hats to build on previous work rather than starting fresh.

The scratchpad is the primary mechanism for:
- Task tracking (with `[ ]`, `[x]`, `[~]` markers)
- Context preservation between iterations
- Handoff between hats

### Backpressure

Ralph enforces quality gates through backpressure. When a builder publishes `build.done`, it must include evidence:

```
tests: pass, lint: pass, typecheck: pass
```

## CLI Reference

### Commands

| Command | Description |
|---------|-------------|
| `ralph run` | Run the orchestration loop (default) |
| `ralph resume` | Resume from existing scratchpad |
| `ralph plan` | Start an interactive PDD planning session |
| `ralph task` | Start an interactive code-task-generator session |
| `ralph events` | View event history |
| `ralph init` | Initialize configuration file |
| `ralph clean` | Clean up `.agent/` directory |
| `ralph emit` | Emit an event to the event log |
| `ralph tools` | Runtime tools for memories and tasks (agent-facing) |

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <FILE>` | Config file path (default: `ralph.yml`) |
| `-v, --verbose` | Verbose output |
| `--color <MODE>` | Color output: `auto`, `always`, `never` |

### `ralph run` Options

| Option | Description |
|--------|-------------|
| `-p, --prompt <TEXT>` | Inline prompt text |
| `-P, --prompt-file <FILE>` | Prompt file path |
| `--max-iterations <N>` | Override max iterations |
| `--completion-promise <TEXT>` | Override completion trigger |
| `--dry-run` | Show what would execute |
| `--no-tui` | Disable TUI mode (TUI is enabled by default) |
| `-a, --autonomous` | Force headless mode |
| `--idle-timeout <SECS>` | TUI idle timeout (default: 30) |
| `--record-session <FILE>` | Record session to JSONL |
| `-q, --quiet` | Suppress output (for CI) |
| `--continue` | Resume from existing scratchpad |

### `ralph init` Options

| Option | Description |
|--------|-------------|
| `--backend <NAME>` | Backend: `claude`, `kiro`, `gemini`, `codex`, `amp`, `copilot`, `opencode` |
| `--preset <NAME>` | Use preset configuration |
| `--list-presets` | List available presets |
| `--force` | Overwrite existing config |

### `ralph plan` Options

| Option | Description |
|--------|-------------|
| `<IDEA>` | Optional rough idea to develop (SOP prompts if not provided) |
| `-b, --backend <BACKEND>` | Backend to use (overrides config and auto-detection) |

### `ralph task` Options

| Option | Description |
|--------|-------------|
| `<INPUT>` | Optional description text or path to PDD plan file |
| `-b, --backend <BACKEND>` | Backend to use (overrides config and auto-detection) |

### `ralph tools` Subcommands

The `tools` command provides agent-facing utilities for runtime state management:

```bash
# Memory management (persistent learning)
ralph tools memory add "content" -t pattern --tags tag1,tag2
ralph tools memory search "query"
ralph tools memory list
ralph tools memory show <id>
ralph tools memory delete <id>

# Task management (runtime tracking)
ralph tools task add "Title" -p 2              # Create task (priority 1-5)
ralph tools task add "X" --blocked-by Y        # With dependency
ralph tools task list                           # All tasks
ralph tools task ready                          # Unblocked tasks only
ralph tools task close <id>                     # Mark complete
```

## Architecture

Ralph is organized as a Cargo workspace with seven crates:

| Crate | Purpose |
|-------|---------|
| `ralph-proto` | Protocol types: Event, Hat, Topic, Error |
| `ralph-core` | Business logic: EventLoop, HatRegistry, Config |
| `ralph-adapters` | CLI backend integrations (Claude, Kiro, Gemini, etc.) |
| `ralph-tui` | Terminal UI with ratatui |
| `ralph-cli` | Binary entry point and CLI parsing |
| `ralph-e2e` | End-to-end test harness for backend validation |
| `ralph-bench` | Benchmarking harness (dev-only) |

## Building & Testing

### Build

```bash
cargo build           # Debug build
cargo build --release # Release build
```

### Test

```bash
# Run all tests (includes smoke tests with JSONL replay)
cargo test

# Run smoke tests specifically
cargo test -p ralph-core smoke_runner

# Run Kiro-specific smoke tests
cargo test -p ralph-core kiro
```

### Smoke Tests

Smoke tests use recorded JSONL fixtures instead of live API calls — fast, free, and deterministic.

**Fixture locations:**
- `crates/ralph-core/tests/fixtures/basic_session.jsonl` — Claude CLI session
- `crates/ralph-core/tests/fixtures/kiro/` — Kiro CLI sessions

**Recording new fixtures:**

```bash
# Record a session
ralph run -c ralph.yml --record-session session.jsonl -p "your prompt"

# Or capture raw CLI output
claude -p "your prompt" 2>&1 | tee output.txt
```

### Linting

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Run `cargo clippy` and `cargo fmt`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

See [AGENTS.md](AGENTS.md) for development philosophy and conventions.

## License

MIT License — See [LICENSE](LICENSE) for details.

## Acknowledgments

- **[Geoffrey Huntley](https://ghuntley.com/ralph/)** — Creator of the Ralph Wiggum technique
- **[Harper Reed](https://harper.blog/)** — Spec-driven development methodology
- **[Strands Agent SOPs](https://github.com/strands-agents/agent-sop)** — Natural language workflows that enable AI agents to perform complex, multi-step tasks with consistency and reliability. 
- **[ratatui](https://ratatui.rs/)** — Terminal UI framework
- **[portable-pty](https://crates.io/crates/portable-pty)** — Cross-platform PTY support

---

*"I'm learnding!" - Ralph Wiggum*

---

[![Star History Chart](https://api.star-history.com/svg?repos=mikeyobrien/ralph-orchestrator&type=date&legend=top-left)](https://www.star-history.com/#mikeyobrien/ralph-orchestrator&type=date&legend=top-left)
