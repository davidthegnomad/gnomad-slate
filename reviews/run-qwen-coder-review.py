#!/usr/bin/env python3
"""Linux code review via Qwen3 Coder (OpenRouter)."""
from __future__ import annotations

import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "reviews" / "LINUX_REVIEW_qwen3-coder.md"
ENV = PROJECT / ".env"
MODEL = "qwen/qwen3-coder:free"
OPENROUTER = "https://openrouter.ai/api/v1"

REVIEW_FILES = [
    "reviews/LINUX_CODE_REVIEW_COMBINED.md",
    "reviews/LINUX_P0_REVIEW_glm-5.1.md",
    "reviews/LINUX_P2_REVIEW_glm-5.1.md",
    "src-tauri/src/lib.rs",
    "src-tauri/src/linux_tray.rs",
    "src-tauri/src/logging.rs",
    "src-tauri/src/menu/linux.rs",
    "src-tauri/src/menu/mod.rs",
    "src-tauri/src/filesystem.rs",
    "src-tauri/tauri.conf.json",
    "src-tauri/capabilities/default.json",
    "src-tauri/Cargo.toml",
    "src-tauri/bundle/linux/gnomad-slate.desktop.hbs",
    "scripts/install-linux-deps.sh",
    "src/App.tsx",
    "src/utils/ipc.ts",
    "src/utils/settings.ts",
    "src/utils/platform.ts",
    "src/components/AppMessageDialog.tsx",
    "src/content/linuxHelp.ts",
    "src/hooks/useLinuxMenuBridge.ts",
]

MAX_SLICE = 48_000


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
    for rel in REVIEW_FILES:
        path = PROJECT / rel
        if not path.is_file():
            parts.append(f"### {rel}\n(missing)\n")
            continue
        text = path.read_text(errors="replace")
        parts.append(f"### {rel}\n```\n{text}\n```\n")
    body = "\n".join(parts)
    if len(body) > MAX_SLICE:
        body = body[:MAX_SLICE] + "\n\n[... truncated for token budget ...]\n"
    return body


def review_prompt(code: str) -> str:
    return f"""You are **Qwen3 Coder** performing a **post-implementation Linux code review** of **Gnomad Slate** (Tauri 2 + React 19).

Context: P0/P1/P2 fixes were applied per prior Nemotron + GLM reviews (tray, Super+Shift+Space, quit lifecycle, plugin-store, CSP, tracing, etc.). GLM P0/P2 reviews reported PASS.

**Target:** Nobara/Fedora Linux, Wayland-first. macOS has separate code (`menu/macos.rs`) — do not recommend macOS regressions.

Output **markdown only**:

# Qwen3 Coder — Linux post-implementation review

## Executive summary
(2–4 sentences)

## Verdict
PASS / PASS WITH FIXES / FAIL — and one-line rationale

## Findings

### Critical
| # | File(s) | Issue | Recommended fix | Why it matters |
|---|---------|-------|-----------------|----------------|

### High
(same table columns)

### Medium

### Low

## Regressions / missed items from prior review
(Anything P0/P1/P2 still incomplete or newly broken)

## Security & lifecycle
(vault, quit, CSP, IPC)

## Packaging & desktop integration

## Top 5 actions (priority order)

Be specific with file paths and symbols. Do not invent files. If something looks fine, say so — do not pad with nitpicks.

---
CODE:
{code}
"""


def main() -> int:
    load_env()
    key = os.environ.get("OPENROUTER_API_KEY", "").strip()
    if not key:
        print("OPENROUTER_API_KEY missing in .env", file=sys.stderr)
        return 1

    client = OpenAI(base_url=OPENROUTER, api_key=key, timeout=900.0)
    code = bundle()
    prompt = review_prompt(code)

    last: Exception | None = None
    for attempt in range(1, 5):
        try:
            resp = client.chat.completions.create(
                model=MODEL,
                messages=[{"role": "user", "content": prompt}],
                temperature=0.15,
                max_tokens=8000,
            )
            body = (resp.choices[0].message.content or "").strip()
            if not body:
                raise RuntimeError("empty response")
            header = (
                f"<!-- api: OpenRouter -->\n"
                f"<!-- model: {MODEL} -->\n"
                f"<!-- scope: Linux post-P0/P1/P2 -->\n"
                f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
            )
            OUT.write_text(header + body + "\n", encoding="utf-8")
            print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
            return 0
        except Exception as exc:
            last = exc
            print(f"attempt {attempt} failed: {exc}", file=sys.stderr)
            wait = 20 * attempt
            if "429" in str(exc):
                wait = 45 * attempt
            time.sleep(wait)

    assert last is not None
    raise last


if __name__ == "__main__":
    raise SystemExit(main())
