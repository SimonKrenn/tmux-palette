// Bundled themes. Curated to match the set at https://terminalcolors.com/.
// bg / fg come from Ghostty's official theme bundle; panel / selected /
// muted / accent are tuned per-theme so each pair (fg/panel, muted/panel,
// accent/panel, fg/selected, …) reads with sensible contrast.

import type { Theme } from "./types"

export type BundledTheme = { slug: string; name: string; theme: Theme }

export const bundledThemes: BundledTheme[] = [
  { slug: "apprentice", name: "Apprentice", theme: {
    bg: "#262626", panel: "#1c1c1c", selected: "#444444",
    fg: "#bcbcbc", muted: "#878787", accent: "#5f87af",
  }},
  { slug: "ayu-dark", name: "Ayu Dark", theme: {
    bg: "#0b0e14", panel: "#242e41", selected: "#3f5072",
    fg: "#bfbdb6", muted: "#98958a", accent: "#53bdfa",
  }},
  { slug: "ayu-light", name: "Ayu Light", theme: {
    bg: "#f8f9fa", panel: "#e6e9ed", selected: "#bdc6d0",
    fg: "#5c6166", muted: "#72797f", accent: "#8b5100",
  }},
  { slug: "ayu-mirage", name: "Ayu Mirage", theme: {
    bg: "#1f2430", panel: "#323a4d", selected: "#4e5a78",
    fg: "#cccac2", muted: "#a5a293", accent: "#6dcbfa",
  }},
  { slug: "catppuccin-frappe", name: "Catppuccin Frappé", theme: {
    bg: "#303446", panel: "#3f445c", selected: "#596082",
    fg: "#c6d0f5", muted: "#a4a9ba", accent: "#8caaee",
  }},
  { slug: "catppuccin-latte", name: "Catppuccin Latte", theme: {
    bg: "#eff1f5", panel: "#dadfe8", selected: "#aeb8cd",
    fg: "#4c4f69", muted: "#6a6d82", accent: "#1e66f5",
  }},
  { slug: "catppuccin-macchiato", name: "Catppuccin Macchiato", theme: {
    bg: "#24273a", panel: "#393d5b", selected: "#565d8b",
    fg: "#cad3f5", muted: "#a5a9bb", accent: "#8aadf4",
  }},
  { slug: "catppuccin-mocha", name: "Catppuccin Mocha", theme: {
    bg: "#1e1e2e", panel: "#383857", selected: "#5a5a8b",
    fg: "#cdd6f4", muted: "#a6a9b9", accent: "#89b4fa",
  }},
  { slug: "cobalt2", name: "Cobalt2", theme: {
    bg: "#132738", panel: "#254c6e", selected: "#3a78ac",
    fg: "#ffffff", muted: "#c8c8c8", accent: "#f0cc09",
  }},
  { slug: "deus", name: "Deus", theme: {
    bg: "#222222", panel: "#1a1a1a", selected: "#3a3a3a",
    fg: "#dadada", muted: "#8a8a8a", accent: "#87afaf",
  }},
  { slug: "dracula", name: "Dracula", theme: {
    bg: "#282a36", panel: "#45495d", selected: "#6a6f8f",
    fg: "#f8f8f2", muted: "#bdc3d8", accent: "#d6acff",
  }},
  { slug: "dracula-soft", name: "Dracula Soft", theme: {
    bg: "#2d2f3f", panel: "#363849", selected: "#534f6c",
    fg: "#f8f8f2", muted: "#9aa1bd", accent: "#bd93f9",
  }},
  { slug: "everforest-dark", name: "Everforest Dark", theme: {
    bg: "#1e2326", panel: "#31393e", selected: "#4d5a62",
    fg: "#d3c6aa", muted: "#97a28f", accent: "#7fbbb3",
  }},
  { slug: "everforest-light", name: "Everforest Light", theme: {
    bg: "#efebd4", panel: "#e8e2c0", selected: "#d0c481",
    fg: "#5c6a72", muted: "#6f7c68", accent: "#954307",
  }},
  { slug: "github-dark", name: "GitHub Dark", theme: {
    bg: "#101216", panel: "#1e2129", selected: "#363c4a",
    fg: "#8b949e", muted: "#707a85", accent: "#6ca4f8",
  }},
  { slug: "github-dark-colorblind", name: "GitHub Dark Colorblind", theme: {
    bg: "#0d1117", panel: "#293548", selected: "#455a7a",
    fg: "#c9d1d9", muted: "#9da3ac", accent: "#58a6ff",
  }},
  { slug: "github-dark-dimmed", name: "GitHub Dark Dimmed", theme: {
    bg: "#22272e", panel: "#303741", selected: "#495462",
    fg: "#adbac7", muted: "#8c96a3", accent: "#539bf5",
  }},
  { slug: "github-dark-high-contrast", name: "GitHub Dark High Contrast", theme: {
    bg: "#0a0c10", panel: "#343f53", selected: "#586a8e",
    fg: "#f0f3f6", muted: "#b7bec6", accent: "#71b7ff",
  }},
  { slug: "github-light", name: "GitHub Light", theme: {
    bg: "#ffffff", panel: "#e0e0e0", selected: "#acacac",
    fg: "#1f2328", muted: "#555e68", accent: "#0969da",
  }},
  { slug: "gotham", name: "Gotham", theme: {
    bg: "#0a0f14", panel: "#11181f", selected: "#1c4a5e",
    fg: "#98d1ce", muted: "#599cab", accent: "#5fb3b3",
  }},
  { slug: "gruvbox-dark", name: "Gruvbox Dark", theme: {
    bg: "#282828", panel: "#414141", selected: "#646464",
    fg: "#ebdbb2", muted: "#b7ada4", accent: "#8ec07c",
  }},
  { slug: "gruvbox-dark-hard", name: "Gruvbox Dark Hard", theme: {
    bg: "#1d2021", panel: "#393e40", selected: "#5a6467",
    fg: "#ebdbb2", muted: "#b6aca3", accent: "#83a598",
  }},
  { slug: "gruvbox-light", name: "Gruvbox Light", theme: {
    bg: "#fbf1c7", panel: "#f3d65d", selected: "#cca80f",
    fg: "#3c3836", muted: "#6b5f54", accent: "#076678",
  }},
  { slug: "iceberg-dark", name: "Iceberg Dark", theme: {
    bg: "#161821", panel: "#2f3447", selected: "#4e5576",
    fg: "#c6c8d1", muted: "#9a9db0", accent: "#84a0c6",
  }},
  { slug: "iceberg-light", name: "Iceberg Light", theme: {
    bg: "#e8e9ec", panel: "#cfd2d8", selected: "#a4a8b3",
    fg: "#33374c", muted: "#595e78", accent: "#2d539e",
  }},
  { slug: "jellybeans", name: "Jellybeans", theme: {
    bg: "#121212", panel: "#393939", selected: "#616161",
    fg: "#dedede", muted: "#aeaeae", accent: "#97bedc",
  }},
  { slug: "kanagawa-dragon", name: "Kanagawa Dragon", theme: {
    bg: "#181616", panel: "#373232", selected: "#5c5555",
    fg: "#c5c9c5", muted: "#99a099", accent: "#8ba4b0",
  }},
  { slug: "kanagawa-lotus", name: "Kanagawa Lotus", theme: {
    bg: "#f2ecbc", panel: "#e7dd86", selected: "#cbb927",
    fg: "#545464", muted: "#6d6d81", accent: "#4d699b",
  }},
  { slug: "kanagawa-wave", name: "Kanagawa Wave", theme: {
    bg: "#1f1f28", panel: "#3a3a4b", selected: "#5c5c77",
    fg: "#dcd7ba", muted: "#b4aa6c", accent: "#7e9cd8",
  }},
  { slug: "lucario", name: "Lucario", theme: {
    bg: "#2b3e50", panel: "#212d38", selected: "#3d566f",
    fg: "#f8f8f2", muted: "#5c98cd", accent: "#72c05d",
  }},
  { slug: "miasma", name: "Miasma", theme: {
    bg: "#222222", panel: "#373737", selected: "#565656",
    fg: "#c2c2b0", muted: "#9c9c7f", accent: "#c9a554",
  }},
  { slug: "moonfly", name: "Moonfly", theme: {
    bg: "#080808", panel: "#2c2c2c", selected: "#4f4f4f",
    fg: "#bdbdbd", muted: "#959595", accent: "#80a0ff",
  }},
  { slug: "nightfly", name: "Nightfly", theme: {
    bg: "#011627", panel: "#0a2236", selected: "#2c4e6e",
    fg: "#bdc1c6", muted: "#7e8c95", accent: "#82aaff",
  }},
  { slug: "nightfox", name: "Nightfox", theme: {
    bg: "#192330", panel: "#2b3c52", selected: "#435e80",
    fg: "#cdcecf", muted: "#a3a5a6", accent: "#719cd6",
  }},
  { slug: "dawnfox", name: "Dawnfox", theme: {
    bg: "#faf4ed", panel: "#f1e0cd", selected: "#deb688",
    fg: "#575279", muted: "#7369a9", accent: "#286983",
  }},
  { slug: "dayfox", name: "Dayfox", theme: {
    bg: "#f6f2ee", panel: "#e3d7ca", selected: "#c2a78b",
    fg: "#3d2b5a", muted: "#685f56", accent: "#2848a9",
  }},
  { slug: "duskfox", name: "Duskfox", theme: {
    bg: "#232136", panel: "#3f3b61", selected: "#635d99",
    fg: "#e0def4", muted: "#b1add1", accent: "#65b1cd",
  }},
  { slug: "night-owl", name: "Night Owl", theme: {
    bg: "#011627", panel: "#033c69", selected: "#0463af",
    fg: "#d6deeb", muted: "#9cb0cf", accent: "#82aaff",
  }},
  { slug: "night-owlish-light", name: "Night Owlish Light", theme: {
    bg: "#ffffff", panel: "#e5e5e5", selected: "#b7b7b7",
    fg: "#403f53", muted: "#686787", accent: "#403f53",
  }},
  { slug: "noctis", name: "Noctis", theme: {
    bg: "#052329", panel: "#0b2f37", selected: "#1d6776",
    fg: "#b3b9b8", muted: "#7ea2a8", accent: "#49ace9",
  }},
  { slug: "nord", name: "Nord", theme: {
    bg: "#2e3440", panel: "#3f4758", selected: "#5c677f",
    fg: "#d8dee9", muted: "#abb2c0", accent: "#88c0d0",
  }},
  { slug: "nord-light", name: "Nord Light", theme: {
    bg: "#e5e9f0", panel: "#ced6e3", selected: "#a2b0c9",
    fg: "#414858", muted: "#5b6780", accent: "#2d5764",
  }},
  { slug: "nordic", name: "Nordic", theme: {
    bg: "#2e3440", panel: "#262b35", selected: "#434c5e",
    fg: "#d8dee9", muted: "#7b88a1", accent: "#88c0d0",
  }},
  { slug: "one-dark", name: "One Dark", theme: {
    bg: "#21252b", panel: "#2f353d", selected: "#48505e",
    fg: "#abb2bf", muted: "#8691a3", accent: "#61afef",
  }},
  { slug: "one-light", name: "One Light", theme: {
    bg: "#f9f9f9", panel: "#dcdcdc", selected: "#ababab",
    fg: "#2a2c33", muted: "#5a5e6d", accent: "#2f5af3",
  }},
  { slug: "one-half-dark", name: "One Half Dark", theme: {
    bg: "#282c34", panel: "#3d4350", selected: "#5c6678",
    fg: "#dcdfe4", muted: "#abb1bf", accent: "#a3b3cc",
  }},
  { slug: "one-half-light", name: "One Half Light", theme: {
    bg: "#fafafa", panel: "#e0e0e0", selected: "#b2b2b2",
    fg: "#383a42", muted: "#626574", accent: "#a626a4",
  }},
  { slug: "panda", name: "Panda", theme: {
    bg: "#292a2b", panel: "#33363a", selected: "#4f5560",
    fg: "#e6e6e6", muted: "#8b91a0", accent: "#ff75b5",
  }},
  { slug: "posterpole", name: "Posterpole", theme: {
    bg: "#202020", panel: "#161616", selected: "#3a3a3a",
    fg: "#d0d0d0", muted: "#a0a0a0", accent: "#e35d5d",
  }},
  { slug: "rose-pine", name: "Rosé Pine", theme: {
    bg: "#191724", panel: "#3c3857", selected: "#645c8f",
    fg: "#e0def4", muted: "#b1aebf", accent: "#9ccfd8",
  }},
  { slug: "rose-pine-dawn", name: "Rosé Pine Dawn", theme: {
    bg: "#faf4ed", panel: "#f1e0cd", selected: "#deb688",
    fg: "#575279", muted: "#756f85", accent: "#b4637a",
  }},
  { slug: "rose-pine-moon", name: "Rosé Pine Moon", theme: {
    bg: "#232136", panel: "#3f3b61", selected: "#635d99",
    fg: "#e0def4", muted: "#b2afc0", accent: "#9ccfd8",
  }},
  { slug: "seoul256", name: "Seoul256", theme: {
    bg: "#4b4b4b", panel: "#575757", selected: "#717171",
    fg: "#dddddd", muted: "#b9b9b9", accent: "#97bdde",
  }},
  { slug: "seoul256-light", name: "Seoul256 Light", theme: {
    bg: "#e2e2e2", panel: "#d5d5d5", selected: "#b4b4b4",
    fg: "#555555", muted: "#6b6b6b", accent: "#006f89",
  }},
  { slug: "shades-of-purple", name: "Shades of Purple", theme: {
    bg: "#1e1d40", panel: "#2d2b55", selected: "#504d7a",
    fg: "#ffffff", muted: "#a599e9", accent: "#fad000",
  }},
  { slug: "solarized-dark", name: "Solarized Dark", theme: {
    bg: "#002b36", panel: "#00333f", selected: "#00485b",
    fg: "#839496", muted: "#4a8897", accent: "#268bd2",
  }},
  { slug: "solarized-light", name: "Solarized Light", theme: {
    bg: "#fdf6e3", panel: "#fbebc2", selected: "#f3c852",
    fg: "#657b83", muted: "#00647e", accent: "#268bd2",
  }},
  { slug: "sonokai", name: "Sonokai", theme: {
    bg: "#2c2e34", panel: "#42464f", selected: "#636875",
    fg: "#e2e2e3", muted: "#b1b5bc", accent: "#76cce0",
  }},
  { slug: "srcery", name: "Srcery", theme: {
    bg: "#1c1b19", panel: "#423f3b", selected: "#6b675f",
    fg: "#fce8c3", muted: "#bfb6af", accent: "#68a8e4",
  }},
  { slug: "tender", name: "Tender", theme: {
    bg: "#282828", panel: "#1e1e1e", selected: "#4a4a4a",
    fg: "#eeeeee", muted: "#888888", accent: "#b3deef",
  }},
  { slug: "tokyo-night", name: "Tokyo Night", theme: {
    bg: "#1a1b26", panel: "#34354b", selected: "#53567a",
    fg: "#c0caf5", muted: "#99a0bf", accent: "#7aa2f7",
  }},
  { slug: "tokyo-night-day", name: "Tokyo Night Day", theme: {
    bg: "#e1e2e7", panel: "#d8d9df", selected: "#babdc8",
    fg: "#3760bf", muted: "#4a5178", accent: "#9854f1",
  }},
  { slug: "tokyo-night-moon", name: "Tokyo Night Moon", theme: {
    bg: "#222436", panel: "#383c59", selected: "#575c8a",
    fg: "#c8d3f5", muted: "#a2a6c8", accent: "#82aaff",
  }},
  { slug: "tokyo-night-storm", name: "Tokyo Night Storm", theme: {
    bg: "#24283b", panel: "#363b58", selected: "#515a84",
    fg: "#c0caf5", muted: "#9ca1bd", accent: "#7aa2f7",
  }},
  { slug: "tomorrow", name: "Tomorrow", theme: {
    bg: "#ffffff", panel: "#e7e7e7", selected: "#bcbcbc",
    fg: "#4d4d4c", muted: "#70706f", accent: "#4271ae",
  }},
  { slug: "tomorrow-night", name: "Tomorrow Night", theme: {
    bg: "#1d1f21", panel: "#34383b", selected: "#53585e",
    fg: "#c5c8c6", muted: "#9aa09c", accent: "#81a2be",
  }},
  { slug: "tomorrow-night-blue", name: "Tomorrow Night Blue", theme: {
    bg: "#002451", panel: "#00459c", selected: "#006bf2",
    fg: "#ffffff", muted: "#c7c7c7", accent: "#bbdaff",
  }},
  { slug: "tomorrow-night-bright", name: "Tomorrow Night Bright", theme: {
    bg: "#000000", panel: "#393939", selected: "#656565",
    fg: "#eaeaea", muted: "#b6b6b6", accent: "#7aa6da",
  }},
  { slug: "tomorrow-night-eighties", name: "Tomorrow Night Eighties", theme: {
    bg: "#2d2d2d", panel: "#3f3f3f", selected: "#5d5d5d",
    fg: "#cccccc", muted: "#a4a4a4", accent: "#cc99cc",
  }},
  { slug: "zenbones", name: "Zenbones", theme: {
    bg: "#f0edec", panel: "#dad3d1", selected: "#b4a6a1",
    fg: "#2c363c", muted: "#6b5c54", accent: "#286486",
  }},
  { slug: "zenbones-dark", name: "Zenbones Dark", theme: {
    bg: "#1c1917", panel: "#36312d", selected: "#595049",
    fg: "#b4bdc3", muted: "#9f938c", accent: "#6099c0",
  }},
  { slug: "zenbones-light", name: "Zenbones Light", theme: {
    bg: "#f0edec", panel: "#dad3d1", selected: "#b4a6a1",
    fg: "#2c363c", muted: "#6b5c54", accent: "#286486",
  }},
  { slug: "minimal", name: "Minimal", theme: {
    bg: "#000000", panel: "#141414", selected: "#3a3a3a",
    fg: "#ffffff", muted: "#9a9a9a", accent: "#ffffff",
  }},
]

export const bundledThemeMap: Record<string, Theme> =
  Object.fromEntries(bundledThemes.map((t) => [t.slug, t.theme]))
