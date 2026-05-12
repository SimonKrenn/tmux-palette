# tmux-palette

A Raycast-style command palette for tmux, built with [opentui](https://github.com/anomalyco/opentui).

Type a few letters, pick a command, hit enter — split a pane, jump to a window,
detach a session, whatever. Designed to be easy to extend with your own
palettes.

## Status

Early. The default `commands` palette works. Sub-palettes (`find-pane`,
`move-pane`) are planned, not yet ported.

## Install

```bash
git clone https://github.com/eduwass/tmux-palette ~/Sites/tmux-palette
cd ~/Sites/tmux-palette
bun install
```

Bind it to a tmux key in your `.tmux.conf`:

```tmux
bind p run-shell "~/Sites/tmux-palette/bin/tmux-palette.sh"
```

(Or `bind-key -n C-p ...` to make it a global keybinding without a prefix.)

Reload tmux config and hit your binding.

## Usage

- **Type** to filter. Multi-word search is supported (`split horiz`).
- **Up/Down arrows** or **Ctrl+P / Ctrl+N** to move selection.
- **Enter** to run the selected command.
- **Esc** to cancel.
- **Mouse** works too — click rows, scroll the wheel.

### Auto-aliases

Initials of multi-word titles are matched automatically. Type `nw` for "New
Window", `cs` for "Choose Session", `sh` for "Split Horizontal", etc. These
aren't displayed in the UI — they just work.

### Manual aliases

If you want a chip displayed in the row, add it explicitly:

```ts
{ icon: "", title: "Split Horizontal", aliases: ["sh"], action: { tmux: "..." } }
```

Now `sh` shows as a small badge next to the title.

## Extending

The default palette lives in `src/palettes/commands.ts`. To add your own
commands, edit that file (or fork). Each item is:

```ts
{
  icon: "󰍉",              // any nerd-font glyph
  title: "Find Pane",
  description?: "...",    // optional, dimmed text after title
  shortcut?: "Cmd+Shift+P", // optional, right-aligned label
  category?: "Panes",     // optional, groups items under a header
  aliases?: ["fp"],       // optional, visible chip + searchable
  action: { tmux: "..." } // see Actions below
}
```

### Actions

```ts
{ tmux: "split-window -h" }     // runs `tmux <cmd>` after the popup closes
{ shell: "echo hi" }            // runs a shell command after the popup closes
{ palette: "find-pane" }        // chains into another palette
{ run: (ctx) => { ... } }       // custom JS, runs in-process
```

`{ tmux }` is special: it dispatches *after* the popup closes, so interactive
tmux prompts (`confirm-before`, `command-prompt`) actually get keyboard
input. Without this, prompts hang because the popup still owns stdin.

## Themes

Built-in themes: `midnight-purple` (default), `dracula`, `tokyo-night`,
`minimal`. Set on the palette:

```ts
definePalette({ theme: "dracula", items: [...] })
```

Or define your own:

```ts
definePalette({
  theme: { bg: "#000", panel: "#111", selected: "#222", fg: "#fff", muted: "#888", accent: "#0ff" },
  items: [...]
})
```

## How it works (the trick)

The bash wrapper opens a `tmux display-popup` running the palette. When you
pick an item, the palette writes the encoded command to a tempfile and exits.
The wrapper *then* reads the tempfile and runs the command — *after* the
popup is gone. This matters because interactive tmux commands like
`confirm-before` need stdin, which is captured by the popup while it's open.

## License

MIT
