#!/usr/bin/env python3
"""GLM (NVIDIA NIM) review of P2 Linux implementation."""
from __future__ import annotations

import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "reviews" / "LINUX_P2_REVIEW_glm-5.1.md"
ENV = PROJECT / ".env"
MODEL = "z-ai/glm-5.1"
NIM = "https://integrate.api.nvidia.com/v1"

P2_FILES = [
    "src-tauri/tauri.conf.json",
    "src-tauri/src/logging.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/Cargo.toml",
    "src/utils/settings.ts",
    "src/App.tsx",
    "reviews/LINUX_CODE_REVIEW_COMBINED.md",
]

CHECKLIST = """
P2 checklist:
1. Strict CSP in tauri.conf.json (not null)
2. tracing instead of eprintln in Rust
3. Settings migrated to @tauri-apps/plugin-store
4. modShortcut() used consistently (no bare ⌘ on Linux)
5. Debounced sidebar search; consolidated save paths
6. Binary named gnomad-slate
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
    for rel in P2_FILES:
        p = PROJECT / rel
        if not p.is_file():
            parts.append(f"### {rel}\n(missing)\n")
            continue
        text = p.read_text(errors="replace")
        if len(text) > 30_000:
            text = text[:30_000] + "\n[truncated]\n"
        parts.append(f"### {rel}\n```\n{text}\n```\n")
    return "\n".join(parts)


def main() -> int:
    load_env()
    key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not key:
        print("NVIDIA_API_KEY missing", file=sys.stderr)
        return 1

    client = OpenAI(base_url=NIM, api_key=key, timeout=600.0)
    prompt = f"""Review **P2 Linux polish** just implemented in Gnomad Slate.

{CHECKLIST}

Output markdown only:

# GLM — P2 implementation review

## Verdict
PASS / PASS WITH FIXES / FAIL

## Checklist
| # | P2 item | Status | Evidence | Remaining work |

## Gaps before ship

## Code quality

---
{bundle()}
"""

    for attempt in range(1, 4):
        try:
            resp = client.chat.completions.create(
                model=MODEL,
                messages=[{"role": "user", "content": prompt}],
                temperature=0.1,
                max_tokens=5000,
            )
            body = (resp.choices[0].message.content or "").strip()
            header = (
                f"<!-- api: NVIDIA NIM -->\n"
                f"<!-- model: {MODEL} -->\n"
                f"<!-- scope: P2 post-implementation -->\n"
                f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
            )
            OUT.write_text(header + body + "\n", encoding="utf-8")
            print(f"wrote {OUT} ({OUT.stat().st_size} bytes)")
            return 0
        except Exception as exc:
            print(f"attempt {attempt} failed: {exc}", file=sys.stderr)
            time.sleep(12 * attempt)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
