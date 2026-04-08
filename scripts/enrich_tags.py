#!/usr/bin/env python3
"""Batch-enrich SKILL.md frontmatter with Codex CLI-generated tags."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CODEX_BIN = "codex"
DEFAULT_TIMEOUT_SECONDS = 180
DEFAULT_MAX_SKILL_CHARS = 14_000
DEFAULT_OUTPUT_TAG_COUNT = 10
FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---(?P<body>.*)$", re.DOTALL)
NAME_RE = re.compile(r"(?m)^name:\s*(.+?)\s*$")
TAGS_RE = re.compile(r"(?m)^tags:\s*(.*?)\s*$")
TAG_PATTERN = re.compile(r"^[a-z0-9][a-z0-9+._#-]*$")

TAG_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "properties": {
        "tags": {
            "type": "array",
            "minItems": 6,
            "maxItems": 12,
            "items": {
                "type": "string",
                "pattern": "^[a-z0-9][a-z0-9+._#-]*$",
            },
        }
    },
    "required": ["tags"],
}

DEVELOPER_PROMPT = """\
You generate retrieval tags for AI skills stored in markdown files.

Return 6 to 12 tags that improve semantic matching for search and routing.

Rules:
- Use only the SKILL.md content provided in the prompt. Do not inspect files or run commands.
- Return JSON only, matching the provided schema.
- Tags must be lowercase and concise.
- Prefer concrete domains, technologies, paradigms, failure modes, and workflows.
- Avoid generic filler like ai, skill, guide, expert, developer, software, code, coding.
- Avoid near-duplicates and redundant synonyms.
"""


@dataclass(frozen=True)
class SkillDocument:
    path: Path
    relative_path: Path
    content: str
    frontmatter: str
    body: str
    skill_id: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--codex-bin",
        default=DEFAULT_CODEX_BIN,
        help=f"Codex CLI binary to run (default: {DEFAULT_CODEX_BIN})",
    )
    parser.add_argument(
        "--model",
        help="Codex CLI model override. If omitted, use the CLI default/profile.",
    )
    parser.add_argument(
        "--reasoning-effort",
        choices=("low", "medium", "high", "xhigh"),
        help="Codex CLI reasoning effort override. If omitted, use the CLI default/profile.",
    )
    parser.add_argument(
        "--filter",
        help="Only process SKILL.md paths containing this substring.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        help="Maximum number of skill files to process.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Regenerate tags for skills that already have a tags field.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print generated tags without writing files.",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=DEFAULT_TIMEOUT_SECONDS,
        help=f"Per-skill Codex timeout in seconds (default: {DEFAULT_TIMEOUT_SECONDS})",
    )
    parser.add_argument(
        "--max-skill-chars",
        type=int,
        default=DEFAULT_MAX_SKILL_CHARS,
        help=f"Maximum characters from each SKILL.md to send to Codex (default: {DEFAULT_MAX_SKILL_CHARS})",
    )
    return parser.parse_args()


def ensure_codex_available(codex_bin: str) -> None:
    if shutil.which(codex_bin):
        return
    raise SystemExit(f"Codex CLI not found on PATH: {codex_bin}")


def iter_skill_paths(path_filter: str | None) -> Iterable[Path]:
    for skill_path in sorted(REPO_ROOT.rglob("SKILL.md")):
        if "skill-template" in str(skill_path):
            continue
        if path_filter and path_filter not in str(skill_path.relative_to(REPO_ROOT)):
            continue
        yield skill_path


def load_skill_document(skill_path: Path) -> SkillDocument | None:
    content = skill_path.read_text(encoding="utf-8")
    match = FRONTMATTER_RE.match(content)
    if not match:
        print(f"  SKIP {skill_path}: no frontmatter")
        return None

    frontmatter = match.group(1).strip()
    body = match.group("body")
    skill_id_match = NAME_RE.search(frontmatter)
    skill_id = (
        skill_id_match.group(1).strip() if skill_id_match else skill_path.parent.name
    )

    return SkillDocument(
        path=skill_path,
        relative_path=skill_path.relative_to(REPO_ROOT),
        content=content,
        frontmatter=frontmatter,
        body=body,
        skill_id=skill_id,
    )


def has_tags(frontmatter: str) -> bool:
    return TAGS_RE.search(frontmatter) is not None


def build_prompt(skill: SkillDocument, max_skill_chars: int) -> str:
    trimmed_content = skill.content[:max_skill_chars]
    return (
        f"{DEVELOPER_PROMPT}\n"
        f"Repository path: {skill.relative_path}\n"
        f"Skill ID: {skill.skill_id}\n\n"
        "Generate high-signal retrieval tags for this skill.\n\n"
        "SKILL.md content:\n"
        f"{trimmed_content}\n"
    )


def build_codex_command(
    codex_bin: str,
    schema_path: Path,
    output_path: Path,
    args: argparse.Namespace,
) -> list[str]:
    command = [
        codex_bin,
        "exec",
        "--ephemeral",
        "--color",
        "never",
        "--sandbox",
        "read-only",
        "--skip-git-repo-check",
        "-C",
        str(REPO_ROOT),
        "--output-schema",
        str(schema_path),
        "-o",
        str(output_path),
        "-c",
        "features.shell_tool=false",
        "-c",
        'web_search="enabled"',
    ]
    if args.model:
        command.extend(["--model", args.model])
    if args.reasoning_effort:
        command.extend(["-c", f'model_reasoning_effort="{args.reasoning_effort}"'])
    command.append("-")
    return command


def request_tags_with_codex(
    skill: SkillDocument, args: argparse.Namespace
) -> list[str]:
    prompt = build_prompt(skill, args.max_skill_chars)

    with tempfile.TemporaryDirectory(prefix="codex-enrich-tags-") as tmpdir:
        tmpdir_path = Path(tmpdir)
        schema_path = tmpdir_path / "schema.json"
        output_path = tmpdir_path / "output.json"
        schema_path.write_text(json.dumps(TAG_SCHEMA, indent=2), encoding="utf-8")

        command = build_codex_command(args.codex_bin, schema_path, output_path, args)
        completed = subprocess.run(
            command,
            input=prompt,
            text=True,
            capture_output=True,
            timeout=args.timeout,
            check=False,
        )

        if completed.returncode != 0:
            stderr = completed.stderr.strip()
            stdout = completed.stdout.strip()
            detail = stderr or stdout or "codex exec failed without output"
            raise RuntimeError(f"codex exec failed for {skill.relative_path}: {detail}")

        if not output_path.exists():
            raise RuntimeError(
                f"Codex did not write a final output file for {skill.relative_path}"
            )

        try:
            payload = json.loads(output_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise RuntimeError(
                f"Codex output was not valid JSON for {skill.relative_path}: {exc}"
            ) from exc

    return parse_tags(payload)


def parse_tags(payload: dict) -> list[str]:
    raw_tags = payload.get("tags")
    if not isinstance(raw_tags, list):
        raise RuntimeError("Structured output did not include a tags list")

    normalized: list[str] = []
    seen: set[str] = set()
    for raw_tag in raw_tags:
        tag = normalize_tag(raw_tag)
        if not tag or tag in seen:
            continue
        if not TAG_PATTERN.fullmatch(tag):
            raise RuntimeError(f"Invalid tag returned by Codex: {raw_tag!r}")
        normalized.append(tag)
        seen.add(tag)

    if len(normalized) < 6:
        raise RuntimeError(f"Codex returned too few usable tags: {normalized!r}")

    return normalized[:DEFAULT_OUTPUT_TAG_COUNT]


def normalize_tag(raw_tag: object) -> str:
    if not isinstance(raw_tag, str):
        return ""
    tag = raw_tag.strip().lower()
    tag = re.sub(r"[\s/]+", "-", tag)
    tag = re.sub(r"[^a-z0-9+._#-]", "", tag)
    tag = re.sub(r"-{2,}", "-", tag)
    tag = re.sub(r"^[^a-z0-9]+", "", tag)
    return tag


def write_tags(skill: SkillDocument, tags: list[str]) -> None:
    new_frontmatter = skill.frontmatter.rstrip() + f"\ntags: {', '.join(tags)}"
    new_content = f"---\n{new_frontmatter}\n---{skill.body}"
    skill.path.write_text(new_content, encoding="utf-8")


def replace_tags(skill: SkillDocument, tags: list[str]) -> None:
    replacement = f"tags: {', '.join(tags)}"
    new_frontmatter, count = TAGS_RE.subn(replacement, skill.frontmatter, count=1)
    if count != 1:
        raise RuntimeError(f"Expected exactly one tags field in {skill.relative_path}")
    new_content = f"---\n{new_frontmatter}\n---{skill.body}"
    skill.path.write_text(new_content, encoding="utf-8")


def main() -> int:
    args = parse_args()
    ensure_codex_available(args.codex_bin)

    enriched = 0
    replaced = 0
    skipped = 0
    failed = 0

    for index, skill_path in enumerate(iter_skill_paths(args.filter), start=1):
        if args.limit is not None and index > args.limit:
            break

        skill = load_skill_document(skill_path)
        if not skill:
            skipped += 1
            continue

        already_tagged = has_tags(skill.frontmatter)
        if already_tagged and not args.force:
            print(f"  SKIP {skill.skill_id}: already has tags")
            skipped += 1
            continue

        try:
            tags = request_tags_with_codex(skill, args)
        except subprocess.TimeoutExpired:
            print(
                f"  FAIL {skill.skill_id}: codex exec timed out after {args.timeout}s"
            )
            failed += 1
            continue
        except Exception as exc:  # noqa: BLE001 - keep batch processing running
            print(f"  FAIL {skill.skill_id}: {exc}")
            failed += 1
            continue

        if args.dry_run:
            action = "REPLACE" if already_tagged else "ADD"
            print(f"  {action} {skill.skill_id}: {', '.join(tags)}")
        else:
            if already_tagged:
                replace_tags(skill, tags)
                replaced += 1
            else:
                write_tags(skill, tags)
                enriched += 1
            print(f"  ✓ {skill.skill_id}: {', '.join(tags)}")

    print(
        "\nDone: "
        f"{enriched} enriched, "
        f"{replaced} replaced, "
        f"{skipped} skipped, "
        f"{failed} failed"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
