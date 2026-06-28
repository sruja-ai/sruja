# Taste (Continuously Learned by [CommandCode][cmd])

[cmd]: https://commandcode.ai/

# architecture
See [architecture/taste.md](architecture/taste.md)

# code-style
- Do not import or copy code from the acengage-orchestrator project into the sruja codebase; implement inspired patterns as original Rust code. Confidence: 0.75
- Make multi-agent pipeline features configuration-driven via config files (e.g., pipeline.toml, agent markdown prompts) rather than hardcoded values. Confidence: 0.80
- Pipeline stages and agent prompts should be dynamically driven by the goal, not hardcoded — the pipeline should adapt to what the goal requires. Confidence: 0.75

# model-routing
- For the multi-agent pipeline: use GLM-5.2 for analysis/self-review, GLM-5.1 for QA test generation, Mimo-2.5-Pro for QA review, GLM-4.7 for implementation, and Mimo-2.5 for implementation review. Confidence: 0.70
