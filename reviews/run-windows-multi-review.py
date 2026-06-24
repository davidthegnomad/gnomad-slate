#!/usr/bin/env python3
"""Multi-model Windows code review for gnomad-slate (slate_win64 folder)."""
from __future__ import annotations

import argparse
import os
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
WIN_ROOT = PROJECT / "slate_win64"
OUT_DIR = PROJECT / "reviews"
ENV_PATH = PROJECT / ".env"
NIM_BASE_URL = "https://integrate.api.nvidia.com/v1"

GLM_LABEL = "glm-5.1"
GLM_MODEL = "z-ai/glm-5.1"
NEMOTRON_LABEL = "nemotron-super"
NEMOTRON_MODEL = "nvidia/nemotron-3-super-120b-a12b"
SYNTH_MODEL = "deepseek-ai/deepseek-v4-flash"

REVIEWERS = [
    (GLM_LABEL, GLM_MODEL),
    (NEMOTRON_LABEL, NEMOTRON_MODEL),
]

RUST_FILES = [
    "src-tauri/src/lib.rs",
    "src-tauri/src/main.rs",
    "src-tauri/src/app_state.rs",
    "src-tauri/src/window.rs",
    "src-tauri/src/vault_io.rs",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/commands/app.rs",
    "src-tauri/src/commands/notes.rs",
    "src-tauri/src/commands/vault.rs",
    "src-tauri/src/commands/clipboard.rs",
    "src-tauri/src/menu/mod.rs",
    "src-tauri/src/menu/windows/mod.rs",
    "src-tauri/src/menu/windows/definition.rs",
    "src-tauri/src/menu/windows/events.rs",
    "src-tauri/src/windows_tray.rs",
    "src-tauri/src/windows_clipboard.rs",
    "src-tauri/src/shortcuts.rs",
    "src-tauri/src/clipboard.rs",
    "src-tauri/src/clipboard_filter.rs",
    "src-tauri/src/crypto.rs",
    "src-tauri/src/vault_session.rs",
    "src-tauri/src/unlock_limiter.rs",
    "src-tauri/src/filesystem.rs",
    "src-tauri/src/biometrics.rs",
    "src-tauri/src/logging.rs",
    "src-tauri/tauri.conf.json",
    "src-tauri/capabilities/default.json",
    "src-tauri/Cargo.toml",
]

FRONTEND_FILES = [
    "src/App.tsx",
    "src/App.css",
    "src/commands/paletteCommands.ts",
    "src/commands/executePaletteCommand.ts",
    "src/components/CommandPalette.tsx",
    "src/components/ClipboardCapsule.tsx",
    "src/components/VaultPanel.tsx",
    "src/hooks/useCommandPalette.ts",
    "src/hooks/useClipboardCapsule.ts",
    "src/hooks/useGlobalKeybindings.ts",
    "src/hooks/useNoteEditor.ts",
    "src/hooks/useVault.ts",
    "src/hooks/useAppSettings.ts",
    "src/hooks/useWindowClose.ts",
    "src/hooks/useDebouncedPreview.ts",
    "src/hooks/useDesktopMenuBridge.ts",
    "src/components/HelpDialog.tsx",
    "src/components/AppMessageDialog.tsx",
    "src/components/AboutDialog.tsx",
    "src/content/windowsHelp.ts",
    "src/types/app.ts",
    "src/constants/timing.ts",
    "src/utils/platform.ts",
    "src/utils/ipc.ts",
    "src/utils/settings.ts",
    "src/utils/modKeyLabel.ts",
    "package.json",
    "vite.config.ts",
]

MAX_SLICE = 48_000


def load_env() -> None:
    if not ENV_PATH.is_file():
        return
    for line in ENV_PATH.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def bundle(files: list[str]) -> str:
    parts: list[str] = []
    for rel in files:
        path = WIN_ROOT / rel
        if not path.is_file():
            parts.append(f"### slate_win64/{rel}\n(missing)\n")
            continue
        text = path.read_text(errors="replace")
        parts.append(f"### slate_win64/{rel}\n```\n{text}\n```\n")
    body = "\n".join(parts)
    if len(body) > MAX_SLICE:
        body = body[:MAX_SLICE] + "\n\n[... truncated for token budget ...]\n"
    return body


