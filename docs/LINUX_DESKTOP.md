# Gnomad Slate — Linux Desktop

**Channel:** alpha (see About dialog)  
**Target:** Nobara / Fedora, Wayland-first

---

## Build

```bash
bash scripts/install-linux-deps.sh   # Fedora/Nobara (dnf)
npm install
npm run tauri:dev                    # dev + HMR
npm run tauri:build                  # .rpm, .deb, AppImage
```

**Debian/Ubuntu deps:**

```bash
sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev \
  libappindicator3-dev librsvg2-dev patchelf
```

Requires **WebKit2GTK 4.1** and **libsoup3** (Tauri 2).

---

## Launcher behavior

| Action | How |
|--------|-----|
| Summon / hide | `Super+Shift+Space` (global shortcut) |
| Show when hidden | System tray icon (left-click) or **Window → Show** |
| Quit | **File → Quit** (`Ctrl+Q`), tray **Quit**, or close window → save if needed → `confirm_quit` |
| Close (hide) | **File → Close** (`Ctrl+W`), window X, or **Window → Hide** — app stays in tray for summon |

Linux does **not** use macOS hide-on-blur. The app stays visible until you hide it or quit.

---

## Data location

```
~/.local/share/gnomad-slate/
├── notes/
├── vault.enc
└── archive/
```

Settings (theme, clipboard filter, line numbers) persist via Tauri plugin-store (`settings.json` in app config dir).

---

## Wayland notes

- Global shortcuts depend on the compositor; `Super+Shift+Space` avoids conflicts with KDE/GNOME launchers.
- Use the **tray icon** if the shortcut fails to register.
- In-app dialogs replace `alert()` so WebKitGTK focus is not stolen.

---

## Binary & package

- Installed binary: `gnomad-slate` (not `tauri-app`)
- RPM/DEB include `.desktop` entry with `StartupWMClass=gnomad-slate`

---

## Tests

```bash
npm test
cd src-tauri && cargo test
```

---

## Support

**david@gnomadstudio.org** · [gnomadstudio.org](https://gnomadstudio.org)

See also [BUILD_PLATFORMS.md](BUILD_PLATFORMS.md) and review artifacts in `reviews/`.
