/** Normalize Tauri IPC errors for user-facing messages. */
export function invokeErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}