def review_prompt(model_label: str, slice_name: str, code: str) -> str:
    return f"""You are a **senior desktop engineer** reviewing **Gnomad Slate** Windows port code in `slate_win64/` (Tauri 2 + React 19).

**Reviewer:** {model_label}
**Slice:** {slice_name}
**Target:** Windows 10/11 desktop (Win32/WebView2). macOS/Linux have separate code paths.

Perform a thorough senior-level review of this **post-refactor** codebase (commands/ module split, app_state, vault_io, hooks-based React frontend) covering:
1. **Security vulnerabilities** (vault/crypto, IPC, CSP, path traversal, clipboard, secrets exposure)
2. **Performance & optimization** (unnecessary re-renders, blocking I/O, memory, event listeners)
3. **Code quality & efficiency** (architecture, error handling, maintainability, Tauri lifecycle)
4. **Windows desktop integration** (tray, shortcuts, menus, close/quit lifecycle)
5. **Line numbers toggle** — verify `showLineNumbers`, menu `toggle_line_numbers`, persistence, and split/edit UX

Output **markdown only**:

# {model_label} — Windows review ({slice_name})

## Executive summary
(2–4 sentences)

## Verdict
PASS / PASS WITH FIXES / FAIL — one-line rationale

## Security vulnerabilities
| # | File:line (if visible) | Issue | Severity | Recommended fix | Why it matters |
|---|------------------------|-------|----------|-----------------|----------------|

## Optimization opportunities
| # | File(s) | Issue | Recommended change | Expected benefit |
|---|---------|-------|-------------------|------------------|

## Code quality (senior review)
| # | File(s) | Observation | Recommendation |
|---|---------|-------------|----------------|

## Windows-specific concerns

## Line numbers toggle review
(Is it complete? Menu + toolbar + persistence + scroll sync?)

## Top 10 actions (priority order)

Be specific with file paths and symbols. Do not invent files. Cite approximate line numbers when possible.

---
CODE:
{code}
"""


def combined_prompt(reports: dict[str, str]) -> str:
    excerpts: list[str] = []
    for name, body in reports.items():
        text = re.sub(r"<!--.*?-->\n?", "", body, flags=re.S)
        parts: list[str] = []
        for heading in [
            "Executive summary",
            "Verdict",
            "Security vulnerabilities",
            "Optimization opportunities",
            "Code quality",
            "Windows-specific concerns",
            "Line numbers toggle review",
            "Top 10 actions",
            "Top 5 actions",
        ]:
            m = re.search(rf"## {re.escape(heading)}(.*?)(?=\n## |\Z)", text, re.S | re.I)
            if m:
                parts.append(f"## {heading}\n{m.group(1).strip()[:4500]}")
        excerpts.append(f"# Source: {name}\n\n" + "\n\n".join(parts)[:26000])
    joined = "\n\n---\n\n".join(excerpts)
    return f"""Merge these **Windows port code reviews** of **gnomad-slate** (`slate_win64/`, post-refactor) into ONE combined report.

Reviewers: GLM 5.1 and Nemotron Super 120B (both NVIDIA NIM).

Output **ONLY valid markdown**. No thinking aloud. No preamble.

# Gnomad Slate — Combined Windows Code Review

## Executive summary

## Consensus verdict

## What both reviewers agree on
| Priority | Finding | Recommended action | File(s) | Reviewers |

## Security — agreed issues
| # | File:line | Issue | Severity | Recommended fix | Why it matters |

## Security — disagreements or single-reviewer findings
| Reviewer | Finding | Severity | Notes |

## Optimization — agreed
| # | File(s) | Issue | Recommended change | Expected benefit |

## Optimization — disagreements or single-reviewer
| Reviewer | Finding | Recommended change | Expected benefit |

## Code quality — agreed

## Line numbers toggle — consensus

## Rebuild recommendation
**Verdict:** Keep & patch / Partial rewrite / Full Windows-first rebuild
**Rationale:**

## Implementation order
### P0 (do first)
### P1 (next)
### P2 (later)

## Reviewer-specific highlights
### GLM-only findings
### Nemotron-only findings

---
INDIVIDUAL REVIEW EXCERPTS:
{joined}
"""


def retry_wait_seconds(exc: Exception, attempt: int) -> int:
    text = str(exc)
    if "retry_after_seconds" in text:
        import re

        match = re.search(r"'retry_after_seconds':\s*(\d+)", text)
        if match:
            return int(match.group(1)) + 5
    if "429" in text:
        return 45 * attempt
    return 15 * attempt


def call_chat(
    client: OpenAI,
    model: str,
    prompt: str,
    max_tokens: int = 6000,
    retries: int = 6,
) -> str:
    last_err: Exception | None = None
    for attempt in range(1, retries + 1):
        try:
            resp = client.chat.completions.create(
                model=model,
                messages=[{"role": "user", "content": prompt}],
                temperature=0.15,
                max_tokens=max_tokens,
            )
            content = resp.choices[0].message.content
            if not content or not content.strip():
                raise RuntimeError("empty response")
            return content.strip()
        except Exception as exc:
            last_err = exc
            print(f"  {model} attempt {attempt} failed: {exc}", file=sys.stderr)
            if attempt < retries:
                wait = retry_wait_seconds(exc, attempt)
                print(f"  waiting {wait}s before retry...", file=sys.stderr)
                time.sleep(wait)
    assert last_err is not None
    raise last_err


