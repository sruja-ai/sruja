# Testing

## Quick verify (CLI)

From repo root, run:

```bash
# Build and run all workspace tests
cargo build --release -p sruja-cli
cargo test --workspace

# Extraction CLI contracts (lint JSON, discover JSON)
cargo test -p sruja-cli --test extraction_cli

# Optional: quickstart on this repo
./target/release/sruja quickstart -r .
./target/release/sruja lint book/valid-examples/getting-started.sruja --format json
./target/release/sruja discover --context -r . --format json
```

## Make targets

- `just test` (or `make test`) – `cargo test --workspace`
- `just test-rust` (or `make test-rust`) – `cargo test --workspace`
- `just test-extraction` (or `make test-extraction`) – extraction_cli tests (lint/discover JSON schema)
- `just test-arch-intel` (or `make test-arch-intel`) – why_e2e tests
- `just build` (or `make build`) – release build of CLI

## Comprehensive test (external repos)

See [comprehensive_test.sh](comprehensive_test.sh). Clones sample repos and runs quickstart; requires network and time.
