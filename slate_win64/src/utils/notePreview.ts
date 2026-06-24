/** Strip ephemeral marker for sidebar preview snippets. */
export function getPreviewText(content: string, isEphemeral: boolean): string {
  let text = content;
  if (isEphemeral && text.startsWith("<!-- ephemeral -->\n")) {
    text = text.replace("<!-- ephemeral -->\n", "");
  }
  const trimmed = text.trim();
  return trimmed.length > 0 ? trimmed.substring(0, 40) : "Empty note";
}
