#!/usr/bin/env python3
"""Linux code review via NVIDIA NIM (Kimi, Qwen, DeepSeek, etc.)."""
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
ENV = PROJECT / ".env"
NIM = "https://integrate.api.nvidia.com/v1"

# Verified on NIM catalog 2026-06-23
DEFAULT_MODELS = {
    "kimi": "moonshotai/kimi-k2.6",
    "qwen": "qwen/qwen3.5-122b-a10b",
    "deepseek": "deepseek-ai/deepseek-v4-flash",
    "glm": "z-ai/glm-5.1",
}

REVIEW_FILES = [
    "reviews/LINUX_CODE_REVIEW_COMBINED.md",
    "reviews/LINUX_P0_REVIEW_glm-5.1.md",
    "reviews/LINUX_P2_REVIEW_glm-5.1.md",
    "src-tauri/src/lib.rs",
    "src-tauri/src/linux_tray.rs",
    "src-tauri/src/logging.rs",
    "src-tauri/src/menu/linux.rs",
    "src-tauri/src/filesystem.rs",
    "src-tauri/tauri.conf.json",
    "src-tauri/Cargo.toml",
    "src/App.tsx",
    "src/utils/ipc.ts",
    "src/utils/settings.ts",
]

MAX_SLICE = 40_000


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
            continue
        text = path.read_text(errors="replace")
        if len(text) > 22_000:
            text = text[:22_000] + "\n[truncated]\n"
        parts.append(f"### {rel}\n```\n{text}\n```\n")
    body = "\n".join(parts)
    return body[:MAX_SLICE] + ("\n[... truncated ...]\n" if len(body) > MAX_SLICE else "")


def prompt(model_label: str, code: str) -> str:
    return f"""You are **{model_label}** performing a **post-implementation Linux code review** of **Gnomad Slate** (Tauri 2 + React 19).

P0/P1/P2 Linux fixes are in place (tray, Super+Shift+Space, quit lifecycle, plugin-store, CSP, tracing). Prior GLM reviews: PASS.

Target: Nobara/Fedora, Wayland-first. macOS uses `menu/macos.rs` — do not regress macOS.

Output **markdown only**:

# {model_label} — Linux post-implementation review

## Executive summary

## Verdict
PASS / PASS WITH FIXES / FAIL

## Findings
### Critical
| # | File(s) | Issue | Recommended fix | Why |
### High
### Medium
### Low

## Regressions vs prior review checklist

## Security & lifecycle

## Top 5 actions

Be specific. Do not invent files.

---
{code}
"""


def slug(model_id: str) -> str:
    return model_id.replace("/", "-").replace(".", "")


def main() -> int:
    parser = argparse.ArgumentParser(description="Linux code review via NVIDIA NIM")
    parser.add_argument(
        "--model",
        default=DEFAULT_MODELS["kimi"],
        help=f"NIM model id (aliases: {', '.join(DEFAULT_MODELS)})",
    )
    args = parser.parse_args()

    model_id = DEFAULT_MODELS.get(args.model, args.model)
    load_env()
    key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not key:
        print("NVIDIA_API_KEY missing in .env", file=sys.stderr)
        return 1

    short = args.model if args.model in DEFAULT_MODELS else slug(model_id)
    out = OUT_DIR / f"LINUX_REVIEW_{short}.md"

    client = OpenAI(base_url=NIM, api_key=key, timeout=900.0)
    label = model_id.split("/")[-1]
    body_prompt = prompt(label, bundle())

    last: Exception | None = None
    for attempt in range(1, 4):
        try:
            resp = client.chat.completions.create(
                model=model_id,
                messages=[{"role": "user", "content": body_prompt}],
                temperature=0.15,
                max_tokens=6000,
            )
            body = (resp.choices[0].message.content or "").strip()
            if not body:
                raise RuntimeError("empty response")
            header = (
                f"<!-- api: NVIDIA NIM -->\n"
                f"<!-- model: {model_id} -->\n"
                f"<!-- scope: Linux post-P0/P1/P2 -->\n"
                f"<!-- generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')} -->\n\n"
            )
            out.write_text(header + body + "\n", encoding="utf-8")
            print(f"wrote {out} ({out.stat().st_size} bytes)")
            return 0
        except Exception as exc:
            last = exc
            print(f"attempt {attempt} failed: {exc}", file=sys.stderr)
            time.sleep(20 * attempt)

    assert last
    raise last


if __name__ == "__main__":
    raise SystemExit(main())
