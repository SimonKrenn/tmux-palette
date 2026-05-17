import { readFileSync } from "node:fs"

export const CONFIG_DIR =
  `${process.env.XDG_CONFIG_HOME ?? `${process.env.HOME ?? ""}/.config`}/tmux-palette`

export function loadJSON<T>(name: string, fallback: T): T {
  try {
    const raw = readFileSync(`${CONFIG_DIR}/${name}.json`, "utf8")
    return JSON.parse(raw) ?? fallback
  } catch {
    return fallback
  }
}
