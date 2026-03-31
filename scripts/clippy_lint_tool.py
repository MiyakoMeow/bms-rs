#!/usr/bin/env python3
"""
Clippy Lint Categorizer - Fetch lint metadata from official clippy documentation
and reorder Cargo.toml by group/applicability.

Usage:
    python3 clippy_lint_tool.py <Cargo.toml_path>           # Show categorized lints
    python3 clippy_lint_tool.py <Cargo.toml_path> --apply  # Apply reordering to file
"""

import re
import sys
import urllib.request
from pathlib import Path
from html.parser import HTMLParser
from typing import Dict, Tuple


CLIPPY_DOCS_URL = "https://rust-lang.github.io/rust-clippy/stable/index.html"

GROUP_ORDER = {
    'correctness': 0,
    'complexity': 1,
    'style': 2,
    'pedantic': 3,
    'perf': 4,
    'restriction': 5,
    'suspicious': 6,
    'nursery': 7,
    'cargo': 8,
    'deprecated': 9,
}

APPLICABILITY_ORDER = {
    'MachineApplicable': 0,
    'MaybeIncorrect': 1,
    'HasPlaceholders': 2,
    'Unspecified': 3,
}


class ClippyLintParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.lints: Dict[str, Tuple[str, str]] = {}
        self._in_article = False
        self._current_id = ""
        self._current_group = ""
        self._current_applicability = ""

    def handle_starttag(self, tag, attrs):
        if tag == "article":
            self._in_article = True
            for attr, val in attrs:
                if attr == "id":
                    self._current_id = val
                    break
        elif self._in_article and tag == "span":
            attr_dict = dict(attrs)
            cls = attr_dict.get("class", "")
            if "lint-group" in cls:
                for c in cls.split():
                    if c.startswith("group-"):
                        self._current_group = c[6:]
                        break
            elif "applicability" in cls:
                pass  # Applicability is set via handle_data

    def handle_endtag(self, tag):
        if tag == "article" and self._in_article:
            if self._current_id and self._current_group:
                self.lints[self._current_id] = (
                    self._current_group.title(),
                    self._current_applicability
                )
            self._in_article = False
            self._current_id = ""
            self._current_group = ""
            self._current_applicability = ""

    def handle_data(self, data):
        if self._in_article and self._current_group:
            stripped = data.strip()
            if stripped in ('MachineApplicable', 'MaybeIncorrect', 'HasPlaceholders', 'Unspecified'):
                self._current_applicability = stripped


def fetch_lint_metadata() -> Dict[str, Tuple[str, str]]:
    print(f"Fetching lint list from {CLIPPY_DOCS_URL}...")
    try:
        with urllib.request.urlopen(CLIPPY_DOCS_URL, timeout=30) as response:
            html = response.read().decode("utf-8")
    except Exception as e:
        print(f"Error fetching URL: {e}")
        sys.exit(1)

    parser = ClippyLintParser()
    parser.feed(html)
    print(f"Fetched {len(parser.lints)} lints")
    return parser.lints


LINT_METADATA: Dict[str, Tuple[str, str]] = {}


def get_lint_info(lint_name: str) -> Tuple[str, str]:
    if LINT_METADATA:
        return LINT_METADATA.get(lint_name, ('Unknown', 'Unspecified'))
    return ('Unknown', 'Unspecified')


def parse_cargo_toml_lints(content: str) -> Dict[str, str]:
    lints = {}
    in_clippy_section = False
    lines = content.split('\n')
    
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('[lints.clippy]'):
            in_clippy_section = True
            continue
        elif stripped.startswith('[lints.') and in_clippy_section:
            break
        elif in_clippy_section and '=' in stripped:
            if stripped.startswith('#'):
                continue
            if 'pedantic' in stripped and '{' in stripped:
                lints['pedantic'] = 'pedantic_override'
                continue
            if stripped.startswith('wildcard_imports'):
                lints['wildcard_imports'] = 'allow'
                continue
            match = re.match(r'^(\w+)\s*=\s*["\'](\w+)["\']', stripped)
            if match:
                lints[match.group(1)] = match.group(2)
    
    return lints


