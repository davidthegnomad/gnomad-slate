#!/usr/bin/env python3
"""GLM (NVIDIA NIM) review of P0 Linux implementation."""
from __future__ import annotations

import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "reviews" / "LINUX_P0_REVIEW_glm-5.1.md"
ENV = PROJECT / ".env"
MODEL = "z-ai/glm-5.1"
NIM = "https://integrate.api.nvidia.com/v1"

P0_FILES = [
    "src-tauri/src/lib.rs",
    "src-tauri/src/linux_tray.rs",
    "src-tauri/src/menu/linux.rs",
    "src/App.tsx",
    "src/utils/ipc.ts",
    "src/components/AppMessageDialog.tsx",
    "src/content/linuxHelp.ts",
    "reviews/LINUX_CODE_REVIEW_COMBINED.md",
]

CHECKLIST = """
P0 checklist from LINUX_CODE_REVIEW_COMBINED.md:
1. Gate maybe_show_on_app_activate / macos_app to macOS only
2. Remove or Wayland-gate hide-on-blur (macOS only)
3. Global shortcut Super+Shift+Space (not Alt+Space)
4. System tray Show/Hide/Quit
5. Close/quit lifecycle: save → vault lock → exit
6. Replace alert() with in-app modal on Linux
7. Remove mount-time win.setSize()
"""


def load_env() -> None:
    if not ENV.is_file():
        return
    for line in ENV.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def bundle() -> str:
    parts: list[str] = []
    for rel in P0_FILES:
        p = PROJECT / rel
        if not p.is_file():
            parts.append(f"### {rel}\n(missing)\n")
            continue
        text = p.read_text(errors="replace")
        if len(text) > 28_000:
            text = text[:28_000] + "\n[truncated]\n"
        parts.append(f"### {rel}\n```\n{text}\n```\n")
    return "\n".join(parts)


def main() -> int:
    load_env()
    key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not key:
        print("NVIDIA_API_KEY missing", file=sys.stderr)
        return 1

    client = OpenAI(base_url=NIM, api_key=key, timeout=600.0)
    code = bundle()
    prompt = f"""You are GLM reviewing **P0 Linux fixes** just implemented in Gnomad Slate (Tauri 2 + React 19).

{CHECKLIST}

Verify each P0 item against the code. Output **markdown only**:

# GLM — P0 implementation review

## Verdict
PASS / PASS WITH FIXES / FAIL — one line summary

## Checklist
| # | P0 item | Status (done/partial/missing) | Evidence (file:line or symbol) | Remaining work |
|---|---------|------------------------------|--------------------------------|----------------|

## Critical gaps (if any)

## High-priority follow-ups before P1

## Code quality notes (Linux / Wayland / GTK)

Be specific. Do not invent files.

---
CODE AND PRIOR REVIEW:
{code}
"""

    last: Exception | None = None
    for attempt in range(1, 4):
        try:
            resp = client.chat.completions.create(
                model=MODEL,
                messages=[{"role": "user", "content": prompt}],
                temperature=0.1,
                max_tokens=6000,
            )
            body = (resp.choices[0].message.content or "").strip()
            if not body:
                raise RuntimeError("empty response")
            header = (
                f"<!-- api: NVIDIA NIM -->\n"
                f"<!-- model: {MODEL} -->\n"
                f"<!-- scope: P0 post-implementation -->\n"
                f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
            )
            OUT.write_text(header + body + "\n", encoding="utf-8")
            print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
            return 0
        except Exception as exc:
            last = exc
            print(f"attempt {attempt} failed: {exc}", file=sys.stderr)
            time.sleep(12 * attempt)
    assert last
    raise last


if __name__ == "__main__":
    raise SystemExit(main())
