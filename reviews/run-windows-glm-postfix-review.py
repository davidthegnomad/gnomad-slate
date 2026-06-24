#!/usr/bin/env python3
"""GLM post-fix review of slate_win64 — modularity, clarity, documentation."""
from __future__ import annotations

import importlib.util
import os
import sys
import time
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "reviews" / "WINDOWS_REVIEW_glm-5.1-postfix.md"
NIM_BASE_URL = "https://integrate.api.nvidia.com/v1"
MODEL = "z-ai/glm-5.1"

_spec = importlib.util.spec_from_file_location(
    "run_windows_multi_review",
    Path(__file__).resolve().parent / "run-windows-multi-review.py",
)
_mod = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(_mod)


def prompt(slice_name: str, code: str, prior_summary: str) -> str:
    return f"""You are **GLM 5.1** performing a **post-fix code review** of the Gnomad Slate Windows port in `slate_win64/`.

Context: A prior review flagged security, lifecycle, and monolithic-structure issues. Fixes were applied (v3 per-vault KDF salt, path validation, clipboard TTL, exit lifecycle, debounced preview, useMemo filters, secret reveal via ref).

**Slice:** {slice_name}
**Prior review highlights:** {prior_summary}

Focus this review on **maintainability and architecture**, not re-stating fixed security items unless regressions exist:

1. **Monolithic code** — flag God components, God modules, files that should be split (with concrete extraction plan)
2. **Complicated logic** — identify non-obvious code that lacks comments/docstrings; specify what to explain and where
3. **Module boundaries** — hooks, contexts, Rust module separation, IPC surface
4. **Naming & readability** — unclear abstractions, magic numbers, duplicated logic
5. **Testability** — what blocks unit tests and how to unblock

Output **markdown only**:

# GLM 5.1 — Post-fix review ({slice_name})

## Executive summary

## Verdict
PASS / PASS WITH FIXES / FAIL

## Monolithic structure findings
| # | File | Problem | Recommended split | Target module(s) |
|---|------|---------|-------------------|------------------|

## Complicated code needing explanation
| # | File:area | Why it's complex | Comment/doc to add |
|---|-----------|------------------|-------------------|

## Refactor plan (priority order)
### P0 — split before next feature
### P1 — clarity & comments
### P2 — nice to have

## Specific file-level recommendations
(bullet list with paths)

## Top 10 actions

Be specific. Do not invent files. Prefer small, reviewable extractions over big-bang rewrites.

---
CODE:
{code}
"""


PRIOR = (
    "v3 vault salt, atomic writes, clipboard TTL, QUIT_IN_PROGRESS lifecycle, "
    "debounced preview, filteredNotes useMemo, revealedValue ref, line numbers toggle"
)


def call(client: OpenAI, text: str, max_tokens: int = 7000) -> str:
    last: Exception | None = None
    for attempt in range(1, 4):
        try:
            resp = client.chat.completions.create(
                model=MODEL,
                messages=[{"role": "user", "content": text}],
                temperature=0.15,
                max_tokens=max_tokens,
            )
            body = (resp.choices[0].message.content or "").strip()
            if body:
                return body
            raise RuntimeError("empty response")
        except Exception as exc:
            last = exc
            print(f"  attempt {attempt}: {exc}", file=sys.stderr)
            time.sleep(15 * attempt)
    assert last is not None
    raise last


def main() -> int:
    _mod.load_env()
    key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not key:
        print("NVIDIA_API_KEY missing", file=sys.stderr)
        return 1

    client = OpenAI(base_url=NIM_BASE_URL, api_key=key, timeout=900.0)
    rust = _mod.bundle(_mod.RUST_FILES)
    fe = _mod.bundle(_mod.FRONTEND_FILES)

    print(f"== post-fix review ({MODEL}) ==")
    rust_review = call(client, prompt("Rust / Tauri backend", rust, PRIOR))
    time.sleep(3)
    fe_review = call(client, prompt("React frontend", fe, PRIOR))
    combined = f"{rust_review}\n\n---\n\n{fe_review}"

    header = (
        f"<!-- api: NVIDIA NIM -->\n"
        f"<!-- model: {MODEL} -->\n"
        f"<!-- scope: slate_win64 post-fix modularity review -->\n\n"
    )
    OUT.write_text(header + combined + "\n", encoding="utf-8")
    print(f"wrote {OUT.name} ({OUT.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
