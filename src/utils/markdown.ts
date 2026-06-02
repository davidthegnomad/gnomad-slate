import { marked } from "marked";

marked.setOptions({
  gfm: true,
  breaks: true,
});

/** Strip script tags from rendered HTML (local preview hardening). */
export function sanitizePreviewHtml(html: string): string {
  return html
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, "")
    .replace(/\son\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "");
}

export function renderMarkdownToHtml(markdown: string): string {
  const raw = marked.parse(markdown, { async: false }) as string;
  return sanitizePreviewHtml(raw);
}
