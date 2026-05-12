#!/usr/bin/env bash
set -euo pipefail

TMUX_BIN="$(command -v tmux)"
DIR="$(cd "$(dirname "$0")/.." && pwd)"
CMD_FILE="$(mktemp)"
trap 'rm -f "$CMD_FILE"' EXIT

PALETTE="${1:-commands}"

# Raycast-style fixed width; height grows with content (capped at screen).
DEFAULT_W=90

WANT_H="$(bun "$DIR/src/cli.ts" "$PALETTE" --measure 2>/dev/null || echo 20)"
CH="$($TMUX_BIN display-message -p '#{client_height}' 2>/dev/null || echo 24)"
CW="$($TMUX_BIN display-message -p '#{client_width}' 2>/dev/null || echo 80)"

MAX_H=$(( CH - 2 ))  # leave 1 row of breathing room top + bottom
H=$(( WANT_H > MAX_H ? MAX_H : WANT_H ))
W=$(( DEFAULT_W > CW - 4 ? CW - 4 : DEFAULT_W ))

# Allow env override.
H="${TMUX_PALETTE_HEIGHT:-$H}"
W="${TMUX_PALETTE_WIDTH:-$W}"

# TMUX_PALETTE_BIN is set so { palette: "..." } subpalette chaining knows
# how to invoke ourselves — without it we'd assume "tmux-palette" is on PATH.
$TMUX_BIN display-popup -B -w "$W" -h "$H" -E \
  "TMUX_PALETTE_CMD='$CMD_FILE' TMUX_PALETTE_BIN='$0' exec bun '$DIR/src/cli.ts' $PALETTE"

if [ -s "$CMD_FILE" ]; then
  CMD="$(cat "$CMD_FILE")"
  case "$CMD" in
    tmux:*)
      # Don't propagate the dispatched command's exit status. tmux's `run-shell`
      # surfaces "script returned N" on non-zero exit, and some commands return
      # non-zero even when the prompt itself worked.
      eval "$TMUX_BIN ${CMD#tmux:}" || true
      ;;
    shell:*)
      eval "${CMD#shell:}" || true
      ;;
  esac
fi
exit 0
