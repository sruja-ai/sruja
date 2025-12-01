---
title: "Releasing with GoReleaser"
weight: 90
summary: "Build cross-platform binaries and publish archives that match the installer."
tags: [release, goreleaser]
aliases: ["/tutorials/releasing-with-goreleaser/"]
---

# Releasing with GoReleaser

Sruja uses GoReleaser to build and publish releases.

## Configuration

`.goreleaser.yaml` sets builds and archive names:

```yaml
version: 2
project_name: sruja
builds:
  - id: sruja
    main: ./cmd/sruja
    env:
      - CGO_ENABLED=0
    goos: [linux, darwin]
    goarch: [amd64, arm64]
archives:
  - ids: [sruja]
    formats: ["tar.gz"]
    name_template: '{{ .ProjectName }}_{{ .Os | title }}_{{- if eq .Arch "amd64" -}}x86_64{{- else -}}{{ .Arch }}{{- end -}}'
```

## Snapshot Build

```bash
goreleaser release --snapshot --clean
```

This produces `sruja_Darwin_arm64.tar.gz`, `sruja_Linux_x86_64.tar.gz`, etc., which the installer consumes.