def categorize_lints(lints: Dict[str, str]) -> Dict[Tuple[str, str], list]:
    categorized = {}
    for name, level in lints.items():
        if name == 'pedantic':
            continue
        group, applicability = get_lint_info(name)
        key = (group, applicability)
        if key not in categorized:
            categorized[key] = []
        categorized[key].append((name, level))
    return categorized


def generate_toml_section(categorized: Dict[Tuple[str, str], list]) -> str:
    lines = ['[lints.clippy]', 'pedantic = { level = "allow", priority = -1 }', '']
    
    def sort_key(item):
        (group, applicability), _ = item
        g_order = GROUP_ORDER.get(group.lower(), 999)
        a_order = APPLICABILITY_ORDER.get(applicability, 999)
        return (g_order, a_order)
    
    for (group, applicability), lints_list in sorted(categorized.items(), key=sort_key):
        if not lints_list:
            continue
        lines.append(f'# {group} (Applicability: {applicability})')
        for name, level in sorted(lints_list):
            lines.append(f'{name} = "{level}"')
        lines.append('')
    
    return '\n'.join(lines)


def apply_to_file(cargo_path: Path) -> None:
    content = cargo_path.read_text()
    
    clippy_start = content.find('[lints.clippy]')
    if clippy_start == -1:
        print("Error: [lints.clippy] section not found")
        sys.exit(1)
    
    before_lints = content[:clippy_start]
    after_lints_section = content[clippy_start:]
    
    end_idx = after_lints_section.find('\n[lints.')
    if end_idx == -1:
        end_idx = len(after_lints_section)
    after_clippy = after_lints_section[end_idx:]
    
    clippy_section = after_lints_section[:end_idx]
    lints = parse_cargo_toml_lints(clippy_section)
    categorized = categorize_lints(lints)
    new_clippy = generate_toml_section(categorized)
    
    new_content = before_lints + new_clippy + after_clippy
    cargo_path.write_text(new_content)


def print_summary(categorized: Dict[Tuple[str, str], list]) -> None:
    print("\nCategorized lints:")
    
    def sort_key(item):
        (group, applicability), _ = item
        return (GROUP_ORDER.get(group.lower(), 999), APPLICABILITY_ORDER.get(applicability, 999))
    
    for (group, applicability), lints_list in sorted(categorized.items(), key=sort_key):
        print(f"  {group} (Applicability: {applicability}): {len(lints_list)} lints")


def main():
    global LINT_METADATA
    
    if len(sys.argv) < 2 or '--help' in sys.argv:
        print(__doc__)
        return
    
    LINT_METADATA = fetch_lint_metadata()
    
    cargo_path = Path(sys.argv[1])
    if not cargo_path.exists():
        print(f"Error: {cargo_path} not found")
        sys.exit(1)
    
    content = cargo_path.read_text()
    
    clippy_start = content.find('[lints.clippy]')
    if clippy_start == -1:
        print("Error: [lints.clippy] section not found")
        sys.exit(1)
    clippy_end = content.find('\n[lints.', clippy_start + 1)
    if clippy_end == -1:
        clippy_end = len(content)
    clippy_section = content[clippy_start:clippy_end]
    
    lints = parse_cargo_toml_lints(clippy_section)
    print(f"Found {len(lints)} lints in [lints.clippy]")
    
    categorized = categorize_lints(lints)
    print_summary(categorized)
    
    new_section = generate_toml_section(categorized)
    
    if '--apply' in sys.argv:
        apply_to_file(cargo_path)
        print(f"\nUpdated {cargo_path}")
    elif '--dry-run' in sys.argv:
        print("\n--- Generated lints section ---")
        print(new_section)
    else:
        print("\n--- Generated lints section (use --apply to write, --dry-run to preview) ---")
        print(new_section)


if __name__ == '__main__':
    main()
