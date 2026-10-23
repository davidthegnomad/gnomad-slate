import { describe, expect, it } from "vitest";
import { dailyAppendBlock, textForDailyAppend } from "./dailyNote";

describe("daily note append text", () => {
  it("uses the selection when one exists", () => {
    expect(textForDailyAppend("whole note", "picked")).toBe("picked");
  });

  it("uses the editor body when nothing is selected", () => {
    expect(dailyAppendBlock("whole note", "", "1:00 PM")).toContain("whole note");
  });
});
