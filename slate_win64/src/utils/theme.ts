export type ThemePreference = "light" | "dark" | "system";

export function resolveTheme(
  preference: ThemePreference,
  prefersDark: boolean
): "light" | "dark" {
  if (preference === "system") {
    return prefersDark ? "dark" : "light";
  }
  return preference;
}
