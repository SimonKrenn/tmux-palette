import { spawnSync } from "node:child_process"
import { definePalette } from "../palette"
import type { Item, RenderItemCtx } from "../types"

function tmux(args: string[]): string {
  const r = spawnSync("tmux", args, { stdio: ["ignore", "pipe", "pipe"] })
  return r.stdout?.toString().trimEnd() ?? ""
}

function detectAgent(command: string, title: string): string {
  const direct = new Set(["claude", "codex", "aider", "cursor-agent", "opencode", "gemini", "ollama"])
  if (direct.has(command)) return command
  if (title.startsWith("OC | ") || title.startsWith("OC|")) return "opencode"
  if (/^\s*[*✳⠂⠐⠁⠉⠙⠹⠸⠼⠴⠦⠧⠇⠏]\s/.test(title)) return "claude"
  return ""
}

type Pane = {
  session: string
  windowIndex: string
  paneIndex: string
  windowName: string
  paneTitle: string
  command: string
  path: string
  agent: string
  paneActive: boolean
  windowActive: boolean
  isCurrent: boolean
  target: string
}

type ItemData =
  | { kind: "session"; session: string; count: number; path: string; isCurrent: boolean }
  | { kind: "window"; session: string; windowIndex: string; windowName: string; treePrefix: string }
  | { kind: "pane"; pane: Pane; treePrefix: string }

function fetchPanes(): { panes: Pane[]; currentPane: string; currentSession: string } {
  const currentPane = tmux(["display-message", "-p", "#{session_name}:#{window_index}.#{pane_index}"])
  const currentSession = currentPane.split(":")[0] ?? ""

  const lines = tmux([
    "list-panes", "-a",
    "-F", [
      "#{session_name}",
      "#{window_index}",
      "#{pane_index}",
      "#{window_name}",
      "#{pane_title}",
      "#{pane_current_command}",
      "#{pane_current_path}",
      "#{pane_active}",
      "#{window_active}",
    ].join("\t"),
  ]).split("\n").filter(Boolean)

  const panes: Pane[] = []
  for (const line of lines) {
    const [session, windowIndex, paneIndex, windowName, paneTitle, command, path, paneActive, windowActive] = line.split("\t")
    if (!session || !windowIndex || !paneIndex) continue
    const target = `${session}:${windowIndex}.${paneIndex}`
    const title = paneTitle || `pane${paneIndex}`
    panes.push({
      session,
      windowIndex,
      paneIndex,
      windowName: windowName || `window${windowIndex}`,
      paneTitle: title,
      command: command || "",
      path: path || "",
      agent: detectAgent(command || "", title),
      paneActive: paneActive === "1",
      windowActive: windowActive === "1",
      isCurrent: target === currentPane,
      target,
    })
  }
  return { panes, currentPane, currentSession }
}

function buildItems(): Item[] {
  const { panes, currentSession } = fetchPanes()

  const sessionOrder: string[] = []
  const bySession = new Map<string, Map<string, { windowName: string; panes: Pane[] }>>()
  for (const p of panes) {
    if (!bySession.has(p.session)) {
      bySession.set(p.session, new Map())
      sessionOrder.push(p.session)
    }
    const ws = bySession.get(p.session)!
    if (!ws.has(p.windowIndex)) ws.set(p.windowIndex, { windowName: p.windowName, panes: [] })
    ws.get(p.windowIndex)!.panes.push(p)
  }

  const items: Item[] = []
  for (const session of sessionOrder) {
    const windows = [...bySession.get(session)!.entries()]
    const allInSession = windows.flatMap(([, w]) => w.panes)
    const focused = allInSession.find((p) => p.paneActive && p.windowActive) || allInSession[0]
    items.push({
      title: session,
      action: { tmux: `switch-client -t '${session}'` },
      selectable: false,
      data: {
        kind: "session",
        session,
        count: allInSession.length,
        path: focused?.path ?? "",
        isCurrent: session === currentSession,
      } satisfies ItemData,
    })

    for (let wi = 0; wi < windows.length; wi++) {
      const entry = windows[wi]!
      const [windowIndex, w] = entry
      const isLastWin = wi === windows.length - 1
      const winPrefix = `  ${isLastWin ? "└─" : "├─"} `

      if (w.panes.length === 1) {
        const p = w.panes[0]!
        items.push({
          title: p.paneTitle,
          action: {
            tmux: `select-pane -t '${p.target}' \\; select-window -t '${p.session}:${p.windowIndex}' \\; switch-client -t '${p.session}'`,
          },
          data: { kind: "pane", pane: p, treePrefix: winPrefix } satisfies ItemData,
        })
        continue
      }

      items.push({
        title: w.windowName,
        action: { tmux: `select-window -t '${session}:${windowIndex}' \\; switch-client -t '${session}'` },
        selectable: false,
        data: {
          kind: "window",
          session,
          windowIndex,
          windowName: w.windowName,
          treePrefix: winPrefix,
        } satisfies ItemData,
      })

      const panePrefixBase = isLastWin ? "      " : "  │   "
      for (let pi = 0; pi < w.panes.length; pi++) {
        const p = w.panes[pi]!
        const isLastPane = pi === w.panes.length - 1
        items.push({
          title: p.paneTitle,
          action: {
            tmux: `select-pane -t '${p.target}' \\; select-window -t '${p.session}:${p.windowIndex}' \\; switch-client -t '${p.session}'`,
          },
          data: {
            kind: "pane",
            pane: p,
            treePrefix: panePrefixBase + (isLastPane ? "└─ " : "├─ "),
          } satisfies ItemData,
        })
      }
    }
  }

  return items
}

