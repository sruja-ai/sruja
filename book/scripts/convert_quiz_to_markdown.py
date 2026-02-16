#!/usr/bin/env python3
"""
Convert TOML quiz files to markdown format with checkboxes.

This script converts mdbook-quiz format TOML files to simple markdown
with checkboxes that work with quiz-helper.js.

Usage:
    python convert_quiz_to_markdown.py <input.toml> [output.md]
    python convert_quiz_to_markdown.py --all-quizzes
"""

import os
import re
import sys
from pathlib import Path

try:
    import tomllib

    USING_TOMLLIB = True
except ImportError:
    try:
        import toml

        USING_TOMLLIB = False
    except ImportError:
        print("Error: Please install toml library: pip install toml")
        sys.exit(1)


def escape_html(text):
    """Escape HTML special characters."""
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


def convert_multiple_choice(question, question_num):
    """Convert a MultipleChoice question to markdown format."""
    lines = []

    # Question text
    prompt = question.get("prompt", {}).get("prompt", "")
    lines.append(f"**{question_num}. {prompt}**\n")

    # Prepare options
    answer = question.get("answer", {}).get("answer", "")
    distractors = question.get("prompt", {}).get("distractors", [])

    # Combine answer and distractors
    all_options = distractors + [answer]

    # Determine correct answer index
    correct_index = len(all_options) - 1  # Answer is last

    # Randomize if not using answerIndex
    # For simplicity, we'll keep order but mark the correct one
    # In production, you might want to randomize

    # Create checkbox list
    for idx, option in enumerate(all_options):
        letter = chr(97 + idx)  # a, b, c, d, ...
        lines.append(f"- [ ] {letter}) {option}")

    lines.append("")

    # Add check button
    correct_letter = chr(97 + correct_index)
    lines.append(
        f'<button class="check-answer-btn" data-correct="{correct_letter}">Check Answer</button>'
    )
    lines.append("")

    # Add feedback div
    lines.append('<div class="answer-feedback" style="display: none;">')
    lines.append('  <p class="feedback-text"></p>')

    # Add explanation if available
    context = question.get("context", "")
    if context:
        lines.append('  <div class="explanation" style="display: none;">')
        lines.append(f"    {context}")
        lines.append("  </div>")

    lines.append("</div>")
    lines.append("")
    lines.append("---")
    lines.append("")

    return "\n".join(lines)


def convert_short_answer(question, question_num):
    """Convert a ShortAnswer question to details/summary format."""
    lines = []

    # Question text
    prompt = question.get("prompt", {}).get("prompt", "")
    lines.append(f"**{question_num}. {prompt}**\n")

    # Answer
    answer = question.get("answer", {}).get("answer", "")
    alternatives = question.get("answer", {}).get("alternatives", [])

    # Create details/summary for answer
    lines.append("<details>")
    lines.append("<summary><strong>Click to see answer</strong></summary>")
    lines.append("")
    lines.append(f"**Answer:** {answer}")

    if alternatives:
        lines.append("")
        lines.append("**Alternative answers:**")
        for alt in alternatives:
            lines.append(f"- {alt}")

    # Add context if available
    context = question.get("context", "")
    if context:
        lines.append("")
        lines.append("**Explanation:**")
        lines.append(context)

    lines.append("")
    lines.append("</details>")
    lines.append("")
    lines.append("---")
    lines.append("")

    return "\n".join(lines)


def convert_tracing(question, question_num):
    """Convert a Tracing question to details/summary format."""
    lines = []

    # Question text
    lines.append(f"**{question_num}. Trace the following program:**\n")

    # Program code
    program = question.get("prompt", {}).get("program", "")
    lines.append("```")
    lines.append(program.strip())
    lines.append("```\n")

    # Answer
    does_compile = question.get("answer", {}).get("doesCompile", True)
    stdout = question.get("answer", {}).get("stdout", "")
    context = question.get("context", "")

    # Create details/summary for answer
    lines.append("<details>")
    lines.append("<summary><strong>Click to see answer</strong></summary>")
    lines.append("")

    if does_compile:
        lines.append("**Program compiles successfully**")
        if stdout:
            lines.append("")
            lines.append("**Output:**")
            lines.append("```")
            lines.append(stdout)
            lines.append("```")
    else:
        lines.append("**Program does NOT compile**")

    if context:
        lines.append("")
        lines.append("**Explanation:**")
        lines.append(context)

    lines.append("")
    lines.append("</details>")
    lines.append("")
    lines.append("---")
    lines.append("")

    return "\n".join(lines)


