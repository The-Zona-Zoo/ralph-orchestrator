# Event Loop Specification

## Overview

Implement an event-driven orchestration loop with pub/sub messaging that enables multiple CLI agents ("ralphs") to collaborate on tasks. The loop processes a user-provided prompt file, routes work between hats via an event bus, and continues until a completion promise is encountered.

## Concepts

### Hats (Personas)

A **hat** is a set of instructions that defines how the CLI should behave for a given iteration:

- **Implementer**: Writes code, implements features
- **Reviewer**: Reviews code for bugs, style, security
- **Architect**: Designs systems, plans approaches
- **Debugger**: Investigates and fixes bugs
- **Custom**: User-defined roles

### Events

An **event** is a message published to the event bus with:
- **Topic**: Routing key (e.g., `impl.done`, `review.rejected`)
- **Payload**: Content/context for subscribers
- **Source**: Which hat published it

### Event Bus

Central pub/sub hub where:
- Hats **subscribe** to topic patterns (e.g., `impl.*`, `review.done`)
- Hats **publish** events when they complete work
- Events are routed to subscribed hats

## Execution Modes

Both modes use the same pub/sub infrastructure. Single-hat is just multi-hat with one subscriber.

### Single-Hat Mode (Classic Ralph Wiggum)

One hat subscribes to `*` (all events). Classic Ralph Wiggum loop using the pub/sub infrastructure.

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Single-Hat Event Loop                           │
│                                                                     │
│  ┌─────────────┐         ┌─────────────┐                           │
│  │ PROMPT.md   │────────▶│  Event Bus  │◀─── Completion Detector   │
│  └─────────────┘         └──────┬──────┘     (LOOP_COMPLETE)       │
│                                 │                                   │
│                                 ▼                                   │
│                        ┌─────────────┐                             │
│                        │   default   │                             │
│                        │    Hat      │                             │
│                        │             │                             │
│                        │ Subscribes: │                             │
│                        │     *       │  ◀── receives ALL events    │
│                        │             │                             │
│                        │ Publishes:  │                             │
│                        │ task.done   │  ──▶ loops back to self     │
│                        └─────────────┘                             │
│                                                                     │
│  State persisted via: .agent/scratchpad.md, git commits            │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Insight**: The prompt doesn't change, but the codebase does. Each iteration reads updated files/tests and makes incremental progress. Uses same pub/sub as multi-hat for consistency.

### Multi-Hat Mode (Pub/Sub)

Multiple hats collaborate via event-driven messaging.

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Multi-Hat Event Loop                           │
│                                                                     │
│  ┌─────────────┐         ┌─────────────┐                           │
│  │ Prompt File │────────▶│  Event Bus  │◀─── Completion Detector   │
│  │ (PROMPT.md) │         │  (pub/sub)  │                           │
│  └─────────────┘         └──────┬──────┘                           │
│                                 │                                   │
│              ┌──────────────────┼──────────────────┐               │
│              ▼                  ▼                  ▼               │
│     ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│     │ Implementer │    │  Reviewer   │    │  Debugger   │         │
│     │    Hat      │    │    Hat      │    │    Hat      │         │
│     │             │    │             │    │             │         │
│     │ Subscribes: │    │ Subscribes: │    │ Subscribes: │         │
│     │ task.*      │    │ impl.*      │    │ error.*     │         │
│     │ review.done │    │             │    │             │         │
│     │             │    │             │    │             │         │
│     │ Publishes:  │    │ Publishes:  │    │ Publishes:  │         │
│     │ impl.done   │    │ review.*    │    │ fix.done    │         │
│     └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                     │
│     Events route work between hats based on subscriptions          │
└─────────────────────────────────────────────────────────────────────┘
```

## CLI Backend

Supports any headless CLI tool:
- **Claude**: `claude -p "prompt"`
- **Gemini**: `gemini` (stdin)
- **Codex**: `codex --prompt "prompt"`
- **Amp**: `amp` (stdin)
- **Custom**: User-defined command in config

Output streams directly to stdout in real-time while being accumulated for event parsing.

## Event Syntax

Agents publish events using XML-style tags in their output:

```
<event topic="impl.done">
Implemented the authentication module with JWT support.
Files changed: src/auth.rs, src/middleware.rs
</event>
```

With optional target for direct handoff:
```
<event topic="handoff" target="reviewer">
Please review the auth changes in src/auth.rs
</event>
```

## Configuration

### Single-Hat Mode (Classic Ralph)

```yaml
# ralph.yml - Simple single-agent loop
mode: "single"  # One hat subscribes to *, classic Ralph loop

event_loop:
  prompt_file: "PROMPT.md"
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  max_runtime_seconds: 14400
  max_cost_usd: 50.0              # Stop if cost exceeds this
  max_consecutive_failures: 5      # Stop after N failures in a row
  checkpoint_interval: 5           # Git commit every N iterations

cli:
  backend: "claude"

# Implicit default hat (created automatically in single mode):
# hats:
#   default:
#     subscriptions: ["*"]  # Receives all events
#     instructions: <classic ralph wiggum instructions>
```

### Multi-Hat Mode (Pub/Sub)

```yaml
# ralph.yml - Multi-agent with pub/sub routing
mode: "multi"

event_loop:
  prompt_file: "PROMPT.md"
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  max_runtime_seconds: 14400
  starting_hat: "implementer"

cli:
  backend: "claude"

