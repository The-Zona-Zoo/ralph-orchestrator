---
name: creating-hat-collections
description: Use when creating new Ralph hat collection presets, designing multi-agent workflows, or adding hats to existing presets
---

# Creating Hat Collections

## Overview

Hat collections are YAML presets defining multi-agent workflows for Ralph. Each **hat** is an agent persona that responds to events and publishes new ones. Ralph coordinates—you define the roles.

**Core principle:** Hats coordinate through events, not explicit transitions. Ralph handles routing automatically.

## Quick Reference: YAML Structure

```yaml
# Required top-level sections
event_loop:
  prompt_file: "PROMPT.md"           # or prompt: "inline prompt"
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 100
  max_runtime_seconds: 14400
  checkpoint_interval: 5

cli:
  backend: "claude"                   # claude | gemini | kiro | custom

core:
  scratchpad: ".agent/scratchpad.md"
  specs_dir: "./specs/"

# Hat definitions
hats:
  hat_key:                            # lowercase, underscores ok
    name: "Human Readable Name"
    triggers: ["event.name"]          # NOT subscriptions
    publishes: ["other.event"]        # NOT publications
    default_publishes: "other.event"  # Safety net
    instructions: |                   # NOT system_prompt
      [Role-specific instructions]
```

## Hat Definition Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name, emoji optional (e.g., "🔍 Analyzer") |
| `triggers` | Yes | Events this hat responds to |
| `publishes` | Yes | Events this hat can emit |
| `default_publishes` | Recommended | Fallback if hat forgets to publish |
| `instructions` | Yes | Role-specific prompt prepended to agent |

**Fields that DON'T exist:** `emoji` (put emoji in `name`), `description`, `system_prompt`, `subscriptions`, `publications`

## Validation Rules

| Rule | Status |
|------|--------|
| Each trigger → exactly ONE hat | **REQUIRED** (ambiguous routing = error) |
| No `task.start` or `task.resume` triggers | **RESERVED** for Ralph |
| Self-routing (trigger own events) | Allowed |
| Orphan events (no subscriber) | Allowed (Ralph catches) |

## Architecture Patterns

Choose a pattern that fits your workflow:

| Pattern | Description | Use When |
|---------|-------------|----------|
| **Pipeline** | A→B→C linear flow | Sequential stages (analyze→summarize) |
| **Critic-Actor** | One proposes, another critiques | Quality-critical work (code review) |
| **Supervisor-Worker** | Coordinator delegates to specialists | Complex task decomposition |
| **Scientific** | Observe→Hypothesize→Test→Fix | Debugging mysterious bugs |

## Event Naming Conventions

```
<phase>.ready / <phase>.done      # Phase transitions
<thing>.approved / <thing>.rejected  # Review gates
<noun>.found / <noun>.missing     # Discovery events
<action>.request / <action>.complete # Request-response
```

Examples: `analysis.complete`, `review.approved`, `build.blocked`, `spec.rejected`

## Event Flow Design

Design workflow as event chain, not state machine:

```
task.start → [Planner] → build.task → [Builder] → build.done → [Reviewer] → review.approved
                                                                          ↓
                                                             review.changes_requested → [Builder]
```

**Ralph is the universal fallback** - unhandled events go to Ralph, not nowhere.

## Instructions Pattern

Instructions should include:
1. **Role definition** - What this hat does
2. **Process steps** - How to do the work
3. **Event format** - How to publish events
4. **DON'Ts** - Common mistakes to avoid

```yaml
instructions: |
  ## ANALYZER MODE

  You analyze code for issues. One pass, then hand off.

  ### Process
  1. Read the code carefully
  2. Identify issues by category
  3. Publish findings

  ### Event Format
  ```
  <event topic="analysis.complete">
  issues_found: 3
  severity: minor
  </event>
  ```

  ### DON'T
  - Don't fix code yourself
  - Don't skip publishing an event
```

## Complete Example

```yaml
# code-review.yml - Focused code review workflow

event_loop:
  prompt_file: "PROMPT.md"
  completion_promise: "LOOP_COMPLETE"
  max_iterations: 50
  max_runtime_seconds: 3600

cli:
  backend: "claude"

core:
  scratchpad: ".agent/scratchpad.md"

hats:
  analyzer:
    name: "Analyzer"
    triggers: ["analyze.request"]
    publishes: ["analysis.complete", "analysis.blocked"]
    default_publishes: "analysis.complete"
    instructions: |
      ## ANALYZER MODE

      Examine code for bugs, security issues, and improvements.

      ### Event Format
      <event topic="analysis.complete">
      files_reviewed: [list]
      issues: [findings]
      </event>

  summarizer:
    name: "Summarizer"
    triggers: ["analysis.complete"]
    publishes: ["review.complete"]
    default_publishes: "review.complete"
    instructions: |
      ## SUMMARIZER MODE

      Create final review summary from analysis findings.

      ### Event Format
      <event topic="review.complete">
      summary: [organized findings]
      </event>
```

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Using `subscriptions`/`publications` | Use `triggers`/`publishes` |
| Using `system_prompt` | Use `instructions` |
| Adding `emoji`, `description` fields | Remove - not in schema |
| Missing `event_loop` section | Required - add with completion_promise |
| Explicit state machine transitions | Remove - Ralph routes by events |
| Two hats with same trigger | One trigger = one hat (or error) |
| Using `task.start` as trigger | Reserved - use semantic events |

## Testing Your Preset

```bash
# Run smoke tests (validates config parsing + event routing)
cargo test -p ralph-core smoke_runner

# Quick test run
cargo run --bin ralph -- run -c presets/my-preset.yml -p "test prompt" --dry-run
```

Use `/evaluate-presets` skill for comprehensive preset evaluation.

## Where to Put Presets

- `presets/` - Main preset collection
- `presets/minimal/` - Embedded presets for `ralph init`
