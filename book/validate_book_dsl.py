#!/usr/bin/env python3
"""Extract Sruja DSL blocks from book markdown and run sruja lint on each."""

import os
import re
import subprocess
import sys
from pathlib import Path

BOOK_SRC = Path(__file__).parent / "src"
TMP_DIR = Path(__file__).parent / "tmp_validate"
SRUJA = "sruja"


def extract_sruja_blocks(filepath):
    """Extract ```sruja ... ``` blocks from markdown."""
    content = filepath.read_text(encoding="utf-8")
    pattern = r"```sruja\n(.*?)```"
    blocks = re.findall(pattern, content, re.DOTALL)
    results = []
    for block in blocks:
        is_partial = bool(
            re.search(
                r"<!--\s*partial\s*-->|#\s*partial|//\s*partial|EXPECTED_FAILURE",
                block,
                re.IGNORECASE,
            )
        )
        results.append((block.strip(), is_partial))
    return results


def run_lint(filepath):
    """Run sruja lint, return (exit_code, stderr)."""
    result = subprocess.run(
        [SRUJA, "lint", str(filepath)],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent.parent,
    )
    return result.returncode, result.stderr


def main():
    TMP_DIR.mkdir(exist_ok=True)
    failures = []
    passed = 0
    skipped = 0
    total = 0

    for md_file in BOOK_SRC.rglob("*.md"):
        blocks = extract_sruja_blocks(md_file)
        rel = md_file.relative_to(BOOK_SRC)
        for i, (block, is_partial) in enumerate(blocks):
            if not block or len(block) < 10:
                continue
            total += 1
            if is_partial:
                skipped += 1
                print(f"SKIP {rel} (block {i + 1}) - marked as partial\n")
                continue
            tmp_name = f"{rel.stem}_{i}.sruja".replace("/", "_").replace(" ", "_")
            tmp_path = TMP_DIR / tmp_name
            tmp_path.write_text(block, encoding="utf-8")
            code, err = run_lint(tmp_path)
            if code != 0:
                failures.append((str(rel), i + 1, block[:100] + "...", err.strip()))
            else:
                passed += 1

    print(
        f"Validated {total} DSL blocks: {passed} passed, {len(failures)} failed, {skipped} skipped\n"
    )
    for rel, idx, preview, err in failures:
        print(f"FAIL {rel} (block {idx})")
        print(f"  Preview: {preview[:80]}...")
        for line in err.split("\n")[:5]:
            print(f"  {line}")
        print()
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
