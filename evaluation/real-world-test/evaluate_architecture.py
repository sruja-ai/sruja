#!/usr/bin/env python3
"""
Simple evaluation script for Sruja-generated architectures

Usage:
    python evaluate_architecture.py <repo-name>
    python evaluate_architecture.py express
    python evaluate_architecture.py --all
"""

import argparse
import json
import os
import subprocess
from datetime import datetime
from pathlib import Path

# Try to import LLM libraries (optional)
try:
    import openai

    LLM_AVAILABLE = True
except ImportError:
    LLM_AVAILABLE = False


def check_file_exists(repo_path: Path) -> bool:
    """Check if architecture.sruja exists"""
    arch_file = repo_path / "architecture.sruja"
    if not arch_file.exists():
        print(f"❌ No architecture.sruja found in {repo_path}")
        print(f"\nTo generate one:")
        print(f"  cd {repo_path}")
        print(f"  # Use Sruja AI skills to analyze codebase and generate architecture")
        return False
    return True


def run_validation(repo_path: Path) -> dict:
    """Run sruja lint on generated architecture"""
    arch_file = repo_path / "architecture.sruja"
    results = {"valid": False, "errors": [], "warnings": []}

    try:
        result = subprocess.run(
            ["sruja", "lint", str(arch_file)],
            capture_output=True,
            text=True,
            timeout=30,
        )

        output = result.stdout + result.stderr

        if result.returncode == 0:
            results["valid"] = True
            print(f"✅ Validation passed")
        else:
            print(f"⚠️  Validation issues found:")
            print(output)
            results["errors"] = [
                line for line in output.split("\n") if "error" in line.lower()
            ]
            results["warnings"] = [
                line for line in output.split("\n") if "warning" in line.lower()
            ]

    except FileNotFoundError:
        print("⚠️  sruja CLI not found, skipping validation")
        print("   Install with: curl -fsSL https://sruja.ai/install.sh | bash")
    except Exception as e:
        print(f"⚠️  Validation failed: {e}")

    return results


def get_stats(repo_path: Path) -> dict:
    """Get basic statistics about the generated architecture"""
    arch_file = repo_path / "architecture.sruja"
    content = arch_file.read_text()

    stats = {
        "lines": len(content.split("\n")),
        "characters": len(content),
        "systems": content.count("= system"),
        "containers": content.count("= container"),
        "databases": content.count("= database") + content.count("= datastore"),
        "queues": content.count("= queue"),
        "persons": content.count("= person"),
        "relationships": content.count("->"),
    }

    return stats


def evaluate_with_llm(repo_path: Path, repo_name: str) -> dict:
    """Use LLM to evaluate the generated architecture"""
    if not LLM_AVAILABLE:
        return {"error": "OpenAI library not installed. Run: pip install openai"}

    if not os.getenv("OPENAI_API_KEY"):
        return {"error": "OPENAI_API_KEY not set"}

    arch_file = repo_path / "architecture.sruja"
    arch_content = arch_file.read_text()

    # Limit content size for API
    if len(arch_content) > 8000:
        arch_content = arch_content[:8000] + "\n... (truncated)"

    prompt = f"""You are a software architect evaluating a Sruja architecture DSL file generated for the {repo_name} codebase.

Generated architecture DSL:
```
{arch_content}
```

Evaluate this architecture on a scale of 1-10 for each criterion:

1. **Completeness** (1-10): Are main components, modules, and subsystems captured?
2. **Accuracy** (1-10): Does it likely match the actual {repo_name} architecture?
3. **Clarity** (1-10): Is it easy to understand the structure from this DSL?
4. **Usefulness** (1-10): Would this help a new developer understand {repo_name}?

Provide your response in this exact JSON format:
{{
  "completeness": <1-10>,
  "accuracy": <1-10>,
  "clarity": <1-10>,
  "usefulness": <1-10>,
  "average": <average of 4 scores>,
  "strengths": ["<strength 1>", "<strength 2>"],
  "weaknesses": ["<weakness 1>", "<weakness 2>"],
  "missing_components": ["<likely missing component 1>", "<component 2>"],
  "verdict": "<Useful/Partially Useful/Not Useful>"
}}
"""

    try:
        client = openai.OpenAI()
        response = client.chat.completions.create(
            model="gpt-4",
            messages=[{"role": "user", "content": prompt}],
            temperature=0.3,
            max_tokens=1000,
        )

        result_text = response.choices[0].message.content

        # Extract JSON from response
        import json

        json_start = result_text.find("{")
        json_end = result_text.rfind("}") + 1
        if json_start >= 0 and json_end > json_start:
            return json.loads(result_text[json_start:json_end])
        else:
            return {"error": "Could not parse LLM response", "raw": result_text}

    except Exception as e:
        return {"error": str(e)}


def manual_evaluation_checklist(repo_name: str):
    """Print manual evaluation checklist"""
    checklist = f"""
╔══════════════════════════════════════════════════════════════╗
║          Manual Evaluation Checklist for {repo_name:20s}    ║
╚══════════════════════════════════════════════════════════════╝

Please review the generated architecture.sruja and answer:

COMPLETENESS (Are main parts captured?)
  [ ] Main entry points identified
  [ ] Core modules/components documented
  [ ] Key data flows shown
  [ ] External dependencies included
  [ ] Important subsystems represented
  Score: ___/10

ACCURACY (Does it match the codebase?)
  [ ] Component names are correct
  [ ] Relationships reflect actual dependencies
  [ ] No fabricated/hallucinated components
  [ ] Technology choices are accurate
  [ ] Architecture patterns are correct
  Score: ___/10

CLARITY (Is it understandable?)
  [ ] Easy to see high-level structure
  [ ] Component purposes are clear
  [ ] Relationships are well-labeled
  [ ] Hierarchy makes sense
  [ ] Not overly complex
  Score: ___/10

USEFULNESS (Would it help developers?)
  [ ] Would speed up onboarding
  [ ] Reveals important design decisions
  [ ] Helps understand complexity
  [ ] Better than README alone
  [ ] Could guide architectural changes
  Score: ___/10

AVERAGE SCORE: ___/10

VERDICT:
  [ ] Useful (≥7/10)
  [ ] Partially Useful (5-6/10)
  [ ] Not Useful (<5/10)

NOTES:
______________________________________________________________
______________________________________________________________
______________________________________________________________
"""
    print(checklist)


