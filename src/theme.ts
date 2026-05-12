import type { Theme } from "./types"

export const midnightPurple: Theme = {
  bg: "#1e1d40",
  panel: "#2d2b55",
  selected: "#504d7a",
  fg: "#ffffff",
  muted: "#a599e9",
  accent: "#fad000",
}

export const dracula: Theme = {
  bg: "#282a36",
  panel: "#21222c",
  selected: "#44475a",
  fg: "#f8f8f2",
  muted: "#6272a4",
  accent: "#bd93f9",
}

export const tokyoNight: Theme = {
  bg: "#1a1b26",
  panel: "#16161e",
  selected: "#283457",
  fg: "#c0caf5",
  muted: "#565f89",
  accent: "#7aa2f7",
}

export const minimal: Theme = {
  bg: "#000000",
  panel: "#0a0a0a",
  selected: "#1f1f1f",
  fg: "#ffffff",
  muted: "#808080",
  accent: "#ffffff",
}

export const themes: Record<string, Theme> = {
  "midnight-purple": midnightPurple,
  dracula,
  "tokyo-night": tokyoNight,
  minimal,
}

export function resolveTheme(theme: Theme | string | undefined): Theme {
  if (!theme) return midnightPurple
  if (typeof theme === "string") {
    const found = themes[theme]
    if (!found) throw new Error(`Unknown theme: ${theme}. Known: ${Object.keys(themes).join(", ")}`)
    return found
  }
  return theme
}
