#!/usr/bin/env bash
# mdbook preprocessor: Run sruja export on .sruja files and embed output in markdown.
#
# Usage in markdown:
#   <!-- sruja:export examples/advanced_views.sruja --all-views -->
#
# This gets replaced with the generated markdown output.

# First invocation: "supports" <renderer> -> exit 0 if we support it.
if [ "$1" = "supports" ]; then
  [ "$2" = "html" ] && exit 0 || exit 1
fi

# Second invocation: read JSON from stdin, process chapters, output modified book
tmp=$(mktemp -t mdbook-sruja.XXXXXX) || exit 1
trap 'rm -f "$tmp"' EXIT
cat > "$tmp"

# Use python3 to process the book JSON (temp path passed as argv so heredoc can stay quoted)
python3 - "$tmp" << 'PYTHON_SCRIPT'
import json
import subprocess
import re
import sys

def process_markdown(content):
    """Replace sruja export directives with generated output."""
    # Find all sruja export directives
    pattern = r'<!--\s*sruja:export\s+(.+?)\s*-->'

    def replace_directive(match):
        directive = match.group(1).strip()

        # Parse the directive
        parts = directive.split()
        if not parts:
            return match.group(0)

        # First part should be the file path
        file_path = parts[0]

        # Rest are flags
        flags = parts[1:] if len(parts) > 1 else []

        # Run sruja export
        try:
            cmd = ['sruja', 'export', 'markdown', file_path] + flags
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                check=True,
                cwd='.'
            )

            # Return the generated markdown
            return result.stdout

        except subprocess.CalledProcessError as e:
            # Return error message in HTML comment
            return f'<!-- Error: {e.stderr} -->'

    # Replace all directives
    return re.sub(pattern, replace_directive, content, flags=re.DOTALL)

# Read and process the book JSON (temp file path from argv).
# mdbook sends [context, book]; we must output the same structure.
with open(sys.argv[1], 'r') as f:
    payload = json.load(f)

if isinstance(payload, list) and len(payload) >= 2:
    context, book = payload[0], payload[1]
else:
    # Fallback: stdin is the book only (older or custom invocations)
    context, book = None, payload if isinstance(payload, dict) else {}

def process_section(section):
    if 'Chapter' in section:
        chapter = section['Chapter']
        if 'content' in chapter:
            chapter['content'] = process_markdown(chapter['content'])
        for sub in chapter.get('sub_items', []):
            process_section(sub)
    if 'Part' in section:
        part = section['Part']
        for ch in part.get('chapters', []):
            process_section(ch)
    if 'Preface' in section:
        preface = section['Preface']
        if 'content' in preface:
            preface['content'] = process_markdown(preface['content'])

# Book has "items" (array of Chapter/Part/Preface)
for section in book.get('items', book.get('sections', [])):
    process_section(section)

# Output: mdbook expects only the modified Book object (not the [context, book] array)
print(json.dumps(book))
PYTHON_SCRIPT
