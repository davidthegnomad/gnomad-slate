import { describe, it, expect } from "vitest";
import { resolveTheme } from "./theme";

describe("resolveTheme", () => {
  it('returns "dark" for system when prefersDark is true', () => {
    expect(resolveTheme("system", true)).toBe("dark");
  });

  it('returns "light" for system when prefersDark is false', () => {
    expect(resolveTheme("system", false)).toBe("light");
  });

  it("returns explicit values", () => {
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });
});
