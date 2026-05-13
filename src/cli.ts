import { runPalette } from "./palette"
import { commands } from "./palettes/commands"
import { findPane } from "./palettes/find-pane"
import { movePane } from "./palettes/move-pane"
import type { Item, PaletteDef } from "./types"
import { userCommands, userSizing } from "./userConfig"

const DEFAULT_WIDTH = 90
const DEFAULT_MAX_HEIGHT = 24
const DEFAULT_PAD_X = 3

const palettes: Record<string, PaletteDef> = {
  commands,
  "find-pane": findPane,
  "move-pane": movePane,
}

const name = process.argv[2] || "commands"
let def = palettes[name]

if (!def) {
  console.error(`Unknown palette: ${name}. Available: ${Object.keys(palettes).join(", ")}`)
  process.exit(1)
}

// Append user-defined items to the commands palette (~/.config/tmux-palette/commands.json).
if (name === "commands") {
  const extras = userCommands()
  if (extras.length) {
    const baseItems: Item[] = typeof def.items === "function" ? await def.items() : def.items
    def = { ...def, items: [...baseItems, ...extras] }
  }
}

// Measure mode: print "<rows>\t<width>\t<padX>" so the bash wrapper
// can size the popup. Defaults are applied here so sizing.json
// overrides flow through naturally.
if (process.argv.includes("--measure")) {
  const items: Item[] = typeof def.items === "function" ? await def.items() : def.items
  const cats = new Set(items.map((i) => i.category).filter((c): c is string => Boolean(c))).size
  // chrome: top pad (1) + header (1) + search (1) + spacer (1) + footer spacer (1) + footer (1) + bottom pad (1) = 7
  const sizing = userSizing()
  const maxHeight = sizing.maxHeight ?? DEFAULT_MAX_HEIGHT
  const width = sizing.width ?? DEFAULT_WIDTH
  const padX = sizing.padX ?? DEFAULT_PAD_X
  const desired = items.length + cats + 7
  const rows = Math.min(desired, maxHeight)
  console.log(`${rows}\t${width}\t${padX}`)
  process.exit(0)
}

await runPalette(def)
