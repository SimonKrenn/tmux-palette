import { describe, expect, test } from "bun:test"
import { defaultFilter, multiFuzzyScore } from "../src/fuzzy"
import type { Item } from "../src/types"

const noop = { shell: ":" } as const

describe("multiFuzzyScore", () => {
  test("requires every query part to match", () => {
    expect(multiFuzzyScore("split horizontal pane", ["split", "pane"])).toBeGreaterThan(0)
    expect(multiFuzzyScore("split horizontal pane", ["split", "window"])).toBe(0)
  })
})

describe("defaultFilter", () => {
  const items: Item[] = [
    { title: "Split Horizontal", category: "Panes", action: noop },
    { title: "New Window", category: "Windows", action: noop },
    { title: "Choose Session", aliases: ["sessions"], action: noop },
  ]

  test("matches title initials through auto aliases", () => {
    expect(defaultFilter(items, "sh").map((i) => i.title)).toEqual(["Split Horizontal"])
  })

  test("matches explicit aliases", () => {
    expect(defaultFilter(items, "sessions").map((i) => i.title)).toEqual(["Choose Session"])
  })
})
