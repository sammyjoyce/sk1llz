#!/usr/bin/env python3
"""Generate skills.json manifest from repository structure."""

import json
import re
from pathlib import Path
from datetime import datetime, timezone
import yaml

REPO_ROOT = Path(__file__).parent.parent
SKILL_FILE = "SKILL.md"

def parse_frontmatter(content: str) -> dict:
    """Extract YAML frontmatter from markdown."""
    match = re.match(r'^---\s*\n(.*?)\n---(?:\s*\n|$)', content, re.DOTALL)
    if not match:
        return {}

    raw_frontmatter = match.group(1)

    try:
        frontmatter = yaml.safe_load(raw_frontmatter) or {}
    except yaml.YAMLError:
        frontmatter = None

    if isinstance(frontmatter, dict):
        return frontmatter

    # Legacy fallback: support the repo's older "key: value" frontmatter
    # even when the value itself contains YAML-significant ":" characters.
    frontmatter = {}
    for line in raw_frontmatter.strip().split('\n'):
        if ':' not in line:
            continue
        key, value = line.split(':', 1)
        frontmatter[key.strip()] = value.strip()
    return frontmatter

def normalize_manifest_text(value: object) -> str:
    """Collapse YAML scalars into CLI-friendly single-line metadata."""
    if not isinstance(value, str):
        return ""
    return re.sub(r"\s+", " ", value).strip()

def extract_frontmatter_tags(frontmatter: dict) -> list[str]:
    """Normalize tags from either YAML strings or YAML lists."""
    raw_tags = frontmatter.get("tags", "")

    if isinstance(raw_tags, str):
        candidates = raw_tags.split(",")
    elif isinstance(raw_tags, list):
        candidates = [str(tag) for tag in raw_tags]
    else:
        return []

    tags = []
    for tag in candidates:
        normalized = tag.strip().replace('-', '_')
        if normalized and normalized not in tags:
            tags.append(normalized)
    return tags

def extract_category_and_tags(path: Path, frontmatter: dict) -> tuple[str, list[str]]:
    """Extract category and generate tags from path + frontmatter."""
    parts = path.relative_to(REPO_ROOT).parts
    
    # e.g., ('languages', 'python', 'vanrossum', 'SKILL.md')
    # or    ('domains', 'systems-architecture', 'lamport', 'SKILL.md')
    tags = []
    category = parts[0] if parts else "unknown"
    
    for part in parts[:-1]:  # Exclude SKILL.md
        if part not in ('SKILL.md',):
            tags.append(part.replace('-', '_'))
    
    for tag in extract_frontmatter_tags(frontmatter):
        if tag not in tags:
            tags.append(tag)
    
    return category, tags

def get_skill_files(skill_dir: Path) -> list[str]:
    """Get list of files in a skill directory."""
    files = []
    for f in skill_dir.iterdir():
        if f.is_file() and not f.name.startswith('.'):
            files.append(f.name)
    return sorted(files)

def generate_manifest() -> dict:
    """Generate the complete skills manifest."""
    skills = []
    
    for skill_path in sorted(REPO_ROOT.rglob(SKILL_FILE)):
        # Skip template
        if 'skill-template' in str(skill_path):
            continue
            
        skill_dir = skill_path.parent
        rel_path = skill_dir.relative_to(REPO_ROOT)
        
        content = skill_path.read_text()
        frontmatter = parse_frontmatter(content)
        category, tags = extract_category_and_tags(skill_path, frontmatter)
        
        # Extract engineer name from path
        engineer = skill_dir.name
        
        # Get subcategory (e.g., 'python' from 'languages/python/vanrossum')
        parts = rel_path.parts
        subcategory = parts[1] if len(parts) > 2 else None
        
        skill = {
            "id": frontmatter.get("name", engineer),
            "name": engineer,
            "description": normalize_manifest_text(frontmatter.get("description", "")),
            "category": category,
            "subcategory": subcategory,
            "path": str(rel_path),
            "files": get_skill_files(skill_dir),
            "tags": tags,
        }
        skills.append(skill)
    
    manifest = {
        "version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "repository": "https://github.com/copyleftdev/sk1llz",
        "raw_base_url": "https://raw.githubusercontent.com/copyleftdev/sk1llz/master",
        "skill_count": len(skills),
        "skills": skills,
    }
    
    return manifest

def main():
    manifest = generate_manifest()
    
    output_path = REPO_ROOT / "skills.json"
    with open(output_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"Generated {output_path} with {manifest['skill_count']} skills")

if __name__ == "__main__":
    main()
