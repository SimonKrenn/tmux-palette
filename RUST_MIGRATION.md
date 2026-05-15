# Rust Migration State

This document keeps the Rust port resumable across sessions. Update it whenever a migration milestone lands.

## Goal

Port `tmux-palette` from Bun/TypeScript to Rust while preserving user-facing behavior, especially tmux popup launch semantics, delayed action dispatch, JSON config compatibility, and rendering/input parity.

## Current Status

- Rust implementation lives under `crates/tmux-palette/`.
- TypeScript/Bun remains the production implementation.
- Current Rust milestone includes pure parity modules and initial config/palette loading:
  - model/action types
  - text width/truncation/aliases
  - fuzzy filtering
  - render helper logic
  - command palette invariants
  - action encoding
  - config JSON loading
  - active theme resolution
  - custom palette loading
  - plugin command parsing without timeout
  - builtin tmux palette helpers (`commands`, `find-pane`, `move-pane`)
  - theme switcher palette item generation and active-theme file writing
  - initial TUI state machine and terminal IO (`crossterm` raw mode, keyboard/mouse navigation, filtering, nested palettes, delayed dispatch)
  - opt-in Rust launcher path via `@palette-runtime rust` / `TMUX_PALETTE_RUNTIME=rust`
  - improved Rust TUI parity for find-pane tree rows and common edit/navigation keys (`Ctrl+P/N/W`, word motion)

## Compatibility Requirements

- Preserve config files under `~/.config/tmux-palette/`.
- Preserve action JSON shapes: `tmux`, `shell`, `popup`, `palette`.
- Preserve delayed dispatch: selected action is written to a command file and executed only after popup closes.
- Preserve `--measure` tab-separated output before replacing the shell launcher.
- Keep JSON parsing permissive: unknown fields should not break user configs.

## Next Phases

1. Finish pure module parity and tests. ✅
2. Add config/theme loading with user overrides. ✅
3. Port custom palettes and plugin command execution. ✅
   - Rust helper currently uses `sh -c` without a timeout; add one if it can be done cleanly without extra deps.
4. Port dynamic tmux palettes (`find-pane`, `move-pane`). ✅ initial helpers/items ported; full tree rendering parity still needs TUI work.
5. Port theme switcher palette. ✅ item generation/save helper ported; live preview/back navigation depends on TUI.
6. Port TUI state machine and terminal IO. ✅ initial runtime ported; find-pane tree rendering and common edit keys ported; remaining parity gaps include selection-range editing polish and broader dogfood validation.
7. Add Rust launcher behind an opt-in flag. ✅ `@palette-runtime rust` keeps Bun as default while exercising the Rust binary/cargo path.
8. Switch default launcher after parity validation.

## Validation Commands

```bash
cargo test
bun test
bun run typecheck
```

## Commit Policy

Make small milestone commits when the Rust port reaches a coherent, validated state.
