/** Selection wins. An empty selection appends the editor body. */
export function textForDailyAppend(editorBody: string, selection: string): string {
  return selection.length > 0 ? selection : editorBody;
}

export function dailyAppendBlock(editorBody: string, selection: string, loggedAt: string): string {
  return `*   [Logged at ${loggedAt}]: ${textForDailyAppend(editorBody, selection)}`;
}
