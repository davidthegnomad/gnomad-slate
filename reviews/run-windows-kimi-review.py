#!/usr/bin/env python3
"""Kimi K2.6 Windows code review for slate_win64 via NVIDIA NIM."""
from __future__ import annotations

import argparse
import importlib.util
import os
import sys
import time
from pathlib import Path

from openai import OpenAI

PROJECT = Path(__file__).resolve().parents[1]
OUT_DIR = PROJECT / "reviews"
NIM_BASE_URL = "https://integrate.api.nvidia.com/v1"

LABEL = "kimi-k2.6"
MODEL = "moonshotai/kimi-k2.6"

_spec = importlib.util.spec_from_file_location(
    "run_windows_multi_review",
    Path(__file__).resolve().parent / "run-windows-multi-review.py",
)
_mod = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(_mod)


def call_chat(client: OpenAI, prompt: str, max_tokens: int = 7000) -> str:
    last_err: Exception | None = None
    for attempt in range(1, 4):
        try:
            resp = client.chat.completions.create(
                model=MODEL,
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
            print(f"  attempt {attempt} failed: {exc}", file=sys.stderr)
            if attempt < 3:
                time.sleep(20 * attempt)
    assert last_err is not None
    raise last_err


def main() -> int:
    parser = argparse.ArgumentParser(description="Kimi K2.6 Windows review via NVIDIA NIM")
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args()

    _mod.load_env()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    key = os.environ.get("NVIDIA_API_KEY", "").strip()
    if not key:
        print("NVIDIA_API_KEY missing in .env", file=sys.stderr)
        return 1

    out_path = OUT_DIR / f"WINDOWS_REVIEW_{LABEL}.md"
    if args.resume and out_path.is_file() and out_path.stat().st_size > 500:
        print(f"skip {LABEL} (exists)")
        return 0

    client = OpenAI(base_url=NIM_BASE_URL, api_key=key, timeout=900.0)
    rust_code = _mod.bundle(_mod.RUST_FILES)
    fe_code = _mod.bundle(_mod.FRONTEND_FILES)

    print(f"== {LABEL} ({MODEL}) ==")
    rust_review = call_chat(client, _mod.review_prompt(LABEL, "Rust / Tauri backend", rust_code))
    time.sleep(3)
    fe_review = call_chat(client, _mod.review_prompt(LABEL, "React frontend (Windows-gated)", fe_code))
    combined = f"{rust_review}\n\n---\n\n{fe_review}"
    _mod.write_report(out_path, "NVIDIA NIM", MODEL, combined, scope="slate_win64 Windows port (post-refactor)")
    print(f"wrote {out_path.name} ({out_path.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
