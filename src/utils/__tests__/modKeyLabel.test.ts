import { describe, it, expect } from "vitest";
import { formatShortcut, modKeyLabel, modShortcut } from "../modKeyLabel";

describe("modKeyLabel", () => {
  it("returns Ctrl or ⌘", () => {
    expect(["Ctrl", "⌘"]).toContain(modKeyLabel());
  });

  it("formats palette-style shortcuts", () => {
    const formatted = formatShortcut("Cmd+Shift+V");
    expect(formatted).toMatch(/Shift/);
  });

  it("builds modShortcut strings", () => {
    expect(modShortcut("P", { shift: true })).toContain("Shift");
  });
});
