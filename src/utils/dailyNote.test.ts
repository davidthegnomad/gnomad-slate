import { describe, expect, it } from "vitest";
import { dailyAppendBlock, textForDailyAppend } from "./dailyNote";

describe("daily note append text", () => {
  it("uses the selection when one exists", () => {
    expect(textForDailyAppend("whole note", "picked")).toBe("picked");
    expect(dailyAppendBlock("whole note", "picked", "1:00 PM")).toBe(
      "*   [Logged at 1:00 PM]: picked"
    );
  });

  it("uses the editor body when nothing is selected", () => {
    expect(textForDailyAppend("whole note", "")).toBe("whole note");
    expect(dailyAppendBlock("whole note", "", "1:00 PM")).toContain("whole note");
  });
});