def convert_question(question, question_num):
    """Convert a single question based on its type."""
    q_type = question.get("type", "")

    if q_type == "MultipleChoice":
        return convert_multiple_choice(question, question_num)
    elif q_type == "ShortAnswer":
        return convert_short_answer(question, question_num)
    elif q_type == "Tracing":
        return convert_tracing(question, question_num)
    else:
        return f"\n<!-- Unknown question type: {q_type} -->\n\n"


def convert_toml_to_markdown(toml_path, output_path=None):
    """Convert a TOML quiz file to markdown format."""

    # Read TOML file
    try:
        if USING_TOMLLIB:
            # tomllib (Python 3.11+) requires binary mode
            with open(toml_path, "rb") as f:
                data = tomllib.load(f)
        else:
            # toml library (Python < 3.11) - can accept filename directly
            data = toml.load(toml_path)
    except Exception as e:
        print(f"Error parsing {toml_path}: {e}")
        return False

    # Extract questions
    questions = data.get("questions", [])

    if not questions:
        print(f"No questions found in {toml_path}")
        return False

    # Generate markdown
    markdown_lines = []
    markdown_lines.append("<!-- Auto-generated quiz from TOML -->")
    markdown_lines.append(
        "<!-- Source: {} -->".format(
            toml_path.name if hasattr(toml_path, "name") else toml_path
        )
    )
    markdown_lines.append("")

    # Convert each question
    for idx, question in enumerate(questions, 1):
        markdown_lines.append(convert_question(question, idx))

    markdown_content = "\n".join(markdown_lines)

    # Determine output path
    if output_path is None:
        output_path = Path(toml_path).with_suffix(".md")

    # Write output
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(markdown_content)

    print(f"✓ Converted {toml_path} -> {output_path} ({len(questions)} questions)")
    return True


def convert_all_quizzes(quizzes_dir):
    """Convert all TOML quiz files in a directory."""

    quizzes_path = Path(quizzes_dir)

    if not quizzes_path.exists():
        print(f"Error: Directory not found: {quizzes_dir}")
        return False

    # Find all TOML files
    toml_files = list(quizzes_path.rglob("*.toml"))

    if not toml_files:
        print(f"No TOML files found in {quizzes_dir}")
        return False

    print(f"Found {len(toml_files)} quiz files to convert\n")

    success_count = 0
    for toml_file in sorted(toml_files):
        # Skip if it's a config file
        if toml_file.name in ["config.toml", "settings.toml"]:
            continue

        # Convert the file
        if convert_toml_to_markdown(toml_file):
            success_count += 1

    print(f"\n✓ Successfully converted {success_count}/{len(toml_files)} files")
    return True


def main():
    """Main entry point."""

    if len(sys.argv) < 2:
        print(__doc__)
        print("\nExamples:")
        print("  python convert_quiz_to_markdown.py lesson-1-quiz.toml")
        print(
            "  python convert_quiz_to_markdown.py lesson-1-quiz.toml lesson-1-quiz.md"
        )
        print("  python convert_quiz_to_markdown.py --all-quizzes ../src/quizzes")
        sys.exit(1)

    if sys.argv[1] == "--all-quizzes":
        # Convert all quizzes in directory
        quizzes_dir = sys.argv[2] if len(sys.argv) > 2 else "../src/quizzes"
        success = convert_all_quizzes(quizzes_dir)
        sys.exit(0 if success else 1)
    else:
        # Convert single file
        toml_path = sys.argv[1]
        output_path = sys.argv[2] if len(sys.argv) > 2 else None

        if not Path(toml_path).exists():
            print(f"Error: File not found: {toml_path}")
            sys.exit(1)

        success = convert_toml_to_markdown(toml_path, output_path)
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
