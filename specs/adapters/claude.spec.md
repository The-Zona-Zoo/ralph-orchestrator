---
status: review
gap_analysis: 2026-01-13
related:
  - ../cli-adapters.spec.md
  - ../pty-mode.spec.md
---

# Claude Adapter

Anthropic's Claude Code CLI. The default and most thoroughly tested backend with Ralph.

## Configuration

| Property | Value |
|----------|-------|
| Command | `claude` |
| Headless flags | `--dangerously-skip-permissions` |
| Prompt mode | Argument (`-p "prompt"`) |
| TTY required | **Yes** — Ralph auto-enables PTY |
| Auto-detect | `claude --version` |
| Auth | `claude` CLI login (interactive) |

## Invocation

```bash
claude --dangerously-skip-permissions -p "your prompt"
```

## Behavior

### TTY Requirement

Claude hangs indefinitely without a TTY, even with the `-p` flag. This is a known issue ([GitHub #9026](https://github.com/anthropics/claude-code/issues/9026)).

**Ralph's behavior:** When `backend: "claude"` is selected (explicitly or via auto-detection), Ralph auto-enables PTY mode regardless of config. This ensures Claude always has a TTY.

### Large Prompt Handling

Large stdin inputs (>7000 chars) may produce empty output ([GitHub #7263](https://github.com/anthropics/claude-code/issues/7263)).

**Ralph's behavior:** For prompts exceeding the threshold, Ralph writes the prompt to a temp file and instructs Claude to read from it.

### Permission Bypass

The `--dangerously-skip-permissions` flag bypasses all permission prompts. This is required for non-interactive operation — without it, Claude will pause waiting for user approval on file writes, command execution, etc.

## Acceptance Criteria

**Given** `backend: "claude"` in config
**When** Ralph executes an iteration
**Then** PTY mode is auto-enabled regardless of `pty_mode` setting

**Given** `backend: "auto"` and Claude is installed
**When** Ralph starts
**Then** Claude is selected (first in priority order)

**Given** a prompt exceeding 7000 characters
**When** Ralph invokes Claude
**Then** the prompt is written to a temp file to avoid the large stdin bug