function shortenPath(path: string): string {
  const home = process.env.HOME || ""
  return home && path.startsWith(home) ? `~${path.slice(home.length)}` : path
}

function renderItem(item: Item, ctx: RenderItemCtx): string {
  const { colors, active, width } = ctx
  const data = item.data as ItemData
  const rowBg = active ? colors.selected : colors.panel

  if (data.kind === "session") {
    const marker = data.isCurrent ? `${colors.accent}▶ ${colors.reset}${rowBg}` : "  "
    const name = `${colors.accent}${colors.bold}${data.session}${colors.reset}${rowBg}`
    const count = `${colors.muted} (${data.count})${colors.reset}${rowBg}`
    const path = data.path ? `  ${colors.muted}${shortenPath(data.path)}${colors.reset}${rowBg}` : ""
    return `${marker}${name}${count}${path}`
  }

  if (data.kind === "window") {
    const titleStyle = active ? colors.bold + colors.fg : colors.fg
    return `${colors.muted}${data.treePrefix}${colors.reset}${rowBg}${titleStyle}${data.windowName}${colors.reset}${rowBg}`
  }

  const p = data.pane
  let markerColor: string
  if (p.isCurrent) markerColor = colors.accent
  else if (p.paneActive) markerColor = "\x1b[38;2;166;227;161m"
  else markerColor = colors.muted
  const markerChar = p.isCurrent ? "▶" : p.paneActive ? "●" : "○"
  const titleStyle = p.isCurrent ? colors.muted : active ? colors.bold + colors.fg : colors.fg

  let left = `${colors.muted}${data.treePrefix}${colors.reset}${rowBg}${markerColor}${markerChar}${colors.reset}${rowBg} ${titleStyle}${p.paneTitle}${colors.reset}${rowBg}`
  let leftPlainW = data.treePrefix.length + 1 + 1 + p.paneTitle.length

  if (p.agent) {
    left += `  ${colors.muted}${p.agent}${colors.reset}${rowBg}`
    leftPlainW += 2 + p.agent.length
  }

  const rightText = `${p.windowIndex}.${p.paneIndex}`
  const right = `${colors.muted}${rightText}${colors.reset}${rowBg}`
  const gap = Math.max(1, width - leftPlainW - rightText.length)
  return `${left}${" ".repeat(gap)}${right}`
}

function filterTree(items: Item[], query: string): Item[] {
  const parts = query.toLowerCase().split(/\s+/).filter(Boolean)
  if (!parts.length) return items

  const okSessions = new Set<string>()
  const okWindows = new Set<string>()
  const okPanes = new Set<string>()

  for (const item of items) {
    const data = item.data as ItemData
    if (data.kind !== "pane") continue
    const p = data.pane
    const haystack = [
      p.session, p.windowName, p.paneTitle, p.command, p.path, p.target, p.agent,
    ].filter(Boolean).join(" ").toLowerCase()
    if (parts.every((part) => haystack.includes(part))) {
      okPanes.add(p.target)
      okSessions.add(p.session)
      okWindows.add(`${p.session}:${p.windowIndex}`)
    }
  }

  return items.filter((item) => {
    const data = item.data as ItemData
    if (data.kind === "session") return okSessions.has(data.session)
    if (data.kind === "window") return okWindows.has(`${data.session}:${data.windowIndex}`)
    return okPanes.has(data.pane.target)
  })
}

export const findPane = definePalette({
  title: "Find Pane",
  grouped: false,
  emptyText: "No panes",
  items: buildItems,
  renderItem,
  filter: filterTree,
})
