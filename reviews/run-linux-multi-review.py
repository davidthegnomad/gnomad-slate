#!/usr/bin/env python3
"""Multi-model Linux code review for gnomad-slate via NVIDIA NIM API."""
from __future__ import annotations

import argparse
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT_DIR = PROJECT / "reviews"
ENV_PATH = PROJECT / ".env"
NIM_BASE_URL = "https://integrate.api.nvidia.com/v1"
DEFAULT_MODEL = "nvidia/nemotron-3-ultra-550b-a55b"

# Second model on NIM when a multi-model pass is requested
SECONDARY_MODEL = "z-ai/glm-5.1"

MODELS = [
    ("nemotron-ultra", DEFAULT_MODEL),
    ("glm-5.1", SECONDARY_MODEL),
]

RUST_FILES = [
    "src-tauri/src/menu/linux.rs",
    "src-tauri/src/menu/mod.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/tauri.conf.json",
    "src-tauri/capabilities/default.json",
    "src-tauri/Cargo.toml",
    "scripts/install-linux-deps.sh",
]

FRONTEND_FILES = [
    "src/App.tsx",
    "src/App.css",
    "src/hooks/useLinuxMenuBridge.ts",
    "src/components/HelpDialog.tsx",
    "src/content/linuxHelp.ts",
    "src/utils/platform.ts",
    "src/utils/ipc.ts",
    "src/utils/modKeyLabel.ts",
    "package.json",
]

MAX_SLICE = 42_000


def load_env() -> None:
    if not ENV_PATH.is_file():
        return
    for line in ENV_PATH.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def nim_client(timeout: float = 900.0) -> OpenAI:
    api_key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not api_key:
        raise RuntimeError("NVIDIA_API_KEY missing in .env")
    return OpenAI(base_url=NIM_BASE_URL, api_key=api_key, timeout=timeout)


def bundle(files: list[str]) -> str:
    parts: list[str] = []
    for rel in files:
        path = PROJECT / rel
        if not path.is_file():
            continue
        text = path.read_text(errors="replace")
        parts.append(f"### {rel}\n```\n{text}\n```\n")
    body = "\n".join(parts)
    if len(body) > MAX_SLICE:
        body = body[:MAX_SLICE] + "\n\n[... truncated for token budget ...]\n"
    return body


def review_prompt(model_label: str, slice_name: str, code: str) -> str:
    return f"""You are a senior Linux desktop engineer reviewing **Gnomad Slate** (Tauri 2 + React 19) for **Linux-only optimization**.

**Reviewer:** {model_label}
**Slice:** {slice_name}

Target: Nobara/Fedora Linux. macOS has separate menu code (`menu/macos.rs`).

Output **markdown only**:

# {model_label} — Linux review ({slice_name})

## Critical
| # | File(s) | Issue | Recommended change | Why |
|---|---------|-------|-------------------|-----|

## High (Wayland/WebKit/GTK/Tauri lifecycle)

## Medium

## Low

## Rebuild vs refactor
(one short paragraph)

## Top 5 actions

Be specific. Do not invent files.

---
CODE:
{code}
"""


def combined_prompt(reports: dict[str, str]) -> str:
    joined = "\n\n---\n\n".join(f"# Source: {name}\n\n{body}" for name, body in reports.items())
    return f"""Merge these Linux code reviews of **gnomad-slate** into ONE combined report.

Output **markdown only**. Make **what to change and why** obvious.

# Gnomad Slate — Combined Linux Code Review

## Executive summary

## Recommended changes (master list)
| Priority | Change (what to do) | Why (rationale) | File(s) | Reviewers agreeing |

## Consensus themes

## Disagreements / optional bets

## Rebuild recommendation
**Verdict:** Keep & patch / Partial rewrite / Full Linux-first rebuild
**Rationale:**

## Implementation order
### Week 1 (P0)
### Week 2–3 (P1)
### Later (P2)

---
INDIVIDUAL REVIEWS:
{joined}
"""


def call_nim(client: OpenAI, model: str, prompt: str, max_tokens: int = 5000) -> str:
    last_err: Exception | None = None
    for attempt in range(1, 4):
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
            if attempt < 3:
                time.sleep(15 * attempt)
    assert last_err is not None
    raise last_err


def write_report(path: Path, model_id: str, body: str) -> None:
    header = (
        f"<!-- api: NVIDIA NIM -->\n"
        f"<!-- model: {model_id} -->\n"
        f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
    )
    path.write_text(header + body.strip() + "\n", encoding="utf-8")


def synthesize_only() -> int:
    load_env()
    client = nim_client()
    reports: dict[str, str] = {}
    for label, _ in MODELS:
        path = OUT_DIR / f"LINUX_REVIEW_{label}.md"
        if not path.is_file():
            # legacy filenames from OpenRouter run
            legacy = {
                "nemotron-ultra": OUT_DIR / "LINUX_REVIEW_nemotron-ultra.md",
                "glm-5.1": OUT_DIR / "LINUX_REVIEW_glm-5.2.md",
            }.get(label)
            if legacy and legacy.is_file():
                reports[label] = legacy.read_text(encoding="utf-8")
                continue
            print(f"Missing {path}", file=sys.stderr)
            return 1
        reports[label] = path.read_text(encoding="utf-8")

    print(f"== synthesis ({DEFAULT_MODEL}) ==")
    synthesis = call_nim(client, DEFAULT_MODEL, combined_prompt(reports), max_tokens=9000)
    final = OUT_DIR / "LINUX_CODE_REVIEW_COMBINED.md"
    write_report(final, f"synthesis:{DEFAULT_MODEL}", synthesis)
    print(f"wrote {final.name} ({final.stat().st_size} bytes)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Linux code review via NVIDIA NIM")
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--synthesis-only", action="store_true")
    args = parser.parse_args()

    if args.synthesis_only:
        return synthesize_only()

    load_env()
    client = nim_client()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    rust_code = bundle(RUST_FILES)
    fe_code = bundle(FRONTEND_FILES)
    all_reports: dict[str, str] = {}

    for label, model_id in MODELS:
        out_path = OUT_DIR / f"LINUX_REVIEW_{label}.md"
        if args.resume and out_path.is_file() and out_path.stat().st_size > 500:
            print(f"== skip {label} (exists) ==")
            all_reports[label] = out_path.read_text(encoding="utf-8")
            continue
        print(f"== {label} ({model_id}) ==")
        rust_review = call_nim(client, model_id, review_prompt(label, "Rust / Tauri backend", rust_code))
        time.sleep(2)  # gentle pacing for NIM quotas
        fe_review = call_nim(client, model_id, review_prompt(label, "React frontend (Linux-gated)", fe_code))
        combined = f"{rust_review}\n\n---\n\n{fe_review}"
        write_report(out_path, model_id, combined)
        all_reports[label] = combined
        print(f"  wrote {out_path.name}")

    print(f"== synthesis ({DEFAULT_MODEL}) ==")
    synthesis = call_nim(client, DEFAULT_MODEL, combined_prompt(all_reports), max_tokens=9000)
    final = OUT_DIR / "LINUX_CODE_REVIEW_COMBINED.md"
    write_report(final, f"synthesis:{DEFAULT_MODEL}", synthesis)
    print(f"wrote {final.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