hats:
  implementer:
    name: "Implementer"
    subscriptions: ["task.*", "review.done", "fix.done"]
    publishes: ["impl.done"]
    instructions: |
      You are the implementation agent.
      Focus on writing clean, tested code.

  reviewer:
    name: "Reviewer"
    subscriptions: ["impl.*"]
    publishes: ["review.done", "review.rejected"]
    instructions: |
      You are the code review agent.
      Review for bugs, style, and security.

  debugger:
    name: "Debugger"
    subscriptions: ["error.*", "review.rejected"]
    publishes: ["fix.done"]
    instructions: |
      You are the debugging specialist.
      Investigate and fix issues.
```

## Prepended Instructions

### Single-Hat Mode

The classic Ralph Wiggum instructions prepended to every iteration:

```
ORCHESTRATION CONTEXT:
You are running within the Ralph Orchestrator loop. This system will call you
repeatedly for multiple iterations until the overall task is complete.

IMPORTANT INSTRUCTIONS:
1. Implement only ONE small, focused task per iteration
2. Mark subtasks complete as you finish them (update PROMPT.md checkboxes)
3. Commit your changes after each iteration for checkpointing
4. Use .agent/workspace/ for temporary files

WORKFLOW:
- Explore: Research and understand the codebase
- Plan: Design your implementation approach
- Implement: Write tests first (TDD), then code
- Commit: Commit changes with clear messages

AGENT SCRATCHPAD:
Before starting, check .agent/scratchpad.md for previous progress.
At iteration end, update it with:
- What you accomplished
- What remains to be done
- Any blockers or decisions made

Do NOT restart from scratch if scratchpad shows progress.

COMPLETION:
When ALL tasks in PROMPT.md are complete, output:
LOOP_COMPLETE

---
ORIGINAL PROMPT:
{prompt_content}
```

### Multi-Hat Mode

See Event Syntax section - agents use `<event>` tags to communicate.

## Acceptance Criteria

### Single-Hat Mode

- **Given** `mode: "single"` in config
- **When** the loop initializes
- **Then** a default hat is created with subscription `["*"]`

- **Given** single-hat mode starts
- **When** the loop begins
- **Then** a `task.start` event is published to the bus
- **And** the default hat receives it (subscribed to `*`)

- **Given** CLI output does NOT contain `LOOP_COMPLETE`
- **When** the iteration completes
- **Then** a `task.continue` event is published
- **And** the default hat receives it and runs again

- **Given** CLI output contains `LOOP_COMPLETE`
- **When** the output is parsed
- **Then** the loop terminates successfully

- **Given** `.agent/scratchpad.md` exists
- **When** instructions are prepended
- **Then** agent is instructed to read scratchpad and continue from previous progress

### Safeguards

- **Given** `max_cost_usd: 50.0` in config
- **When** cumulative cost exceeds $50
- **Then** the loop terminates with cost limit reason

- **Given** `max_consecutive_failures: 5` in config
- **When** 5 iterations fail in a row (non-zero exit)
- **Then** the loop terminates with failure limit reason

- **Given** `checkpoint_interval: 5` in config
- **When** iteration 5, 10, 15... completes
- **Then** a git commit is created with checkpoint message

### Event Bus (Multi-Hat Mode)

- **Given** a hat subscribed to `impl.*`
- **When** an event with topic `impl.done` is published
- **Then** the hat receives the event

- **Given** a hat subscribed to `task.start`
- **When** an event with topic `impl.done` is published
- **Then** the hat does NOT receive the event

### Event Parsing

- **Given** CLI output contains `<event topic="impl.done">content</event>`
- **When** output is parsed
- **Then** an event is created with topic `impl.done` and the content as payload

- **Given** CLI output contains `<event topic="handoff" target="reviewer">`
- **When** output is parsed
- **Then** the event is routed directly to the reviewer hat

### Orchestration Loop

- **Given** a prompt file and starting hat
- **When** the loop starts
- **Then** a `task.start` event is published with prompt contents

- **Given** events are pending for a hat
- **When** the hat executes
- **Then** CLI output streams to stdout in real-time

- **Given** CLI output contains the completion promise
- **When** the output is parsed
- **Then** the loop terminates successfully

- **Given** max_iterations is reached
- **When** the limit is exceeded
- **Then** the loop terminates with timeout reason

### CLI Backends

- **Given** `backend: "claude"` in config
- **When** executing a prompt
- **Then** `claude -p "prompt"` is invoked

- **Given** `backend: "custom"` with `prompt_mode: "stdin"`
- **When** executing a prompt
- **Then** the prompt is written to the CLI's stdin

## Error Handling

1. **Unknown topic**: Log warning, event is dropped (no subscribers)
2. **CLI failure**: Log error, publish `error.cli` event
3. **Parse failure**: Log warning, treat output as raw text
4. **Timeout**: Kill CLI process, publish `error.timeout` event

## Crate Placement

| Component | Crate |
|-----------|-------|
| Event, EventBus, Hat, Topic types | `ralph-proto` |
| HatRegistry, EventLoopConfig, EventLoop | `ralph-core` |
| CliBackend, CliConfig, CliExecutor | `ralph-adapters` |
| CLI entry point, config loading | `ralph-cli` |

## Implementation Order

1. **Phase 1**: Event and EventBus types in `ralph-proto`
2. **Phase 2**: Hat registry and config in `ralph-core`
3. **Phase 3**: CLI executor with streaming in `ralph-adapters`
4. **Phase 4**: EventLoop orchestration in `ralph-core`
5. **Phase 5**: CLI entry point in `ralph-cli`
