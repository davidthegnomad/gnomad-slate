import { useEffect, useMemo, useState } from "react";
import { renderMarkdownToHtml } from "../utils/markdown";
import { PREVIEW_DEBOUNCE_MS } from "../constants/timing";
import type { EditorViewMode } from "../types/app";

/**
 * Debounces markdown → HTML conversion so typing in split/preview mode
 * does not re-parse on every keystroke.
 */
export function useDebouncedPreview(editorContent: string, editorView: EditorViewMode) {
  const [debouncedContent, setDebouncedContent] = useState("");

  useEffect(() => {
    if (editorView === "edit") {
      return;
    }
    const timer = setTimeout(() => setDebouncedContent(editorContent), PREVIEW_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [editorContent, editorView]);

  const previewHtml = useMemo(() => {
    if (editorView === "edit") {
      return "";
    }
    return renderMarkdownToHtml(debouncedContent);
  }, [debouncedContent, editorView]);

  return previewHtml;
}
