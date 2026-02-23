# Valid Book Examples

These `.sruja` files are canonical runnable examples derived from the book. They are intended to pass `sruja lint` and serve as reference implementations.

## Usage

```bash
sruja lint book/valid-examples/*.sruja
```

## Scope

Not all DSL blocks in the book are complete runnable programs. Some are:

- **Illustrative snippets** – Show a single concept (e.g. one relationship) without full context
- **Comparative examples** – Show other notations (C4, DFD) for design-philosophy comparison
- **Progressive examples** – Build up incrementally across a lesson

This folder contains only **complete, self-contained** examples that validate successfully. CI runs `sruja lint` on all `**/*.sruja` files, including these.
