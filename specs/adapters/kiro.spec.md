---
status: review
gap_analysis: 2026-01-13
related:
  - ../cli-adapters.spec.md
---

# Kiro Adapter

AWS's Kiro CLI coding assistant (formerly Amazon Q Developer CLI).

## Configuration

| Property | Value |
|----------|-------|
| Command | `kiro-cli` |
| Subcommand | `chat` |
| Headless flags | `--no-interactive --trust-all-tools` |
| Prompt mode | Argument (positional after flags) |
| TTY required | No |
| Auto-detect | `kiro-cli --version` |
| Auth | `kiro-cli` login (interactive) |

## Invocation

```bash
kiro-cli chat --no-interactive --trust-all-tools "your prompt"
```

## Behavior

### Subcommand Requirement

Kiro requires the `chat` subcommand for prompt execution. The subcommand is passed in the adapter's `args` array, not as part of the command.

### Tool Trust

`--trust-all-tools` enables autonomous tool use without confirmation prompts. Built-in tools include: `read`, `write`, `shell`, `aws`, `report`.

### Known Issue

`--no-interactive` may occasionally still prompt for input. A fix is in progress upstream. Ralph should handle unexpected prompts gracefully by timing out.

## Acceptance Criteria

**Given** `backend: "kiro"` in config
**When** Ralph builds the command
**Then** the command includes `chat` subcommand in args array

**Given** `backend: "kiro"` in config
**When** Ralph executes an iteration
**Then** both `--no-interactive` and `--trust-all-tools` flags are included
