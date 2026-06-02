import { describe, expect, it } from "vitest";
import { renderMarkdownToHtml, sanitizePreviewHtml } from "./markdown";

describe("markdown preview", () => {
  it("renders headings", () => {
    const html = renderMarkdownToHtml("# Hello");
    expect(html).toContain("<h1");
    expect(html).toContain("Hello");
  });

  it("strips script tags from preview html", () => {
    const dirty = '<p>ok</p><script>alert(1)</script>';
    const clean = sanitizePreviewHtml(dirty);
    expect(clean).not.toContain("<script");
    expect(clean).toContain("ok");
  });
});
