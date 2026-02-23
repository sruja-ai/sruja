#!/usr/bin/env python3
"""
Setup script for real-world Sruja testing
Clones 5 popular repositories to test Sruja's architecture generation capabilities
"""

import subprocess
import os
from pathlib import Path

# Test repositories - diverse languages, frameworks, and complexity levels
REPOS = [
    {
        "name": "express",
        "url": "https://github.com/expressjs/express.git",
        "description": "Fast, unopinionated, minimalist web framework for Node.js",
        "language": "JavaScript",
        "complexity": "medium",
        "arch_type": "backend-framework"
    },
    {
        "name": "fastapi",
        "url": "https://github.com/tiangolo/fastapi.git",
        "description": "Modern, fast web framework for building APIs with Python",
        "language": "Python",
        "complexity": "medium",
        "arch_type": "backend-framework"
    },
    {
        "name": "next.js",
        "url": "https://github.com/vercel/next.js.git",
        "description": "The React Framework for Production",
        "language": "TypeScript",
        "complexity": "high",
        "arch_type": "fullstack-framework"
    },
    {
        "name": "prometheus",
        "url": "https://github.com/prometheus/prometheus.git",
        "description": "The Prometheus monitoring system and time series database",
        "language": "Go",
        "complexity": "high",
        "arch_type": "distributed-system"
    },
    {
        "name": "django",
        "url": "https://github.com/django/django.git",
        "description": "The Web framework for perfectionists with deadlines",
        "language": "Python",
        "complexity": "high",
        "arch_type": "fullstack-framework"
    }
]

def clone_repos(base_dir: Path):
    """Clone all test repositories"""
    repos_dir = base_dir / "test-repos"
    repos_dir.mkdir(exist_ok=True)

    print(f"📁 Setting up test repositories in: {repos_dir}\n")

    for repo in REPOS:
        repo_path = repos_dir / repo["name"]

        if repo_path.exists():
            print(f"✓ {repo['name']} already exists, skipping...")
            continue

        print(f"⬇️  Cloning {repo['name']}...")
        print(f"   {repo['description']}")
        print(f"   Language: {repo['language']} | Complexity: {repo['complexity']}")

        try:
            subprocess.run(
                ["git", "clone", "--depth", "1", repo["url"], str(repo_path)],
                check=True,
                capture_output=True
            )
            print(f"✅ Successfully cloned {repo['name']}\n")
        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to clone {repo['name']}: {e}\n")

def create_manifest(base_dir: Path):
    """Create a manifest file with repository information"""
    manifest_path = base_dir / "test-repos" / "MANIFEST.md"

    content = "# Test Repositories for Sruja Architecture Generation\n\n"
    content += "This directory contains popular open-source projects for testing Sruja's "
    content += "ability to generate useful architecture documentation.\n\n"
    content += "## Repositories\n\n"

    for i, repo in enumerate(REPOS, 1):
        content += f"### {i}. {repo['name']}\n"
        content += f"- **Description**: {repo['description']}\n"
        content += f"- **Language**: {repo['language']}\n"
        content += f"- **Complexity**: {repo['complexity']}\n"
        content += f"- **Architecture Type**: {repo['arch_type']}\n"
        content += f"- **URL**: {repo['url']}\n\n"

    manifest_path.write_text(content)
    print(f"📝 Created manifest at {manifest_path}\n")

def main():
    base_dir = Path(__file__).parent

    print("🚀 Sruja Real-World Test Setup")
    print("=" * 50)
    print("\nThis script will clone 5 popular repositories to test")
    print("Sruja's architecture generation capabilities.\n")

    clone_repos(base_dir)
    create_manifest(base_dir)

    print("=" * 50)
    print("✅ Setup complete!")
    print("\n📋 Next steps:")
    print("1. cd test-repos/<repo-name>")
    print("2. Use Sruja AI skills to generate architecture")
    print("   Example: Ask your AI assistant to 'Analyze this codebase and create a Sruja architecture DSL'")
    print("3. Review the generated .sruja file")
    print("4. Run evaluation: python evaluate_architecture.py <repo-name>")
    print("\n" + "=" * 50)

if __name__ == "__main__":
    main()
```
</file>
