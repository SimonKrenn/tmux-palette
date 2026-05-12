export type Action =
  | { tmux: string }
  | { shell: string }
  | { palette: string }
  | { run: (ctx: ActionContext) => void | Promise<void> }

export interface ActionContext {
  readonly cmdFile: string | undefined
}

export type Item = {
  icon?: string
  title: string
  description?: string
  shortcut?: string
  category?: string
  aliases?: string[]
  action: Action
}

export type Theme = {
  bg: string
  panel: string
  selected: string
  fg: string
  muted: string
  accent: string
}

export type PaletteDef = {
  title?: string
  items: Item[] | (() => Item[] | Promise<Item[]>)
  theme?: Theme | string
  grouped?: boolean
  emptyText?: string
}
