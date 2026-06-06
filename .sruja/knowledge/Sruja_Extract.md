# Sruja Extract

> Automatic discovery of architectural artifacts from codebases.

## Purpose

Discovers architectural artifacts from a codebase using a pluggable extractor framework. Finds OpenAPI specs, Kubernetes manifests, Dockerfiles, Terraform configs, Helm charts, Protobuf schemas, GraphQL schemas, AsyncAPI specs, config files, documentation, aliases, and dependencies.

## Responsibilities

- Discover OpenAPI, AsyncAPI, Protobuf, GraphQL specs
- Discover Kubernetes, Docker, Terraform, Helm configs
- Extract documentation, aliases, and dependencies
- Use lazy file I/O (`FileContext`) to avoid redundant reads
- Generate extraction reports with statistics

## Dependencies

- **Internal**: Sruja_Language
- **External**: serde, serde_json, thiserror, log, ignore

## Key Types

- `ExtractionEngine` — Main engine that runs all extractors
- `ExtractionConfig` — Configuration for extraction
- `ExtractionReport`, `ExtractionStats` — Extraction results
- `Extractor` — Trait for pluggable extractors
- `FileContext` — Lazy file I/O context
- `DiscoveredSource` — Discovered architectural artifact

## Built-in Extractors (12)

- `OpenApiExtractor`, `AsyncApiExtractor` — API specs
- `KubernetesExtractor`, `DockerfileExtractor`, `TerraformExtractor`, `HelmExtractor` — Infrastructure
- `ProtoExtractor`, `GraphqlExtractor` — Schema languages
- `ConfigExtractor` — Configuration files
- `DocExtractor` — Documentation
- `AliasExtractor` — Module aliases
- `DependencyExtractor` — Package dependencies

## Code Locations

- `crates/sruja-extract/` — Extract crate
- `src/lib.rs` — Engine and trait definitions
- `src/openapi/`, `src/kubernetes/`, etc. — Individual extractors

---
*Last updated: 2026-06-06*
