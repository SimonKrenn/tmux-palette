# Changelog

## v0.1.1 - 2026-05-13

- Fix: New Session command now switches you to the new session (was silently creating it in the background and leaving you on the current one).
- Fix: Wrapper script works on macOS's bash 3.2.
- Find Pane: cursor starts on the current pane; other panes render muted so the current one reads as the visual anchor.

## v0.1.0 - 2026-05-13

Initial public release.

- Command palette for tmux panes, windows, sessions, and config reloads.
- Nested palettes for finding panes, moving panes, and switching themes.
- Custom user config under `~/.config/tmux-palette/`.
- Custom commands via `commands.json` and hidden built-ins via `hidden.json`.
- Custom palettes from JSON files, built-in items, categories, or shell commands.
- Plugin-style command sources that emit JSON or one item per line.
- Popup actions for terminal tools like `htop`, `btop`, `lazygit`, logs, and `fzf` scripts.
- Curated built-in themes with live preview and support for custom themes.
- Mobile/narrow-terminal fullscreen mode and configurable popup sizing/borders.
- TPM and manual install paths, plus optional guided onboarding prompt.
- Example palettes for GitHub PRs, GitHub Actions, git branches, Docker logs, npm scripts, and file picking.
- CI coverage for Bun tests, TypeScript, Fallow dead-code, and Fallow duplication checks.
