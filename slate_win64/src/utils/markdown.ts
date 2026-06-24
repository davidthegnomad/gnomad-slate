import { marked } from "marked";
import DOMPurify from "dompurify";

export function renderMarkdownToHtml(md: string): string {
  const raw = marked.parse(md, { breaks: true, gfm: true }) as string;
  return DOMPurify.sanitize(raw, {
    USE_PROFILES: { html: true },
    FORBID_TAGS: ["script", "iframe", "object", "embed", "form"],
    FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover"],
  });
}
