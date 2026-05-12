# tmux-palette

A Raycast-style command palette for tmux. Runs on [Bun](https://bun.sh),
zero runtime dependencies, snappy enough to feel like a native widget
(~30ms cold start).

Type a few letters, pick a command, hit enter — split a pane, jump to a window,
detach a session, whatever. Designed to be easy to extend with your own
palettes.

## Install

<details>
<summary><b>Hand off to an AI agent</b> (recommended — auto-detects your terminal theme)</summary>

<br/>

Paste the prompt below into [Claude Code](https://claude.com/claude-code), Cursor, Aider, or any AI coding agent. It will install the repo, set up your tmux binding, and (optionally) match the palette colors to your terminal theme.

````
You are helping a user install tmux-palette — a Raycast-style command palette for tmux. Repo: https://github.com/eduwass/tmux-palette

Follow steps in order. Confirm with the user before any change that modifies their files.

1. Prerequisites
- Run `bun --version`. If Bun is missing, point them to https://bun.sh/docs/installation and stop — do not auto-install.
- Run `tmux -V`. If lower than 3.4, warn that `display-popup -E` may not work, then proceed.

2. Clone and install
- Default path: `~/Sites/tmux-palette`. Ask the user if they want a different location.
- If the path already exists and contains the repo, run `git -C <path> pull` and skip cloning.
- Otherwise: `git clone https://github.com/eduwass/tmux-palette <path> && cd <path> && bun install`.

3. Bind it to a tmux key (required — the palette doesn't open without one)
- Suggested default: `prefix + C-p`. Ask the user if they want a different key.
- Append to `~/.tmux.conf` (create it if missing):
  `bind <key> run-shell "<absolute-path-to-clone>/bin/tmux-palette.sh"`
- Run `tmux source-file ~/.tmux.conf` to reload (or tell them to do it).

4. Match the palette to their terminal theme (optional but nice)
Ask: "Want the palette colors to match your terminal's theme?"

If yes, detect their terminal:
- Check $TERM_PROGRAM and $TERM. Common values: ghostty, iTerm.app, vscode, WezTerm, Apple_Terminal.
- Read the relevant config:
  - Ghostty:    ~/.config/ghostty/config
  - Alacritty:  ~/.config/alacritty/alacritty.toml (or .yml)
  - Kitty:      ~/.config/kitty/kitty.conf  (follow `include` lines)
  - WezTerm:    ~/.wezterm.lua or ~/.config/wezterm/wezterm.lua
  - iTerm2 / others: ask the user for hex codes; their configs are hard to parse.
- Extract: background → `bg`, foreground → `fg`, cursor color → `accent`, selection bg → `selected`. Derive `panel` (slightly lighter than bg) and `muted` (fg dimmed).
- Edit every `definePalette({ ... })` call in `src/palettes/*.ts` to include `theme: { bg, panel, selected, fg, muted, accent }`.
- Report the colors you picked.

5. Test
Tell the user to press their binding. Ask what they see.

6. Offer follow-ups
When it works, ask:
- "Want to change the binding?" — revisit step 3.
- "Want to add a custom command?" — show them the Item shape (icon, title, action, etc), ask what they want, append it to src/palettes/commands.ts. Action types: { tmux: "..." }, { shell: "..." }, { palette: "name" }, { run: (ctx) => ... }.
- "Want to explore the sub-palettes?" — they already have Find Pane and Move Pane to... in the default palette.

Constraints
- Do not push to git or modify files outside the user's home directory.
- Do not auto-install Bun or any other system package.
- If anything fails, stop and explain what went wrong.
````

</details>

<details>
<summary><b>Manual install</b></summary>

<br/>

```bash
git clone https://github.com/eduwass/tmux-palette ~/Sites/tmux-palette
cd ~/Sites/tmux-palette
bun install
```

Bind it to a tmux key in your `.tmux.conf`:

```tmux
bind C-p run-shell "~/Sites/tmux-palette/bin/tmux-palette.sh"
```

(Or `bind-key -n C-p ...` to make it a global keybinding without a prefix.)

Reload: `tmux source-file ~/.tmux.conf` and hit your binding.

</details>

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
