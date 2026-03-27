# Sruja AI Context Benchmark

This benchmark evaluates the effectiveness of architecture-aware context (via Sruja) compared to default repository context in AI-assisted coding tasks.

## Goals

1. **Accuracy**: Measures if the agent identifies the correct architectural components.
2. **Efficiency**: Measures the token cost of the provided context.
3. **Correctness**: Measures if the suggested code follows architectural boundaries.

## Benchmark Harness Design

The harness (`benchmarks/harness.py`) will automate the following:

1. **Test Case Selection**: Uses a set of predefined tasks on target repos (e.g., "Add a new endpoint that persists data to X").
2. **Context Injection**:
   - **Baseline**: Standard RAG/Search provided by the editor.
   - **Sruja**: Sruja MCP tools enabled.
3. **Execution**: Runs an LLM (e.g., Claude 3.5 Sonnet) to solve the task.
4. **Scoring**:
   - **Pass/Fail**: Does the code compile and pass unit tests?
   - **Architectural Violation**: Does the code violate layers (checked via `sruja drift`)?
   - **Token Count**: Total tokens used in the prompt.

## Metrics

| Metric | Description | Target Improvement |
|--------|-------------|--------------------|
| **Discovery Success** | Identify all required dependencies. | +30% over baseline |
| **Boundary Violation** | Imports across forbidden layers. | -50% reduction |
| **Context Density** | Information gain per token. | 2x improvement |

## Next Steps

1. Implement `harness.py`.
2. Define the first 3 test cases for the `sruja` repo itself (dogfooding).
3. Run initial baseline vs. Sruja comparison.