def call_chat_with_fallbacks(
    client: OpenAI,
    models: list[str],
    prompt: str,
    max_tokens: int = 6000,
) -> tuple[str, str]:
    last_err: Exception | None = None
    for model in models:
        try:
            return call_chat(client, model, prompt, max_tokens=max_tokens), model
        except Exception as exc:
            last_err = exc
            print(f"  giving up on {model}, trying next fallback...", file=sys.stderr)
            time.sleep(5)
    assert last_err is not None
    raise last_err


def write_report(path: Path, api: str, model_id: str, body: str, scope: str = "") -> None:
    scope_line = f"<!-- scope: {scope} -->\n" if scope else ""
    header = (
        f"<!-- api: {api} -->\n"
        f"<!-- model: {model_id} -->\n"
        f"{scope_line}"
        f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
    )
    path.write_text(header + body.strip() + "\n", encoding="utf-8")


def run_reviewer(
    label: str,
    model_id: str | list[str],
    api: str,
    client: OpenAI,
    rust_code: str,
    fe_code: str,
    resume: bool,
) -> str:
    out_path = OUT_DIR / f"WINDOWS_REVIEW_{label}.md"
    if resume and out_path.is_file() and out_path.stat().st_size > 500:
        print(f"== skip {label} (exists) ==")
        return out_path.read_text(encoding="utf-8")

    model_desc = model_id if isinstance(model_id, str) else " / ".join(model_id)
    print(f"== {label} ({model_desc} via {api}) ==")

    if isinstance(model_id, list):
        rust_review, used_model = call_chat_with_fallbacks(
            client, model_id, review_prompt(label, "Rust / Tauri backend", rust_code)
        )
        time.sleep(3)
        fe_review, used_model_fe = call_chat_with_fallbacks(
            client, [used_model, *model_id], review_prompt(label, "React frontend (Windows-gated)", fe_code)
        )
        model_id = used_model_fe
    else:
        rust_review = call_chat(client, model_id, review_prompt(label, "Rust / Tauri backend", rust_code))
        time.sleep(3)
        fe_review = call_chat(client, model_id, review_prompt(label, "React frontend (Windows-gated)", fe_code))

    combined = f"{rust_review}\n\n---\n\n{fe_review}"
    write_report(out_path, api, str(model_id), combined, scope="slate_win64 Windows port")
    print(f"  wrote {out_path.name} ({out_path.stat().st_size} bytes)")
    return combined


def synthesize(client: OpenAI, reports: dict[str, str]) -> None:
    print(f"== synthesis ({SYNTH_MODEL}) ==")
    synthesis = call_chat(client, SYNTH_MODEL, combined_prompt(reports), max_tokens=7000)
    final = OUT_DIR / "WINDOWS_CODE_REVIEW_COMBINED.md"
    write_report(final, "NVIDIA NIM", f"synthesis:{SYNTH_MODEL}", synthesis, scope="slate_win64 GLM+Nemotron consensus")
    print(f"wrote {final.name} ({final.stat().st_size} bytes)")


def main() -> int:
    parser = argparse.ArgumentParser(description="Windows slate_win64 code review via GLM + Nemotron (NVIDIA NIM)")
    parser.add_argument("--resume", action="store_true", help="Skip existing individual reviews")
    parser.add_argument("--synthesis-only", action="store_true", help="Only rebuild combined report")
    args = parser.parse_args()

    load_env()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    if not WIN_ROOT.is_dir():
        print(f"slate_win64 not found at {WIN_ROOT}", file=sys.stderr)
        return 1

    nim_key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not nim_key:
        print("NVIDIA_API_KEY missing in .env", file=sys.stderr)
        return 1

    client = OpenAI(base_url=NIM_BASE_URL, api_key=nim_key, timeout=900.0)

    if args.synthesis_only:
        reports: dict[str, str] = {}
        for label, _ in REVIEWERS:
            path = OUT_DIR / f"WINDOWS_REVIEW_{label}.md"
            if not path.is_file():
                print(f"Missing {path}", file=sys.stderr)
                return 1
            reports[label] = path.read_text(encoding="utf-8")
        synthesize(client, reports)
        return 0

    rust_code = bundle(RUST_FILES)
    fe_code = bundle(FRONTEND_FILES)
    reports: dict[str, str] = {}

    for label, model_id in REVIEWERS:
        reports[label] = run_reviewer(
            label, model_id, "NVIDIA NIM", client, rust_code, fe_code, args.resume
        )
        time.sleep(5)

    synthesize(client, reports)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
