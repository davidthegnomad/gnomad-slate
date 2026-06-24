import { describe, it, expect } from "vitest";
import { renderMarkdownToHtml } from "./markdown";

describe("renderMarkdownToHtml", () => {
  it("renders basic markdown", () => {
    const html = renderMarkdownToHtml("# Hello\n\nWorld");
    expect(html).toContain("<h1");
    expect(html).toContain("World");
  });

  it("strips script tags from markdown output", () => {
    const html = renderMarkdownToHtml('# Hi\n\n<script>alert(1)</script>');
    expect(html).not.toContain("<script");
    expect(html).toContain("Hi");
  });
});
