#!/usr/bin/env python3
import json
import os
import subprocess
import time
import argparse
from pathlib import Path

def run_command(cmd, cwd=None, capture=True):
    """Run a shell command and return stdout/stderr."""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=capture, text=True)
    return result

def clone_repo(repo_url, target_dir):
    """Clone a repository if it doesn't exist."""
    if os.path.exists(target_dir):
        print(f"Repo already exists in {target_dir}")
        return True
    
    result = run_command(["git", "clone", "--depth", "1", repo_url, target_dir])
    return result.returncode == 0

def get_sruja_context(repo_path):
    """Generate architecture context using Sruja CLI."""
    result = run_command(["cargo", "run", "-p", "sruja-cli", "--", "context", "-r", repo_path, "--format", "markdown"])
    if result.returncode == 0:
        return result.stdout
    return "Error generating Sruja context."

def run_agent_task(prompt, repo_path, output_file, use_agent_cli=True):
    """Invoke an AI agent to solve the task."""
    if not use_agent_cli:
        print("Mocking agent call...")
        with open(output_file, "w") as f:
            f.write("# Mock Agent Output\nThis is a placeholder for the agent's work.")
        return 0, "mock-agent", 0

    # Using 'agent' CLI (Cursor/Claude Desktop style)
    # We include instructions to write the results to a specific file or dir
    # we'll assume the agent modifies the repo directly and we check the diff
    cmd = ["agent", "--trust", "-p", prompt]
    
    start_time = time.time()
    result = run_command(cmd, cwd=repo_path)
    duration = time.time() - start_time
    
    # In a real benchmark, we'd capture tokens if the tool supports it.
    # For now, we'll estimate or just record the duration.
    return result.returncode, result.stdout, duration

def evaluate_result(repo_path, eval_scripts, arch_file=None):
    """Evaluate architectural integrity and code correctness."""
    results = {
        "build_pass": False,
        "test_pass": False,
        "drift_violations": 0,
        "logs": []
    }
    
    # 1. Run eval scripts (tests/build)
    for script in eval_scripts:
        cmd = script.split()
        res = run_command(cmd, cwd=repo_path)
        results["logs"].append({"cmd": script, "code": res.returncode, "out": res.stdout[:1000]})
        if res.returncode == 0:
            results["build_pass"] = True
            results["test_pass"] = True # Simple heuristic
    
    # 2. Run sruja drift if context exists
    if arch_file and os.path.exists(arch_file):
        res = run_command(["cargo", "run", "-p", "sruja-cli", "--", "drift", "-r", repo_path, "-a", arch_file])
        # Simple count of violations
        violations = res.stdout.count("violation") + res.stdout.count("error")
        results["drift_violations"] = violations
        results["logs"].append({"cmd": "sruja drift", "code": res.returncode, "out": res.stdout})

    return results

def main():
    parser = argparse.ArgumentParser(description="Sruja AI Context Benchmark Harness")
    parser.add_argument("--tasks", default="benchmarks/tasks.json", help="Path to tasks.json")
    parser.add_argument("--output", default="benchmarks/results", help="Directory for results")
    parser.add_argument("--project", help="Filter by project name")
    parser.add_argument("--mock", action="store_true", help="Mock agent calls")
    args = parser.parse_args()

    # Load tasks
    with open(args.tasks, "r") as f:
        projects = json.load(f)

    os.makedirs(args.output, exist_ok=True)
    timestamp = int(time.time())
    run_dir = os.path.join(args.output, f"run_{timestamp}")
    os.makedirs(run_dir, exist_ok=True)

    summary = []

    for project_name, tasks in projects.items():
        if args.project and project_name != args.project:
            continue
        
        print(f"\n=== Benchmarking Project: {project_name} ===")
        repo_path = f"/tmp/sruja_bench_{project_name}"
        
        # We'll use a clean clone for each project to avoid state pollution between runs
        # but for simplicity in this script we'll just ensure it's there.
        if not clone_repo(tasks[0]["repo_url"], repo_path):
            print(f"Failed to clone {project_name}")
            continue

        # Get Sruja context once per project
        sruja_context = get_sruja_context(repo_path)

        for task in tasks:
            print(f"--- Task: {task['name']} ---")
            
            modes = ["baseline", "sruja"]
            task_results = {"task_id": task["id"], "modes": {}}

            for mode in modes:
                print(f"Running mode: {mode}")
                
                # Setup specific repo path for this mode/task
                mode_repo_path = os.path.join(repo_path, f"_{mode}_{task['id']}")
                run_command(["cp", "-R", repo_path, mode_repo_path])

                prompt = task["description"]
                if mode == "sruja":
                    prompt = f"Here is the architectural context for the repository:\n\n{sruja_context}\n\nTask: {prompt}"

                # Run Agent
                code, output, duration = run_agent_task(prompt, mode_repo_path, os.path.join(run_dir, f"{task['id']}_{mode}_raw.md"), use_agent_cli=not args.mock)
                
                # Evaluate
                eval_results = evaluate_result(mode_repo_path, task["eval_scripts"])
                eval_results["duration_sec"] = duration
                
                task_results["modes"][mode] = eval_results
                
            summary.append(task_results)

    # Save summary
    with open(os.path.join(run_dir, "summary.json"), "w") as f:
        json.dump(summary, f, indent=2)
    
    print(f"\nBenchmark complete. Results saved to {run_dir}/summary.json")

if __name__ == "__main__":
    main()