def generate_report(
    repo_name: str, stats: dict, validation: dict, llm_eval: dict = None
):
    """Generate evaluation report"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_dir = Path("results")
    report_dir.mkdir(exist_ok=True)

    report_file = report_dir / f"evaluation_{repo_name}_{timestamp}.md"

    report = f"""# Architecture Evaluation Report: {repo_name}

**Date**: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

## File Statistics

- **Lines**: {stats["lines"]}
- **Characters**: {stats["characters"]}
- **Systems**: {stats["systems"]}
- **Containers**: {stats["containers"]}
- **Databases**: {stats["databases"]}
- **Queues**: {stats["queues"]}
- **Persons**: {stats["persons"]}
- **Relationships**: {stats["relationships"]}

## Validation

- **Status**: {"✅ Valid" if validation["valid"] else "❌ Issues found"}
- **Errors**: {len(validation.get("errors", []))}
- **Warnings**: {len(validation.get("warnings", []))}

"""

    if llm_eval and "error" not in llm_eval:
        report += f"""## LLM Evaluation

| Criterion | Score |
|-----------|-------|
| Completeness | {llm_eval.get("completeness", "N/A")}/10 |
| Accuracy | {llm_eval.get("accuracy", "N/A")}/10 |
| Clarity | {llm_eval.get("clarity", "N/A")}/10 |
| Usefulness | {llm_eval.get("usefulness", "N/A")}/10 |
| **Average** | **{llm_eval.get("average", "N/A")}/10** |

**Verdict**: {llm_eval.get("verdict", "N/A")}

### Strengths
"""
        for strength in llm_eval.get("strengths", []):
            report += f"- {strength}\n"

        report += "\n### Weaknesses\n"
        for weakness in llm_eval.get("weaknesses", []):
            report += f"- {weakness}\n"

        report += "\n### Likely Missing Components\n"
        for component in llm_eval.get("missing_components", []):
            report += f"- {component}\n"

    elif llm_eval and "error" in llm_eval:
        report += f"""## LLM Evaluation

**Error**: {llm_eval["error"]}

"""

    report += """## Manual Evaluation

See checklist above for manual evaluation.

## Next Steps

1. Review generated architecture in context of codebase
2. Compare with existing documentation (if any)
3. Identify gaps and inaccuracies
4. Provide feedback to improve Sruja

---
*Generated by Sruja Real-World Test Framework*
"""

    report_file.write_text(report)
    print(f"\n📄 Report saved to: {report_file}")
    return report_file


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate Sruja-generated architecture"
    )
    parser.add_argument("repo", help="Repository name or path")
    parser.add_argument(
        "--llm",
        action="store_true",
        help="Use LLM for evaluation (requires OPENAI_API_KEY)",
    )
    parser.add_argument(
        "--no-checklist", action="store_true", help="Skip manual checklist"
    )

    args = parser.parse_args()

    # Determine repo path
    if Path(args.repo).is_absolute():
        repo_path = Path(args.repo)
    else:
        repo_path = Path("test-repos") / args.repo

    if not repo_path.exists():
        print(f"❌ Repository not found: {repo_path}")
        print(f"\nAvailable repos:")
        test_repos = Path("test-repos")
        if test_repos.exists():
            for repo in test_repos.iterdir():
                if repo.is_dir() and not repo.name.startswith("."):
                    arch_file = repo / "architecture.sruja"
                    status = "✅" if arch_file.exists() else "⬜"
                    print(f"  {status} {repo.name}")
        return

    print(f"\n{'=' * 60}")
    print(f"Evaluating: {repo_path.name}")
    print(f"{'=' * 60}\n")

    # Check file exists
    if not check_file_exists(repo_path):
        return

    # Get statistics
    print("\n📊 Gathering statistics...")
    stats = get_stats(repo_path)
    print(f"  Lines: {stats['lines']}")
    print(
        f"  Components: {stats['systems']} systems, {stats['containers']} containers, {stats['databases']} databases"
    )
    print(f"  Relationships: {stats['relationships']}")

    # Run validation
    print("\n🔍 Running validation...")
    validation = run_validation(repo_path)

    # LLM evaluation (if requested)
    llm_eval = None
    if args.llm:
        print("\n🤖 Running LLM evaluation...")
        llm_eval = evaluate_with_llm(repo_path, repo_path.name)
        if "error" in llm_eval:
            print(f"  ⚠️  {llm_eval['error']}")
        else:
            print(f"  ✅ LLM evaluation complete")
            print(f"  Average score: {llm_eval.get('average', 'N/A')}/10")

    # Manual checklist
    if not args.no_checklist:
        manual_evaluation_checklist(repo_path.name)

    # Generate report
    report_file = generate_report(repo_path.name, stats, validation, llm_eval)

    print(f"\n{'=' * 60}")
    print("✅ Evaluation complete!")
    print(f"{'=' * 60}\n")


if __name__ == "__main__":
    main()
