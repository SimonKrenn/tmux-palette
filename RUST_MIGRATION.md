# Rust Migration State

This document keeps the Rust port resumable across sessions. Update it whenever a migration milestone lands.

## Goal

Port `tmux-palette` from Bun/TypeScript to Rust while preserving user-facing behavior, especially tmux popup launch semantics, delayed action dispatch, JSON config compatibility, and rendering/input parity.

## Current Status

- Rust implementation lives under `crates/tmux-palette/`.
- TypeScript/Bun remains the production implementation.
- Current Rust milestone targets pure parity modules first:
  - model/action types
  - text width/truncation/aliases
  - fuzzy filtering
  - render helper logic
  - command palette invariants
  - action encoding

## Compatibility Requirements

- Preserve config files under `~/.config/tmux-palette/`.
- Preserve action JSON shapes: `tmux`, `shell`, `popup`, `palette`.
- Preserve delayed dispatch: selected action is written to a command file and executed only after popup closes.
- Preserve `--measure` tab-separated output before replacing the shell launcher.
- Keep JSON parsing permissive: unknown fields should not break user configs.

## Next Phases

1. Finish pure module parity and tests.
2. Add config/theme loading with user overrides.
3. Port custom palettes and plugin command execution.
4. Port dynamic tmux palettes (`find-pane`, `move-pane`).
5. Port TUI state machine and terminal IO.
6. Add Rust launcher behind an opt-in flag.
7. Switch default launcher after parity validation.

## Validation Commands

```bash
cargo test
bun test
bun run typecheck
```

## Commit Policy

Make small milestone commits when the Rust port reaches a coherent, validated state.
