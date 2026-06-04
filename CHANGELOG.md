# Changelog

All notable changes to Sruja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.58.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.57.0...sruja-v0.58.0) (2026-06-04)


### Features

* add barrel file exclusion to reduce drift noise ([5d7dd94](https://github.com/sruja-ai/sruja/commit/5d7dd9490d6c9bee62e3ddf893cd25e8ecd55600))
* **graph:** add temporal graph tracking - snapshots, history, drift velocity, score trends ([0bb5fc1](https://github.com/sruja-ai/sruja/commit/0bb5fc150e739c670aa1dd8cfe5d36eef38831f7))
* **requirements:** enrich DSL with PRD-aligned fields and wire into enforcement ([8e393ae](https://github.com/sruja-ai/sruja/commit/8e393ae854f96412377b4dde7198184b8e5e0142))

## [0.57.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.56.0...sruja-v0.57.0) (2026-06-02)


### Features

* add classification generation to init --scan ([7d21e7f](https://github.com/sruja-ai/sruja/commit/7d21e7f32291b8e71ddfb13b8be7c7d861c238e9))
* make human commands work without federation ([eaa315b](https://github.com/sruja-ai/sruja/commit/eaa315bcffd370728d311335a6332e57c4a421c7))
* Phase 1 human commands — map, trace, explain, before, what-if ([beee897](https://github.com/sruja-ai/sruja/commit/beee897150cc35c1337487efa1962fc0f5895bd4))


### Bug Fixes

* restore original repo.sruja with Agent container ([0f13ee6](https://github.com/sruja-ai/sruja/commit/0f13ee6040bd396155739dc94cb5dca54bff75f5))

## [0.56.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.55.1...sruja-v0.56.0) (2026-06-01)


### Features

* add confidence report command for post-AI-edit review ([53fe826](https://github.com/sruja-ai/sruja/commit/53fe82667216e5f5f6508035dc2ec9fd53d831f3))


### Bug Fixes

* **ci:** align IDE rules with CI and gate deploys with book E2E ([85c1cbe](https://github.com/sruja-ai/sruja/commit/85c1cbe888ee3893e36613395c5f038562d1d2dd))
* **ci:** cache Playwright and extend deploy timeout for book E2E ([4cb5e77](https://github.com/sruja-ai/sruja/commit/4cb5e7750b55479f8350d036745707c66b865b21))
* **ci:** install mdbook-mermaid assets before deploy book build ([50e372a](https://github.com/sruja-ai/sruja/commit/50e372a1d919194bd398fd96d5aa56dc4a5b12e8))
* **ci:** run deploy book E2E in Playwright container job ([c99d736](https://github.com/sruja-ai/sruja/commit/c99d736ca285af2cac92d00d243b706b3d5fb39b))
* **ci:** serve book with http-server for WASM MIME in deploy E2E ([d0a35dc](https://github.com/sruja-ai/sruja/commit/d0a35dcc48fd6bb08f7c5858068f800174be10a2))
* **ci:** serve built book statically for deploy E2E ([542bc3f](https://github.com/sruja-ai/sruja/commit/542bc3f70bf419b1444e13e500e4ab740b2d0e4f))
* **ci:** stop mdbook serve after deploy E2E smoke ([d2615d7](https://github.com/sruja-ai/sruja/commit/d2615d733223e654a8ce065329f017504f516b48))
* **ci:** use minimal Sruja page for deploy book E2E ([16f1899](https://github.com/sruja-ai/sruja/commit/16f1899db29f7ad2a0139e86b410926b77502417))
* **ci:** use tcp URI for serve@14 in book deploy E2E ([8589784](https://github.com/sruja-ai/sruja/commit/8589784d0e163df261548a3484f94346d31f6872))

## [0.55.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.55.0...sruja-v0.55.1) (2026-06-01)


### Bug Fixes

* unblock CI after ownership tests and extension type mismatch ([ae14f6b](https://github.com/sruja-ai/sruja/commit/ae14f6b8ee32b91ae3c604500e480a3e7e147413))

## [0.55.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.54.1...sruja-v0.55.0) (2026-06-01)


### Features

* compact focus, verification hashes, and DSL integrity rules ([78e62d2](https://github.com/sruja-ai/sruja/commit/78e62d25ab6235ed1e9ae0702ecdccfc70e6b502))
* human-centric system intelligence commands ([0e99642](https://github.com/sruja-ai/sruja/commit/0e9964215e07d069d12b63d5ef4f16921daf4335))


### Bug Fixes

* **extension:** mock createTextEditorDecorationType for diagnostics tests ([2a5c977](https://github.com/sruja-ai/sruja/commit/2a5c9775c561b090369d9236be198a637544817e))

## [0.54.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.54.0...sruja-v0.54.1) (2026-05-31)


### Bug Fixes

* **cli:** satisfy clippy unnecessary_sort_by in skill stats ([c77e8a5](https://github.com/sruja-ai/sruja/commit/c77e8a56734a166afe545076462bf1d66e23f574))

## [0.54.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.53.0...sruja-v0.54.0) (2026-05-30)


### Features

* agent memory lifecycle - decay, distillation, session handoff, skill tracking ([b6599b8](https://github.com/sruja-ai/sruja/commit/b6599b80dc91595a1d8313f70607cfb87d411f20))

## [0.53.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.52.1...sruja-v0.53.0) (2026-05-29)


### Features

* hybrid architecture classification with MCP tools for AI agents ([1ca85a8](https://github.com/sruja-ai/sruja/commit/1ca85a81676486b62b735d4d4d146fc6dd84b81a))
* improve UX with status bar, architecture sidebar, gutter decorations, and welcome walkthrough ([6f7664a](https://github.com/sruja-ai/sruja/commit/6f7664acf1e02ed0aafcc766818f27e97ccee279))

## [0.52.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.52.0...sruja-v0.52.1) (2026-05-28)


### Bug Fixes

* **ci:** checkout-saga lint and sruja-memory clippy ([#188](https://github.com/sruja-ai/sruja/issues/188)) ([0c94c05](https://github.com/sruja-ai/sruja/commit/0c94c0531963c25e5adc8b5d245825b7e929f623))

## [0.52.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.51.0...sruja-v0.52.0) (2026-05-27)


### Features

* **enterprise:** add baselines, evidence packs, and multi-CI templates ([8951473](https://github.com/sruja-ai/sruja/commit/8951473b8615866a7a3ef00bad6c0397a42c71ee))
* strategic pivot to agent-native grounding layer and slim mcp default ([919b921](https://github.com/sruja-ai/sruja/commit/919b921680f1d837af91ea01889503aed3f11ecc))
* **telemetry:** add host, skills_used, and session_id tracking to context events ([9534be1](https://github.com/sruja-ai/sruja/commit/9534be1d5a9ad18420626a485c0637679021d983))
* **workflow:** implement e2e workflow lifecycle, readiness checklist, and structured design reviews ([877a585](https://github.com/sruja-ai/sruja/commit/877a58558725e5e38dfc2a47a322676b943e1ddc))


### Bug Fixes

* **ci:** increase verify-task step timeout ([47b3335](https://github.com/sruja-ai/sruja/commit/47b33355cb0d58e42a99df579e9a871e2513349c))
* **workflows:** use checkout@v4 and simplify drift ([6985e8e](https://github.com/sruja-ai/sruja/commit/6985e8e9afcb8ed3fcaa5182de9d5109c25e5c8c))

## [0.51.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.50.0...sruja-v0.51.0) (2026-05-25)


### Features

* **cli:** add verify-task CLI command and MCP tool ([e0307da](https://github.com/sruja-ai/sruja/commit/e0307daef2128bb46e78c449462f9cefde373684))
* drift-first OSS packaging ([f21dc60](https://github.com/sruja-ai/sruja/commit/f21dc60a6fd200bb668335b5835fa9cb191a40de))
* **explorer:** add architecture explorer ([2a6a9f3](https://github.com/sruja-ai/sruja/commit/2a6a9f36320277ec0439e4f0599ce40401ef591c))
* implement MCP tool profiles (Phase T feature tightening) ([32634b7](https://github.com/sruja-ai/sruja/commit/32634b72f0ff72da62264ec5bb0de34687bd9d39))

## [Unreleased]

### OSS traction packaging

- **Drift:** `--structural-only` and `--advisory` on `sruja drift`; scan scope summary, `could_not_infer`, clean-scan line in text/JSON.
- **Quickstart / init:** `--advisory` on quickstart; `sruja start` hero messaging; init recommends structural drift after setup.
- **CLI:** Many Tier-3 commands hidden from `sruja --help`; `agent run` hidden (use `agent plan` + host `verify-task`).
- **MCP:** Default `coding` profile (15 tools); `sruja_agent_run` not in profile (still in `full` only).
- **Docs:** [docs/FEATURE_TIERS.md](docs/FEATURE_TIERS.md), [docs/STRUCTURIZR_VS_SRUJA.md](docs/STRUCTURIZR_VS_SRUJA.md), [docs/OSS_METRICS.md](docs/OSS_METRICS.md); [docs/MESSAGING.md](docs/MESSAGING.md) drift-first Tier 1a/1b.
- **Demo:** [examples/oss-demo/](examples/oss-demo/) pinned structural drift JSON + CI fixture test.

### Deprecation (next minor — remove after callers updated)

| Use instead | Deprecated |
|-------------|------------|
| `sruja drift --ci` | `sruja check` |
| `sruja start` | `sruja quickstart`, `sruja overview`, `sruja onboard` (hidden) |
| `sruja status` | `sruja doctor` (hidden alias) |
| `sruja review` | `sruja daily` (hidden alias) |
| Host `verify-task` | `sruja agent run` (hidden) |

## [0.50.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.49.0...sruja-v0.50.0) (2026-05-20)


### Features

* **cli:** add Phase 3 active context management ([3775fc6](https://github.com/sruja-ai/sruja/commit/3775fc6d82482871418c0cfc7c1c57ed3aa83697))
* **cli:** complete Phase 3 exit — watch_drift, prune fixture, agent apply ([07dd6c9](https://github.com/sruja-ai/sruja/commit/07dd6c9622743840168a80c37d8e610a3665410e))
* **cli:** Phase 3 exit — drift-state, IDE refresh, CI sync check ([8e6a09f](https://github.com/sruja-ai/sruja/commit/8e6a09fa634e59c36d41413c85e1ed2823656baf))
* **cli:** wire Phase 3/4 context host for Cursor MCP ([0282dc6](https://github.com/sruja-ai/sruja/commit/0282dc6ab2ac8d068ede7e3483bb2d3e13445db9))
* **memory:** add Phase 4 indexed memory store and MCP search ([220a9b7](https://github.com/sruja-ai/sruja/commit/220a9b71782e80affed3f130d0430db8af51e2e7))
* workflow AIDLC integration ([2607ad2](https://github.com/sruja-ai/sruja/commit/2607ad2d5612b5edae094ccfddfd85ddc52a03d7))


### Bug Fixes

* **cli:** flatten MCP test module and document layout ([ac99ab3](https://github.com/sruja-ai/sruja/commit/ac99ab3b9003dff5e32375ede74cbb99706e5a1f))

## [0.49.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.48.0...sruja-v0.49.0) (2026-05-18)


### Features

* **cli:** add MCP context ladder and cache-friendly AI exports ([10afe5e](https://github.com/sruja-ai/sruja/commit/10afe5eef135f35682e5ee1d43b3728b07ed4e3a))
* **cli:** add MCP resources, prompts, and sync-ide-rules ([33edacf](https://github.com/sruja-ai/sruja/commit/33edacfb19cf3256f969f06130321cb60c2adf33))

## [0.48.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.47.0...sruja-v0.48.0) (2026-05-15)


### Features

* **cli:** add sruja learn pipeline and MCP tools for learned facts ([5290d50](https://github.com/sruja-ai/sruja/commit/5290d50dffe3049a4b3273068a1ef969e1f27966))
* universalize sruja to generic context graphs (Phase 1) ([3f9038e](https://github.com/sruja-ai/sruja/commit/3f9038e5b9b496030e20c64f1769177d86e9eba6))

## [0.47.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.46.0...sruja-v0.47.0) (2026-05-13)


### Features

* **cli:** context lineage events, temporal focus, MCP tools, facts bundle ([631cc52](https://github.com/sruja-ai/sruja/commit/631cc52770091fe6a3e9241d4cfc90629d405176))
* **dsl:** relation checks, parse recovery, book partial markers ([30093ef](https://github.com/sruja-ai/sruja/commit/30093ef5227de9085cc1cd73fde7487e3f99bacf))


### Bug Fixes

* **ci:** repair deploy-to-github-pages action manifest YAML ([b0a0965](https://github.com/sruja-ai/sruja/commit/b0a096585941225ab34ace46bfca9d5cd56ddcd5))

## [0.46.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.45.1...sruja-v0.46.0) (2026-05-13)


### Features

* **agent:** add plan/apply workflow, baseline modes, and sandboxed trajectories ([1ca86f2](https://github.com/sruja-ai/sruja/commit/1ca86f2717d16b7aa2f12bb3cea8a71d9feb49ba))
* **agent:** add run ids, memory provenance, and run snapshots ([64c844e](https://github.com/sruja-ai/sruja/commit/64c844e49efa462ec0ef38048005f70ac8974b54))
* **cli:** add task context grounding trace ([dd1fbbc](https://github.com/sruja-ai/sruja/commit/dd1fbbc614347c93a5c9f3dec9922ef216dff44f))
* **cli:** MCP readonly profile, call logging, and tool docs ([c725bd5](https://github.com/sruja-ai/sruja/commit/c725bd5af1cfded3ac23a5c5db095affb9463e0a))

## [0.45.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.45.0...sruja-v0.45.1) (2026-05-13)


### Bug Fixes

* **ci:** Pages README from template; tighten deploy docs ([5f20e84](https://github.com/sruja-ai/sruja/commit/5f20e84dccdda347dfc0d8e9c13975e9e0ecb24c))

## [0.45.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.44.1...sruja-v0.45.0) (2026-05-12)


### Features

* **extract:** overhaul extraction framework for robustness and AI coding support ([80829c5](https://github.com/sruja-ai/sruja/commit/80829c5692ea369cd02928044f0adcdffb78471a))


### Bug Fixes

* **extract:** remove AI slop — fix override bug, dead code, and boilerplate ([7d60505](https://github.com/sruja-ai/sruja/commit/7d605053de7ed22d297bcea09c1f7a710a096d54))

## [0.44.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.44.0...sruja-v0.44.1) (2026-05-12)


### Bug Fixes

* **ci:** resolve clippy lint and linux release build failures ([f08c2e5](https://github.com/sruja-ai/sruja/commit/f08c2e5cc271b486f0b37c9253b51d2014a4b860))

## [0.44.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.43.1...sruja-v0.44.0) (2026-05-12)


### Features

* add context engineering capabilities — BM25 retrieval, adaptive hybrid query, MaTTS, Zettelkasten memory ([58286c3](https://github.com/sruja-ai/sruja/commit/58286c399aaae8031b6b3261fdde512b53e13076))
* **book:** Add new courses for Sruja features ([3948b38](https://github.com/sruja-ai/sruja/commit/3948b383f45fa32c9d6e83e1200ab51d609f0264))

## [0.43.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.43.0...sruja-v0.43.1) (2026-05-12)


### Bug Fixes

* **clippy:** resolve collapsible_match and unnecessary_sort_by for Rust 1.95 ([1b97167](https://github.com/sruja-ai/sruja/commit/1b971672326bf623bec54377387ee6cb1e90e77c))

## [0.43.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.42.0...sruja-v0.43.0) (2026-05-12)


### Features

* add reasoned/llm-guided why traversal, consolidate policy evaluation, and implement proposal validation ([8918f96](https://github.com/sruja-ai/sruja/commit/8918f96eb3275bde713aaf247ad1e962ee2daffc))

## [0.42.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.41.1...sruja-v0.42.0) (2026-05-08)


### Features

* **cli:** agent run + enrichment plumbing ([82b7e00](https://github.com/sruja-ai/sruja/commit/82b7e00fbb82cc46f18a4faf1371f3248438f2d2))
* harden sruja and integrate enterprise graph health metrics and book dsl validation ([cdfc72f](https://github.com/sruja-ai/sruja/commit/cdfc72fedcbb21a68c0b62f49f35dbf54872cd08))
* implement outcome-driven evolutionary architectures, fitness evaluation, and health status dashboard ([539f024](https://github.com/sruja-ai/sruja/commit/539f0241d551c004ecfd97ce3cbd53193da06142))


### Bug Fixes

* **clippy:** resolve unnecessary_sort_by warning in context_score.rs ([237ed13](https://github.com/sruja-ai/sruja/commit/237ed13080677337dec4b7cc7f032415f638ceac))
* resolve clippy sort recommendations and C4 nesting and complexity linting for blueprints ([3278b72](https://github.com/sruja-ai/sruja/commit/3278b72ad95b7788ea68fcd5af5e282308107d7f))

## [0.41.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.41.0...sruja-v0.41.1) (2026-05-06)


### Bug Fixes

* resolve E0282 and E0433 compilation errors under wasm32-unknown-unknown in sruja-scan ([c283a1b](https://github.com/sruja-ai/sruja/commit/c283a1bc84a3100f33d30d29f539c1034dc8cc54))

## [0.41.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.40.0...sruja-v0.41.0) (2026-05-06)


### Features

* **cli:** onboard flow, enrichment parity, coverage/docs alignment ([e26957d](https://github.com/sruja-ai/sruja/commit/e26957d532ae55c3bc067c470083d124b7c11377))
* implement incremental scan, community detection, and exporters ([9d6c906](https://github.com/sruja-ai/sruja/commit/9d6c906a204fa88e068b45b26c7d2fc530206fa4))

## [0.40.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.39.0...sruja-v0.40.0) (2026-04-30)


### Features

* add sruja ai coding brief command ([45da2f1](https://github.com/sruja-ai/sruja/commit/45da2f171cf55ad5faf13517864f09e96ebae64b))

## [0.39.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.38.0...sruja-v0.39.0) (2026-04-27)


### Features

* enhance AgenticMemory and manifest discovery robustness ([5d1585c](https://github.com/sruja-ai/sruja/commit/5d1585c0ac6e237e038947ca36230f83ecdc6fb6))
* implement robust file locking for AgenticMemory ([e87cd92](https://github.com/sruja-ai/sruja/commit/e87cd929c14fc9f25dfd4406f71a73108175b512))

## [0.38.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.37.0...sruja-v0.38.0) (2026-04-26)


### Features

* implement Sruja maturation phases (Instrumentation, Context Hints, Infra Discovery) ([8f7c9d4](https://github.com/sruja-ai/sruja/commit/8f7c9d4cf90f5947fc93a6fc8dfbf3e3d383f208))

## [0.37.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.36.0...sruja-v0.37.0) (2026-04-25)


### Features

* implement agentic memory and autonomous optimization loop ([596acb5](https://github.com/sruja-ai/sruja/commit/596acb5609df8294cdb3649f3c446b2b83032d9e))
* implement Evo-inspired agentic patterns (scratchpad, sandboxes, and experimental evaluation) ([70ffc98](https://github.com/sruja-ai/sruja/commit/70ffc98dc8d232611020d72c137e391be1899e58))

## [0.36.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.35.2...sruja-v0.36.0) (2026-04-25)


### Features

* enhance architectural robustness and accuracy invariants ([09abeac](https://github.com/sruja-ai/sruja/commit/09abeac94a6994690aec095ced49c8195349644c))


### Bug Fixes

* prevent DependencyExtractor from generating junk graph nodes ([378c4eb](https://github.com/sruja-ai/sruja/commit/378c4eb04997bafa3817d2248f272b64e65bb2ee))
* remove broad string-matching heuristics that produced junk top-level components ([099db0d](https://github.com/sruja-ai/sruja/commit/099db0d67a90ef262a16f7c2c35116bca9cb51dc))
* resolve all clippy warnings, wire dead code into real paths, fix layer violation detection ([f6902f0](https://github.com/sruja-ai/sruja/commit/f6902f02a33700abd91fe4f6385fbbacb34b7d96))
* resolve for-ai empty output, import json schema mismatch, and why generic answers ([7aafb1e](https://github.com/sruja-ai/sruja/commit/7aafb1e1fe7544ba3f48d80fccafe33dcfaf6250))

## [0.35.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.35.1...sruja-v0.35.2) (2026-04-25)


### Bug Fixes

* break circular dependency sruja-diff-&gt;sruja-intent-&gt;sruja-scan and remove dead code ([414355a](https://github.com/sruja-ai/sruja/commit/414355a86e41ff4485048231f5f4f8204793a184))
* resolve context.json path bug, confidence score always 0, and scan vs DSL ID mismatch ([ed0b1c4](https://github.com/sruja-ai/sruja/commit/ed0b1c43adab9b8f234be55fbd3f8f0124575eb3))

## [0.35.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.35.0...sruja-v0.35.1) (2026-04-24)


### Bug Fixes

* resolve clippy warnings ([f1be73e](https://github.com/sruja-ai/sruja/commit/f1be73e05f4e3276831d6b43f38313e8e51ace2c))
* resolve failing GitHub Actions ([e66230f](https://github.com/sruja-ai/sruja/commit/e66230fe4d1565bdaf71900eceaf3092cdff1d7b))

## [0.35.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.34.0...sruja-v0.35.0) (2026-04-24)


### Features

* Implement Architecture Index MVP with Federated Registry ([133fed8](https://github.com/sruja-ai/sruja/commit/133fed8199dec2329f47a44d5a49b39de75426ca))
* **intent:** implement Phase 4 - Adversarial Critique Engine ('Angry Agent') ([f5ec5fa](https://github.com/sruja-ai/sruja/commit/f5ec5fa6092f44be54915dc4f0ce02df7854ed2e))

## [0.34.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.33.0...sruja-v0.34.0) (2026-04-23)


### Features

* **behavioral-dsl:** implement state machines and api contracts (Phase 3) ([3758a25](https://github.com/sruja-ai/sruja/commit/3758a254d104080c4a2a0e1d90f56ea07580d34b))
* implement Phase 2 architecture-first review workflow ([30f4ebd](https://github.com/sruja-ai/sruja/commit/30f4ebd8e330585b6495b2bd60991130c19ff09f))

## [0.33.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.32.0...sruja-v0.33.0) (2026-04-23)


### Features

* Add context-graph visualization, CI guardrails, and cache resiliency ([ab4261d](https://github.com/sruja-ai/sruja/commit/ab4261db7bd6bb8e811df188133d81608084b637))
* finalize sruja dogfooding with 100/100 context score ([32db9fb](https://github.com/sruja-ai/sruja/commit/32db9fb38a51cc3ae2734e53139a26785eaaa44d))
* refactor node classification to signal-based engine and enhance AI-editor context ([4cc1a67](https://github.com/sruja-ai/sruja/commit/4cc1a6757c37f94b58ab1855c9592e65b9bbede0))


### Bug Fixes

* add missing 'affects' field to Adr in export printer tests ([08dcbb9](https://github.com/sruja-ai/sruja/commit/08dcbb951714fb222abe9c11ed75789abf6b79de))

## [0.32.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.31.0...sruja-v0.32.0) (2026-04-23)


### Features

* **arch:** generalize to Context Graph platform with schema-pluggable kinds ([7c5ded1](https://github.com/sruja-ai/sruja/commit/7c5ded12db3149a8164f06542775cbe71c4a5a98))
* **engine:** implement Phase 2 - Domain Schema and Context Graph transition ([68d2494](https://github.com/sruja-ai/sruja/commit/68d2494ac92e1f7ee409b600eb36d29d27fb5649))
* **intent:** implement phase 3 - dynamic intent check and evidence mapping ([55ee882](https://github.com/sruja-ai/sruja/commit/55ee882407ff63c326ff4a1403ac455703d0a10f))

## [0.31.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.30.1...sruja-v0.31.0) (2026-04-18)


### Features

* **cli:** comprehensive DX overhaul with interactive onboarding, diagnostic dashboards, and interactive watch mode ([13546a1](https://github.com/sruja-ai/sruja/commit/13546a127c95705bd1fa08d689d11e236222815d))

## [0.30.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.30.0...sruja-v0.30.1) (2026-04-10)


### Bug Fixes

* make release-cli Linux build reliable ([e3e50a6](https://github.com/sruja-ai/sruja/commit/e3e50a608554a123f79300eb243c38e7df251184))

## [0.30.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.29.0...sruja-v0.30.0) (2026-04-09)


### Features

* **cli:** refactor infrastructure, enhance status dashboard, and improve health scoring ([b4b3e80](https://github.com/sruja-ai/sruja/commit/b4b3e808a13b38d6838072b20c3f56f7206da44e))

## [0.29.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.28.0...sruja-v0.29.0) (2026-04-08)


### Features

* add --hook and --ci flags to sruja init to automate git hook and GitHub Actions workflow generation ([c0c9368](https://github.com/sruja-ai/sruja/commit/c0c93688bd6a28dae666d61dbc458e5358b420b2))
* add --hook and --ci flags to sruja init to automate git hook and GitHub Actions workflow generation ([4a77d21](https://github.com/sruja-ai/sruja/commit/4a77d21319001cb9f072eef185c3f96065651376))
* add --inject option to sruja export for automated diagram injection in markdown files ([209ad08](https://github.com/sruja-ai/sruja/commit/209ad08d1fc102a2c8ebe4f398fb4613fd30a808))

## [0.28.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.27.1...sruja-v0.28.0) (2026-04-05)


### Features

* tighten architecture context with task focus and ADR intent ([1560c1a](https://github.com/sruja-ai/sruja/commit/1560c1ae510b8f2a192bf5cfa97b381b25a182bd))

## [0.27.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.27.0...sruja-v0.27.1) (2026-04-05)


### Bug Fixes

* resolve clippy warnings to improve code robustness ([52e66d2](https://github.com/sruja-ai/sruja/commit/52e66d20d2b13e6baff92c6d0e6e68a216d40e04))

## [0.27.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.26.0...sruja-v0.27.0) (2026-04-02)


### ⚠ BREAKING CHANGES

* remove runtime/knowledge/sources commands; delete tests; docs cleanup

### cli

* remove runtime/knowledge/sources commands; delete tests; docs cleanup ([bf98c04](https://github.com/sruja-ai/sruja/commit/bf98c048f152ab9f8b6a91a872bdc38ca8c61dba))


### Features

* **scan:** add manifest discovery and scan quality ([7a70de8](https://github.com/sruja-ai/sruja/commit/7a70de886d19d2c7ec97287492754f9b7e4d13d4))

## [0.26.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.25.0...sruja-v0.26.0) (2026-03-27)


### Features

* add D2 export and extension command refactor ([520e8f1](https://github.com/sruja-ai/sruja/commit/520e8f16cce3e4acaea19aab37db1f72cb0f5b5c))

## [0.25.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.24.1...sruja-v0.25.0) (2026-03-25)


### Features

* **vscode-ext:** docs & refs thread panel ([c1b47c2](https://github.com/sruja-ai/sruja/commit/c1b47c253c2871cbce9d5d6e439687360ed9d0d0))

## [0.24.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.24.0...sruja-v0.24.1) (2026-03-22)


### Bug Fixes

* formatting issues ([9e9e61a](https://github.com/sruja-ai/sruja/commit/9e9e61a3f9779b3cfd5cdce90ccae3aa654da453))

## [0.24.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.23.0...sruja-v0.24.0) (2026-03-21)


### Features

* add knowledge field to link elements to code graphs ([7b01747](https://github.com/sruja-ai/sruja/commit/7b01747570b7a7a1527bcd743de9d7cd9f24076f))
* **extension:** sequence diagram preview for scenarios/flows ([d9db0ab](https://github.com/sruja-ai/sruja/commit/d9db0ab7cb2f700038a70064963d88659065235f))


### Bug Fixes

* implement LSP code actions, graph staleness check, and improve error handling ([94ff3d6](https://github.com/sruja-ai/sruja/commit/94ff3d6d4e7658cc7615d126af561fd5a75db992))

## [0.23.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.22.1...sruja-v0.23.0) (2026-03-20)


### Features

* add MCP stdio server and Cursor registration ([7633ea4](https://github.com/sruja-ai/sruja/commit/7633ea40b9adb60e6b661c98dab181b6022ac32d))
* **context:** add repomap command and enhance discover --context ([5027071](https://github.com/sruja-ai/sruja/commit/50270717337176d849cb6bb56ed31ce039bd6cf4))

## [0.22.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.22.0...sruja-v0.22.1) (2026-03-20)


### Bug Fixes

* resolve lint issues in sruja-cli commands ([ef659df](https://github.com/sruja-ai/sruja/commit/ef659df484fe06091b82fe742024e6c2d6d1a273))
* wasm build + check output improvements ([1bd466d](https://github.com/sruja-ai/sruja/commit/1bd466d0f49a4ae6f68d5c5070e5793f6d00aba4))

## [0.22.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.21.0...sruja-v0.22.0) (2026-03-19)


### Features

* **extension:** add focused diagram preview and JSON view export ([7e750da](https://github.com/sruja-ai/sruja/commit/7e750dabc9fec98768a5255398df70ddf375a0ee))
* **validation:** enforce C4 hierarchy with ContainerNestingRule ([d23245c](https://github.com/sruja-ai/sruja/commit/d23245ce0b9bb37cc08bac996f9d9a6ff4a7119d))

## [0.21.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.20.0...sruja-v0.21.0) (2026-03-18)


### Features

* **onboarding:** improve DSL diagnostics, baseline, validation ([2bb94ec](https://github.com/sruja-ai/sruja/commit/2bb94ecf1aed73c23fde48150877a3528685ed1d))


### Bug Fixes

* formatting issues ([d40e05c](https://github.com/sruja-ai/sruja/commit/d40e05cd0e6e405c26b720aab2cd9ae656ba26a8))

## [0.20.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.19.0...sruja-v0.20.0) (2026-03-18)


### Features

* **cli:** add impact analysis command ([6aa9eaa](https://github.com/sruja-ai/sruja/commit/6aa9eaac8f3ef0f59c4293154daf2f96c1f3644a))


### Bug Fixes

* DSL printer roundtrip; add WASM parity tests ([1e20b21](https://github.com/sruja-ai/sruja/commit/1e20b21b271d532d669b0a40085aba22a577e4ca))

## [0.19.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.5...sruja-v0.19.0) (2026-03-18)


### Features

* architecture index sources and policy rules ([a8160a6](https://github.com/sruja-ai/sruja/commit/a8160a65e33ce1bdbcbff6efbbc1fc93e48d95bf))
* implement sources CLI command and JSON export for architecture index ([7fa062d](https://github.com/sruja-ai/sruja/commit/7fa062da4907f873d27ee2dac1f6ce82b02c681d))


### Bug Fixes

* **extension:** render mermaid diagrams correctly in markdown preview ([9f5a04b](https://github.com/sruja-ai/sruja/commit/9f5a04b6be4edcae5eff1cb6c0bfc4411660d9b3))

## [0.18.5](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.4...sruja-v0.18.5) (2026-03-16)


### Bug Fixes

* **extension:** align @types/vscode with engines.vscode ^1.85.0 ([c2a6f6b](https://github.com/sruja-ai/sruja/commit/c2a6f6bb56fbcee2a4c664e2b8b62eaf4c925065))

## [0.18.4](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.3...sruja-v0.18.4) (2026-03-16)


### Bug Fixes

* **extension:** lower VS Code engine to ^1.85.0 for compatibility with 1.105.x ([c28564f](https://github.com/sruja-ai/sruja/commit/c28564f3cc7f3b2550e0f609fcc6785048a8a9bb))

## [0.18.3](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.2...sruja-v0.18.3) (2026-03-15)


### Bug Fixes

* **wasm:** correct element range for go-to-definition in VS Code ([3f5427f](https://github.com/sruja-ai/sruja/commit/3f5427f267617cdf1943b85fa9dafcd74891af47))

## [0.18.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.1...sruja-v0.18.2) (2026-03-15)


### Bug Fixes

* ruby tree sitter ([38f7c9a](https://github.com/sruja-ai/sruja/commit/38f7c9a119a74619720ba13d63c19e6e1c6d7304))
* **security:** remove mocha to resolve diff DoS vulnerability (GHSA-73rr-hh4g-fpgx) ([21eaa77](https://github.com/sruja-ai/sruja/commit/21eaa77230c2adc8c6e720d401ca37393fb4a7db))
* **security:** resolve code scanning alerts ([6c67d40](https://github.com/sruja-ai/sruja/commit/6c67d40ba879da5fe7dac1efe014d3aa98ff7082))
* **security:** update package-lock.json to remove serialize-javascript ([1163620](https://github.com/sruja-ai/sruja/commit/1163620c2f8c61243557e2da5c3ffa3a5da63cec))
* **sruja-scan:** use tree_sitter_ruby::LANGUAGE for 0.23 API ([92ecec6](https://github.com/sruja-ai/sruja/commit/92ecec6f89cc60df72f315ae8b074f648fc7256c))

## [0.18.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.18.0...sruja-v0.18.1) (2026-03-15)


### Bug Fixes

* cast child indices to u32 for tree-sitter 0.26.7 compatibility ([87f628f](https://github.com/sruja-ai/sruja/commit/87f628f2063add6efc2854e7ce1f3d34c290b840))
* upgrade tree-sitter to 0.26.7 with API compatibility ([#69](https://github.com/sruja-ai/sruja/issues/69)) ([6643c3f](https://github.com/sruja-ai/sruja/commit/6643c3fdcf90a5336c03ad198455bea318491a99))

## [0.18.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.17.2...sruja-v0.18.0) (2026-03-15)


### Features

* Add AI editor integration with skills.sh support ([b6f0071](https://github.com/sruja-ai/sruja/commit/b6f0071dd57031a4de5ad0b9de25013ba08ca395))
* add architecture agent skill examples ([a98ce42](https://github.com/sruja-ai/sruja/commit/a98ce42bda511ff65d6a59f8e2b62bab5f160d4b))
* add check, federation, review commands and docs ([5283306](https://github.com/sruja-ai/sruja/commit/52833060f1a1ba47d7b1bc6f5494c04dc015ac07))
* add context export command for AI tools ([557d8e9](https://github.com/sruja-ai/sruja/commit/557d8e95280d886ab5e1f53290a5be00c5a2c61c))
* Add deployment README to website repositories ([814d2e6](https://github.com/sruja-ai/sruja/commit/814d2e6dd9f8f0c6ad23935a12ddc0ff87f15cfa))
* add GitHub Action for architecture drift CI and update README with buyable features ([fd7680d](https://github.com/sruja-ai/sruja/commit/fd7680d29f49bd7afbe4a99a6d890ff750081db6))
* Add GPG signing for git tags in workflows ([d266c0f](https://github.com/sruja-ai/sruja/commit/d266c0f4ffcf4e6b6972e873641fafef7b01f0c1))
* add manual and programmatic triggers for staging deployment ([a76d3b9](https://github.com/sruja-ai/sruja/commit/a76d3b9e32913a9fb906dc5d84a753b070781613))
* Add path-based filtering to workflows for monorepo efficiency ([6504908](https://github.com/sruja-ai/sruja/commit/65049089e0839b50b73dab23dffea8c75dfb6af8))
* add quickstart --generate-baseline flag ([9f42ee7](https://github.com/sruja-ai/sruja/commit/9f42ee7fc20961000b7d330434ced84e9a93c59c))
* Add release candidate workflow for testing before production ([9ca7df5](https://github.com/sruja-ai/sruja/commit/9ca7df567985aa076467bbca2e424876874138e2))
* add Rust coverage to Codecov, simplify compliance reporting, improve ValidatorBuilder ([a167b24](https://github.com/sruja-ai/sruja/commit/a167b2401dc3153bc9015cdfa5fe8759cfe211cb))
* architecture intelligence, new crates, CLI refactor ([bef699f](https://github.com/sruja-ai/sruja/commit/bef699fb3fb0e941b172c4069cfae81aec80cdef))
* **architecture-agent,evaluation,docs:** app-focused discovery loop and CLI tests ([8c7e7d4](https://github.com/sruja-ai/sruja/commit/8c7e7d48f6beb6e19b843c2a6f298f79637c86ac))
* **architecture:** restore sruja WASM, vscode extension and LSP for IDE architecture drafting ([c6b55e3](https://github.com/sruja-ai/sruja/commit/c6b55e391539170ea775eb3772414552d2971350))
* better error messages for lint/validate (DSL roadmap [#1](https://github.com/sruja-ai/sruja/issues/1)) ([931e3e8](https://github.com/sruja-ai/sruja/commit/931e3e8cbb99f3adbab2cc706aaf25f1d91bf70f))
* **book:** add gzip/brotli WASM compression for smaller transfer ([3675aa6](https://github.com/sruja-ai/sruja/commit/3675aa658a789b12312f79081ad19c3376bd490a))
* **book:** add home icon to top nav linking to book index ([a80e123](https://github.com/sruja-ai/sruja/commit/a80e123f8b2701066a946afa1cddf66862aea742))
* **book:** add Sruja 32x32 favicon for docs (sruja.ai) ([5fcaa87](https://github.com/sruja-ai/sruja/commit/5fcaa8706df292faf7a7e21ae3222d0b92e82866))
* **book:** convert quizzes to markdown format and integrate into lessons ([bc4d082](https://github.com/sruja-ai/sruja/commit/bc4d082f34eac7782cf1b484ffa18756ea048833))
* **ci:** add deploy-staging workflow using existing website deploy secrets ([307bc71](https://github.com/sruja-ai/sruja/commit/307bc718373c09cd98570d0321476190e48b09d0))
* **ci:** add Open VSX publishing to staging workflow ([4e86cf2](https://github.com/sruja-ai/sruja/commit/4e86cf279425fd1bd40ab87bbbf7af0f773ffae1))
* **cli:** add architecture completion score to analyze ([8860029](https://github.com/sruja-ai/sruja/commit/8860029aaf910800d02c47a2fa87c020e8f8b47e))
* **cli:** enhance advanced architecture insights and expose timeline ([d10f38b](https://github.com/sruja-ai/sruja/commit/d10f38b7a96361a1e097f181a1ec25425338fd2d))
* **cli:** enhance architecture baseline generation with C4 components, flows, and ADRs ([f132324](https://github.com/sruja-ai/sruja/commit/f132324019a1a26c443898c04beb300f90f9a04f))
* complete buyable features implementation ([b5d6af6](https://github.com/sruja-ai/sruja/commit/b5d6af613f1d6fb22c67495efe314e02dac56ff9))
* Component Knowledge & Documentation System ([bbea0b8](https://github.com/sruja-ai/sruja/commit/bbea0b82d65fca0ec0136628cc784e9450a31af9))
* Deploy designer to staging on main, production on release ([b1b4fad](https://github.com/sruja-ai/sruja/commit/b1b4fad92b616fcd0dd62a60b38b64f3f058e15d))
* **designer:** integrate overview/roles in inspector & fix scrolling ([5a1cc67](https://github.com/sruja-ai/sruja/commit/5a1cc672c63f51b5f0d24bba9fa84fd9953a44a1))
* **designer:** polish header navigation and clean up settings ([98de198](https://github.com/sruja-ai/sruja/commit/98de19824f10ff4c4d4a7d48ac3ab5c60f3cc899))
* diagram improvements, theme fixes, and infrastructure upgrades ([fc8ce84](https://github.com/sruja-ai/sruja/commit/fc8ce84e577039d6d6fa463e549e041b1d0c1252))
* **diagram:** improve layout quality, label positioning, and active refinement loop ([53525ea](https://github.com/sruja-ai/sruja/commit/53525ea3de8b3e355ca6aba3eb052c018d50bab7))
* **docs,skill,eval:** architecture discovery best practices, evaluation, and diff-and-refine ([919685e](https://github.com/sruja-ai/sruja/commit/919685ec8c18fd1f32d19d724fdf7ac067720414))
* **export,discover,skill:** enrich markdown export and selective capture ([fc1964b](https://github.com/sruja-ai/sruja/commit/fc1964b00bea34e30bcfa4e4cc929f299f32dfab))
* **export:** make Markdown export robust and feature-complete ([24c3413](https://github.com/sruja-ai/sruja/commit/24c3413fe649fc07e69affa7892278d9061b153e))
* **export:** markdown/mermaid improvements, view block syntax, extension tests ([5ec53f2](https://github.com/sruja-ai/sruja/commit/5ec53f2a4d1d1414c41374567f75522eeec62d64))
* **extension:** add markdown preview with auto-update and editor title button ([cf1301f](https://github.com/sruja-ai/sruja/commit/cf1301fad2d34c03685dedb86acefa92ae21e8d9))
* **extension:** surface CLI drift/analyze/why in editor ([11e5981](https://github.com/sruja-ai/sruja/commit/11e598178f08a5d4a056cb8231ee118ac63d52fe))
* fix multi-repo architecture examples and add version/dependencies ([dcfbf83](https://github.com/sruja-ai/sruja/commit/dcfbf83bd375827e65947741b5e5cb4c06b0c09b))
* implement container layout improvements with lhead/ltail and depth-based styling ([5eb9829](https://github.com/sruja-ai/sruja/commit/5eb9829b1ed0c68f5178a4fab0c6186b0d3faae5))
* implement context-aware inspector and global layout controls ([8751f69](https://github.com/sruja-ai/sruja/commit/8751f69800c16db508fb38c1ea3f53c73e82c5b0))
* implement manual diagram mode and fix layout issues ([f25b928](https://github.com/sruja-ai/sruja/commit/f25b928d833e0fe7bbd782d2b4e0ca7389fa4cc1))
* implement PR-scoped drift and one-click baseline generation ([a38b3f8](https://github.com/sruja-ai/sruja/commit/a38b3f8e7951eff63ef15d79f98cecb862a5d8cb))
* Implement staging and production deployment workflow ([5f53f16](https://github.com/sruja-ai/sruja/commit/5f53f169f958e3fb7a1ad3415d275490fe774184))
* improve Builder Wizard for beginners ([69207cb](https://github.com/sruja-ai/sruja/commit/69207cbbb5f32106a638699e8122d6c38a081d4e))
* improve diagram quality for complex examples ([cebe003](https://github.com/sruja-ai/sruja/commit/cebe0036bd8c6d49e2d75a14152d01eb70bab33a))
* improve diagram quality for complex examples ([027a2b7](https://github.com/sruja-ai/sruja/commit/027a2b7330c6f3249420c5d68301e1b6be3fbd54))
* integrate Sentry for error tracking and performance monitoring ([8f7eac4](https://github.com/sruja-ai/sruja/commit/8f7eac4c95b8aadac7f8fc77b783df2b23a224d1))
* **layout:** implement LikeC4-inspired structural layout improvements ([24edea5](https://github.com/sruja-ai/sruja/commit/24edea53066433c3e515b57850b21b3fb9e2bf31))
* lint/discover JSON, extraction tests, extension JSON diagnostics, skill docs ([1b28118](https://github.com/sruja-ai/sruja/commit/1b2811854f413895d6092a01d5d26047e9009cb8))
* pre-onboarding-refactor checkpoint ([5bff0a9](https://github.com/sruja-ai/sruja/commit/5bff0a9efa6c89fbcde0a56d278c3c51114434fe))
* **release:** build CLI for GitHub Releases and add install script ([a592ca8](https://github.com/sruja-ai/sruja/commit/a592ca8038a96c6b5306d59d7b72a5e1182e7829))
* **scan:** Cargo and tree-sitter detector with tests ([595fe7e](https://github.com/sruja-ai/sruja/commit/595fe7eddab6f49715dcf6bcda43ac1dbe7b5c22))
* upgrade top nav bar with central command palette trigger and refined styling ([3c1b6e2](https://github.com/sruja-ai/sruja/commit/3c1b6e207ba247470274094cea3c8e3d2d3c2c47))
* WASM size reduction, extension icon, mdBook logo ([3ded042](https://github.com/sruja-ai/sruja/commit/3ded0429c58148ff5c2e1a5a84334c9659964776))
* **website:** improve Try it live section and fix build/workflow issues ([8f42a56](https://github.com/sruja-ai/sruja/commit/8f42a56d7343fb7a17c6c8f4de6ca6af5df34de4))


### Bug Fixes

* add .vscodeignore to exclude parent directories from VSIX ([2bee097](https://github.com/sruja-ai/sruja/commit/2bee097cce7851fbdf60ee85d02d176e2c50e487))
* add apps/vscode-extension to staging deployment path filters ([264649e](https://github.com/sruja-ai/sruja/commit/264649e132a53cf275fad56aa0a20eda109ef2b3))
* add checkout step to deploy job for local action access ([e7dd45b](https://github.com/sruja-ai/sruja/commit/e7dd45b6b21c9061f9879fcc8a9e02deffab75cc))
* add explicit type annotations to fix TypeScript errors in Astro pages ([60ebfd3](https://github.com/sruja-ai/sruja/commit/60ebfd3f1e5612c62b6310ed90a8823b5cb01464))
* add root package files to staging deployment path filters ([ed8cb13](https://github.com/sruja-ai/sruja/commit/ed8cb1353731fd8c6e11ffe68f8f804eede48c4d))
* Add verification step for designer checkout in unified-release ([7535dd7](https://github.com/sruja-ai/sruja/commit/7535dd7cccc52ba5a906d9a2572ed0dd22c5e968))
* Add verification step to designer deployment in unified-release ([b1e1144](https://github.com/sruja-ai/sruja/commit/b1e11446d737cba2a542d1ef6ff08c1cbfe4757a))
* address clippy lints and remove dead code ([fe1588a](https://github.com/sruja-ai/sruja/commit/fe1588ada28a6de647b5d2e6f718eceb1294d811))
* **book:** install mdbook-frontmatter-strip in deploy workflows ([da9927b](https://github.com/sruja-ai/sruja/commit/da9927b91e162d16480e05a94a18dd3d13b3c700))
* build extension before running tests ([2308e23](https://github.com/sruja-ai/sruja/commit/2308e23c2131af6c0da73d20f1746a58ecc61e98))
* Build shared packages before designer app build ([b9b68db](https://github.com/sruja-ai/sruja/commit/b9b68dbdf88b0e713c87b1f6fd18562f05ea9c17))
* CI merge-ready — scoped causal/feedback loops, layer fixes, format & clippy ([647436d](https://github.com/sruja-ai/sruja/commit/647436d8ba66d38bc06799db15a2b38c7bc16f82))
* **ci:** add deploy action to staging workflow path filters ([b0f4996](https://github.com/sruja-ai/sruja/commit/b0f49963ceaa3a4b3f0a370a98169d44048f539f))
* **ci:** add missing WASM build step to designer deployment job ([4bb87c4](https://github.com/sruja-ai/sruja/commit/4bb87c4f83c541c9332d7d48527c31a4c7b0f799))
* **ci:** add WASM build dependency to extension tests ([d1fe602](https://github.com/sruja-ai/sruja/commit/d1fe602c2ceaa5ea8276fcddd7ebbf0e7e406a07))
* **ci:** avoid secrets context in step condition ([1536b06](https://github.com/sruja-ai/sruja/commit/1536b06e37c940172b8143e2e3bd4fe76bab98d6))
* **ci:** build Sruja CLI in drift workflow instead of curl install ([59a6c76](https://github.com/sruja-ai/sruja/commit/59a6c76b3c979336388bea8936a545e349510010))
* **ci:** copy-wasm arg is relative to book/ so use 'book' not 'book/book' ([d6649ee](https://github.com/sruja-ai/sruja/commit/d6649eee592688a73c4e6c53d17edc06448d3b83))
* **ci:** declare reusable-workflow secrets ([9878863](https://github.com/sruja-ai/sruja/commit/9878863c0d9141de5dbfa68a0f69659c4a1b36ff))
* **ci:** don’t block Marketplace publish on Open VSX ([45e8456](https://github.com/sruja-ai/sruja/commit/45e84563a3d1272b4550ac0318a65ac2874e5771))
* **ci:** ensure WASM is found when copying into book output ([6c34cd5](https://github.com/sruja-ai/sruja/commit/6c34cd529cf731543f3dc828187ad76adb493083))
* **ci:** fix Algolia sync workflow and integrate with deployments ([6561e70](https://github.com/sruja-ai/sruja/commit/6561e705e0812040c3ae45fbaaa75a55843407df))
* **ci:** fix scripts, lint errors and frontend tests ([992605b](https://github.com/sruja-ai/sruja/commit/992605b6ae96a15f7630e5921a6d6fe118053520))
* **ci:** full clone and fetch tags in release-please so changelog is scoped to current release ([aa217dc](https://github.com/sruja-ai/sruja/commit/aa217dcad97cb8f3928bea37aff8e8692a87d25f))
* **ci:** ignore relative and absolute internal links in markdown-link-check ([fa8536b](https://github.com/sruja-ai/sruja/commit/fa8536beed4d05a9e58bee98c86b92460b136106))
* **ci:** lower jest coverage thresholds to current baseline ([6bd053e](https://github.com/sruja-ai/sruja/commit/6bd053ededae8f03c712750bfe0664b92594be85))
* **ci:** parse sruja-v* tag for extension version, fail on invalid version ([97bc0dc](https://github.com/sruja-ai/sruja/commit/97bc0dcd27537282bc17fef86a9b633d491cfd22))
* **ci:** release-cli – drop macos-13, normalize version, concurrency, build-failed job ([86d7b99](https://github.com/sruja-ai/sruja/commit/86d7b99f1800d5c55968c3699bf78001e4f23c5d))
* **ci:** release-cli tag for gh-release, pin external actions to commit SHA ([edf44eb](https://github.com/sruja-ai/sruja/commit/edf44eb39e696ac098f9d07b91b304185965d0dd))
* **ci:** resolve feedback_loops_basic and sruja_architecture_v2 lint failures ([10dd320](https://github.com/sruja-ai/sruja/commit/10dd320aed348b585ad4baa71df3f89bc5f5b24d))
* **ci:** skill-lint path and wasm-opt bulk memory ([8006d83](https://github.com/sruja-ai/sruja/commit/8006d833786c1307bce4ecd7be36ebf9da11902c))
* **ci:** skip skill-lint steps when package not in workspace ([d31868d](https://github.com/sruja-ai/sruja/commit/d31868d97127cfd7c8c03fedb68e3c81bb230c1b))
* **ci:** use dtolnay/rust-toolchain in publish-extension; internal-docs next-steps ([a04bc12](https://github.com/sruja-ai/sruja/commit/a04bc12aedae27fd09f2ce6fa7b957618f43377f))
* **ci:** use npm ci from root in docs-quality workflow ([87190f3](https://github.com/sruja-ai/sruja/commit/87190f3ab412f1146f733b1fc519e7a9c5cce39e))
* **ci:** use valid actions/github-script tag ([2e985bc](https://github.com/sruja-ai/sruja/commit/2e985bc7d4717e9b7a3ba74c38042403dd8ba756))
* **cli:** correct syntax rules for flow and external APIs in architecture prompt ([39a89a3](https://github.com/sruja-ai/sruja/commit/39a89a38a05d2d527874a150a6af9cc5306130f5))
* clippy (single_char_add_str, useless_conversion) for CI ([9444841](https://github.com/sruja-ai/sruja/commit/94448412fe098c0acd232283bb4082bae93605cf))
* clippy print_literal in sync_cmd ([a8fe50f](https://github.com/sruja-ai/sruja/commit/a8fe50ff5042a3154cc68534551c27abb7a7e013))
* clippy unnecessary_map_or, redundant_closure, collapsible_if ([f26dd6b](https://github.com/sruja-ai/sruja/commit/f26dd6bfa8472d9c1c2c9f5a9c6cf6d5e013b687))
* code formatting ([56af032](https://github.com/sruja-ai/sruja/commit/56af0325b74e282995650d514f974754cefc36b7))
* Complete GPG signing setup for hotfix tags ([5ff72ac](https://github.com/sruja-ai/sruja/commit/5ff72ac119b8ebb30535ac8c2aeda40a476896f7))
* Complete revert to manual RC tag creation ([62da557](https://github.com/sruja-ai/sruja/commit/62da557662a24056129cea052152a20d4971e9cc))
* configure Codecov permissions and settings for coverage uploads ([859b493](https://github.com/sruja-ai/sruja/commit/859b493a5ec14c27f347989e7b5cbfc5d8ca0302))
* convert RC version format for VS Code Marketplace compatibility ([0aae462](https://github.com/sruja-ai/sruja/commit/0aae46274d7740d195b3e7728d14f30a0b3d4eb2))
* correct extension development path in test runner ([467235b](https://github.com/sruja-ai/sruja/commit/467235b3977e17dbd3973863b6b9f6430e19ddf6))
* Correct GPG action inputs and improve error handling ([b26c049](https://github.com/sruja-ai/sruja/commit/b26c049490ffda96fd5e19f0885fe519fdad6c04))
* Correct GPG action inputs in release-candidate workflow ([ba49571](https://github.com/sruja-ai/sruja/commit/ba495719b8ae9353669588d5c34c28b46dcd3cb7))
* Correct job name in release candidate workflow ([946d6ea](https://github.com/sruja-ai/sruja/commit/946d6ea5324d1240f98e12ce8246c9f03609a783))
* correct output reference in deploy-staging workflow ([609510c](https://github.com/sruja-ai/sruja/commit/609510cf0ff4bb9214398ffe9ec2dd0df1ad0220))
* correct publisher ID from sruja-ai to srujaai ([f6107dc](https://github.com/sruja-ai/sruja/commit/f6107dc3717cd76071a4b4bf2d0eb37641895484))
* correct vsce publish flag from --skipDuplicate to --skip-duplicate ([3807040](https://github.com/sruja-ai/sruja/commit/3807040f336fa8f582245532d7af8360677c3ccc))
* correct WASM output and copy paths in staging deployment ([524c92f](https://github.com/sruja-ai/sruja/commit/524c92f32f32ac9033bf142591d8247e006fdb80))
* **deploy:** accept mdBook output (index.html only), remove Astro check ([ffe75c4](https://github.com/sruja-ai/sruja/commit/ffe75c4b451c176747b73cda297f70a7761738d2))
* **designer:** enable builder panel collapse and add builder tab ([07004f4](https://github.com/sruja-ai/sruja/commit/07004f48818943728371274cbdd164a4b40daef7))
* **designer:** fix editor issues, flickering, reloads and turbo upgrade ([069bbd5](https://github.com/sruja-ai/sruja/commit/069bbd54f8cd081017fe877b54a5a75cd0b4f099))
* **designer:** layout polish - examples dropdown and header cleanup ([f385c54](https://github.com/sruja-ai/sruja/commit/f385c54fb86280be52834b29e7538557ee857606))
* **designer:** remove unused variables and imports to fix build ([7e8db8c](https://github.com/sruja-ai/sruja/commit/7e8db8c741c81a15c5df35f5322219f4bdf32bad))
* **examples:** wire all elements in airflow-architecture to satisfy lint ([6b73f11](https://github.com/sruja-ai/sruja/commit/6b73f114bbdb4d3f672d685120bb1b1fec81f7b8))
* **extension:** open Markdown preview after Export to Markdown ([1748ce3](https://github.com/sruja-ai/sruja/commit/1748ce3985cf7fae05a9160a01cd2be4d872b336))
* **extension:** use publisher SrujaAI so Marketplace listing updates (not new extension) ([2c3d480](https://github.com/sruja-ai/sruja/commit/2c3d480a8e6a9af38d2ec24123645bc979d7f34b))
* **extension:** wasm-pack out-dir relative to crate, use pkg-nodejs ([f8eafed](https://github.com/sruja-ai/sruja/commit/f8eafed93492a744de4a1505269eb0ef163a96b8))
* formatting and skill architecture refactoring ([d470779](https://github.com/sruja-ai/sruja/commit/d4707793ed1897ec83666711039892db64caf177))
* github ci issues ([653ec59](https://github.com/sruja-ai/sruja/commit/653ec599e810884617995cfef05024c3b2e4c327))
* Improve git directory handling in deployment workflows ([d335522](https://github.com/sruja-ai/sruja/commit/d335522c17567e6e391a2dbc1dac6611917a23b4))
* Improve git repository verification in deployment workflows ([ee0f127](https://github.com/sruja-ai/sruja/commit/ee0f12787214696fd5f55a064e81f73a5660c3f4))
* Improve git verification for designer deployment ([88b9a39](https://github.com/sruja-ai/sruja/commit/88b9a39204c242c8755ce7013fcd86c431c9109b))
* Improve git verification in unified-release workflow ([de9bf38](https://github.com/sruja-ai/sruja/commit/de9bf38a1d6f26ef976e634fc8105f350c51e786))
* Improve version detection in release candidate workflow ([bee0c27](https://github.com/sruja-ai/sruja/commit/bee0c279bf9c7bff2593740621d6e7fae69a9cc2))
* improve workflow path filters and GPG action error handling ([f0cd1bd](https://github.com/sruja-ai/sruja/commit/f0cd1bd2192ea823c5bc63b0340cd619a75387c9))
* Install npm dependencies in release candidate workflow ([18a22a9](https://github.com/sruja-ai/sruja/commit/18a22a9b4886c32acdb9690076d5b74614c9648b))
* Only create README for root site deployments ([f7b96e8](https://github.com/sruja-ai/sruja/commit/f7b96e8cb654531a229af5e2163127c0ff14ea58))
* open workspace folder for extension tests ([ef860e8](https://github.com/sruja-ai/sruja/commit/ef860e839db606797509fd4792ac89eba940b4af))
* **parser:** resolve pattern_microservices parse failures ([19c2b3d](https://github.com/sruja-ai/sruja/commit/19c2b3d64803bdbc5a31fa93d6b9e999398ba553))
* **release-cli:** resolve tag in set-version step for workflow_call ([aa929fa](https://github.com/sruja-ai/sruja/commit/aa929fab11f1482282cdb48a600f4d245e8d83fe))
* **release-cli:** use INPUT_VERSION when non-empty (workflow_call sees push) ([8a3c751](https://github.com/sruja-ai/sruja/commit/8a3c751cda4b078ceec5673c8f4c1a753be593d1))
* **release:** auto-trigger CLI build and extension publish from release-please ([00ba9b4](https://github.com/sruja-ai/sruja/commit/00ba9b41b6e603c774a60e3309aedfa2d143b0a8))
* **release:** use workflow_call instead of gh workflow run to avoid 403 ([09368b5](https://github.com/sruja-ai/sruja/commit/09368b5ea8f365e1c9272b0dd56f3fd48164962a))
* remove custom assets config to fix Astro asset 404 errors on staging ([5094663](https://github.com/sruja-ai/sruja/commit/509466372961d128044578056ff52ea92f1be0ce))
* remove duplicate FAQ entry in book SUMMARY.md ([5b2f3bf](https://github.com/sruja-ai/sruja/commit/5b2f3bf4b4f3033de40f5e69337455dc9a08e03d))
* remove files property, use .vscodeignore only ([89e91f7](https://github.com/sruja-ai/sruja/commit/89e91f723294a31e52741c92ecf60a7a8be374f4))
* Remove invalid prerelease inputs from release-please-action ([78dc95b](https://github.com/sruja-ai/sruja/commit/78dc95bedd00acb5659299b89994ade27b1b3eca))
* remove template syntax from action description ([0414df0](https://github.com/sruja-ai/sruja/commit/0414df0bfb42fa8aecdf267ddab590d2b8208777))
* remove unnecessary u32 cast in parser.rs ([76e8e2b](https://github.com/sruja-ai/sruja/commit/76e8e2b859315ef0464c08922b1f345923fb4fec))
* replace console.log with allowed console methods in E2E tests ([a018506](https://github.com/sruja-ai/sruja/commit/a0185065546224f185efb83d4920e5855c0fbd3f))
* resolve duplicate permissions key and ensure authenticated pushes ([911e34e](https://github.com/sruja-ai/sruja/commit/911e34e834bb7b705a3f7b057cd042cb2d24d9e3))
* resolve explicit any lint errors in website and designer utilities ([34fb853](https://github.com/sruja-ai/sruja/commit/34fb8536272be0e5510b601f2225c9f4a8a8706a))
* resolve header visual overlaps and improve responsiveness ([8b00ff2](https://github.com/sruja-ai/sruja/commit/8b00ff2582001519b962c2c92fe4bbd9deac9819))
* resolve lint E205 orphans and E204 cycle in causal loops ([1fe168d](https://github.com/sruja-ai/sruja/commit/1fe168db16497be53aacdea3941e1b1eed45b4f6))
* rust migration cleanup, diagram edges, and build compression ([b8723e4](https://github.com/sruja-ai/sruja/commit/b8723e47be9215bf9864872367d5bf852c1f64be))
* **security:** cargo audit + TruffleHog; docs: remove obsolete MCP/ai refs ([35cc089](https://github.com/sruja-ai/sruja/commit/35cc089256c3469af60c4fd1883132d47431d0ea))
* **shared:** export convertDslToJson as alias for convertDslToModel ([2a146df](https://github.com/sruja-ai/sruja/commit/2a146df860437ff03b781ec75c98075a96f6c374))
* skill cli args ([09fe5ae](https://github.com/sruja-ai/sruja/commit/09fe5aeacfcf76644bfda36d347a34b52fd0ad27))
* skill files formatting ([4b4cdd4](https://github.com/sruja-ai/sruja/commit/4b4cdd4484693722ae817c34933ff378faca0117))
* skills formatting ([778ea2d](https://github.com/sruja-ai/sruja/commit/778ea2d854ff6e0c9271dd5cca426cdc416889ba))
* **skill:** use person only for humans, system for external software ([35cd541](https://github.com/sruja-ai/sruja/commit/35cd5416b3ea70a719dd77312d30fe0fa8942526))
* skip cycle detection for variable-only loops, connect InventoryLoop ([2413fc7](https://github.com/sruja-ai/sruja/commit/2413fc70d938942f84e4f7e200d4f60f26843cd5))
* skip husky in CI to prevent npm ci failures ([97bba8c](https://github.com/sruja-ai/sruja/commit/97bba8c5f7e4ce2ed285daed33e38c8492166e93))
* staging deploy ([2fce43f](https://github.com/sruja-ai/sruja/commit/2fce43f92052d16eb7194455dc1330ab75ce63fa))
* standalone algolia generation script and tsx integration ([7b499c3](https://github.com/sruja-ai/sruja/commit/7b499c320c575fab0da10569c04468b135ac07e1))
* Standardize workflow actions and remove duplicates ([5788d83](https://github.com/sruja-ai/sruja/commit/5788d83661890d86d67262f304ea8013d6d7e89f))
* stories for storybook ([0888158](https://github.com/sruja-ai/sruja/commit/08881588c9e3c5c4cef0338fb6bfc0c030f321a8))
* storybook syntax error and missing import in DetailsView.stories.tsx ([3edc536](https://github.com/sruja-ai/sruja/commit/3edc53640ea1444a2abe17e0b906c6156c42f70b))
* strengthen .vscodeignore to prevent parent directory inclusion ([d39f72c](https://github.com/sruja-ai/sruja/commit/d39f72c63e5cce8d62c6d0b4f5e145e4f6ebb4e9))
* tests ([e5758d0](https://github.com/sruja-ai/sruja/commit/e5758d0c4f9e75be1eb87a4c1b6f55b724cc46de))
* **tests:** correct course content syntax to match language definition ([0863829](https://github.com/sruja-ai/sruja/commit/08638299008d9b8ca5d2f2331939c6252d821075))
* update E2E tests to match current hero content ([87b891e](https://github.com/sruja-ai/sruja/commit/87b891ecf274225eb2e12f94c99a37d740706685))
* update extension ID from sruja-ai.sruja to srujaai.sruja ([fd41bd8](https://github.com/sruja-ai/sruja/commit/fd41bd8ce71215321be1df55b5899d01f870f82e))
* update golangci-lint version to 'latest' for compatibility with action v9 ([86be338](https://github.com/sruja-ai/sruja/commit/86be33872408aa45ee1c8bf381d559ddb9447fcc))
* update libasound2 to libasound2t64 for Ubuntu 24.04 ([27d5df8](https://github.com/sruja-ai/sruja/commit/27d5df8b5a7cfe96f4f94a505c2a7ef9a7c7dc12))
* Update unified-release.yml to use secret for app-id ([bb6e9f7](https://github.com/sruja-ai/sruja/commit/bb6e9f7466a79c6b12869388fffc3836b484b9cc))
* use alpha pre-release identifier for VS Code Marketplace ([d1f702e](https://github.com/sruja-ai/sruja/commit/d1f702e9658224a2ec529525331fd7d5e190fa5d))
* use base semver version for VS Code Marketplace pre-releases ([41ac434](https://github.com/sruja-ai/sruja/commit/41ac4345bfd3876dfa3acd41daeade71733164af))
* use CARGO_PKG_VERSION instead of hardcoded version string ([59aaddc](https://github.com/sruja-ai/sruja/commit/59aaddce0af95ff7aab5e5612f624d20e827271e))
* Use composite action for Go setup in create-go-release job ([05378d8](https://github.com/sruja-ai/sruja/commit/05378d8d8c3f97f678dfd5877b668cb28af4fc63))
* use cp -a in production and staging prepare steps for _astro ([72bcf81](https://github.com/sruja-ai/sruja/commit/72bcf81ce0ba505dfd5cc55b244839ed9f8782bf))
* use cp instead of rsync to ensure _astro directory is copied correctly ([71b9cc5](https://github.com/sruja-ai/sruja/commit/71b9cc55a52a87f75e29029401560e9d5e2d42ab))
* use dynamic extension ID in publish logging ([0d4c4bc](https://github.com/sruja-ai/sruja/commit/0d4c4bc5ba1a069a6ef65ca64cf7d4bcc34dcfa7))
* use existing example file in semantic tokens test ([7d163b4](https://github.com/sruja-ai/sruja/commit/7d163b482a928b82ba45be5fe4c460fda040e6b5))
* Use release-please format for RC tags (v{version}-rc.{number}) ([5e689a8](https://github.com/sruja-ai/sruja/commit/5e689a8b61cb65a12813524aba805f3e8ae0f5c7))
* use release-please suggested version directly for extension ([d1c5300](https://github.com/sruja-ai/sruja/commit/d1c5300238623690044ec23de0cc49fe88fe51d4))
* Use secret instead of variable for GitHub App ID ([5854f47](https://github.com/sruja-ai/sruja/commit/5854f47577980b0e4909c18af7d555e7561775b2))
* **vscode-extension:** fix invalid property access in staging tests ([5bc651b](https://github.com/sruja-ai/sruja/commit/5bc651ba5a47c7bb5e5dafce3cd2c023c508ef54))
* **vscode-extension:** fix test paths and glob version mismatch ([be35b3a](https://github.com/sruja-ai/sruja/commit/be35b3a5b8ea6ace3ac3147f64c5fe4097a36d88))
* **wasm:** add --enable-sign-ext to wasm-opt for LLVM sign-extension ops ([c22db26](https://github.com/sruja-ai/sruja/commit/c22db26dd03c8acfdec0a22dfde0fb83fad6fd8d))
* **website:** improve suppression of glob-loader duplicate ID warnings ([cff0ebf](https://github.com/sruja-ai/sruja/commit/cff0ebf5f04c03c7db7203d8c7455a7a9e2e6aa8))
* **workflows:** use input version first and only use ref when it's a version tag ([a96792f](https://github.com/sruja-ai/sruja/commit/a96792f4aa26485f15f1331932e42b47fd0ead5a))

## [0.17.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.17.1...sruja-v0.17.2) (2026-03-15)


### Bug Fixes

* configure Codecov permissions and settings for coverage uploads ([859b493](https://github.com/sruja-ai/sruja/commit/859b493a5ec14c27f347989e7b5cbfc5d8ca0302))

## [0.17.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.17.0...sruja-v0.17.1) (2026-03-15)


### Bug Fixes

* use CARGO_PKG_VERSION instead of hardcoded version string ([59aaddc](https://github.com/sruja-ai/sruja/commit/59aaddce0af95ff7aab5e5612f624d20e827271e))

## [0.17.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.16.0...sruja-v0.17.0) (2026-03-15)


### Features

* add Rust coverage to Codecov, simplify compliance reporting, improve ValidatorBuilder ([a167b24](https://github.com/sruja-ai/sruja/commit/a167b2401dc3153bc9015cdfa5fe8759cfe211cb))

## [0.16.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.15.1...sruja-v0.16.0) (2026-03-14)


### Features

* **export:** markdown/mermaid improvements, view block syntax, extension tests ([5ec53f2](https://github.com/sruja-ai/sruja/commit/5ec53f2a4d1d1414c41374567f75522eeec62d64))

## [0.15.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.15.0...sruja-v0.15.1) (2026-03-14)


### Bug Fixes

* remove duplicate FAQ entry in book SUMMARY.md ([5b2f3bf](https://github.com/sruja-ai/sruja/commit/5b2f3bf4b4f3033de40f5e69337455dc9a08e03d))
* **skill:** use person only for humans, system for external software ([35cd541](https://github.com/sruja-ai/sruja/commit/35cd5416b3ea70a719dd77312d30fe0fa8942526))

## [0.15.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.14.1...sruja-v0.15.0) (2026-03-14)


### Features

* add check, federation, review commands and docs ([5283306](https://github.com/sruja-ai/sruja/commit/52833060f1a1ba47d7b1bc6f5494c04dc015ac07))


### Bug Fixes

* clippy unnecessary_map_or, redundant_closure, collapsible_if ([f26dd6b](https://github.com/sruja-ai/sruja/commit/f26dd6bfa8472d9c1c2c9f5a9c6cf6d5e013b687))

## [0.14.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.14.0...sruja-v0.14.1) (2026-03-14)


### Bug Fixes

* clippy print_literal in sync_cmd ([a8fe50f](https://github.com/sruja-ai/sruja/commit/a8fe50ff5042a3154cc68534551c27abb7a7e013))

## [0.14.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.13.0...sruja-v0.14.0) (2026-03-14)


### Features

* **cli:** add architecture completion score to analyze ([8860029](https://github.com/sruja-ai/sruja/commit/8860029aaf910800d02c47a2fa87c020e8f8b47e))

## [0.13.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.12.0...sruja-v0.13.0) (2026-03-13)


### Features

* **extension:** surface CLI drift/analyze/why in editor ([11e5981](https://github.com/sruja-ai/sruja/commit/11e598178f08a5d4a056cb8231ee118ac63d52fe))
* lint/discover JSON, extraction tests, extension JSON diagnostics, skill docs ([1b28118](https://github.com/sruja-ai/sruja/commit/1b2811854f413895d6092a01d5d26047e9009cb8))

## [0.12.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.11.0...sruja-v0.12.0) (2026-03-13)


### Features

* **export,discover,skill:** enrich markdown export and selective capture ([fc1964b](https://github.com/sruja-ai/sruja/commit/fc1964b00bea34e30bcfa4e4cc929f299f32dfab))

## [0.11.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.10.2...sruja-v0.11.0) (2026-03-12)


### Features

* **architecture,evaluation,docs:** app-focused discovery loop and CLI tests ([8c7e7d4](https://github.com/sruja-ai/sruja/commit/8c7e7d48f6beb6e19b843c2a6f298f79637c86ac))
* **docs,skill,eval:** architecture discovery best practices, evaluation, and diff-and-refine ([919685e](https://github.com/sruja-ai/sruja/commit/919685ec8c18fd1f32d19d724fdf7ac067720414))

## [0.10.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.10.1...sruja-v0.10.2) (2026-03-11)


### Bug Fixes

* **ci:** build Sruja CLI in drift workflow instead of curl install ([59a6c76](https://github.com/sruja-ai/sruja/commit/59a6c76b3c979336388bea8936a545e349510010))
* **security:** cargo audit + TruffleHog; docs: remove obsolete MCP/ai refs ([35cc089](https://github.com/sruja-ai/sruja/commit/35cc089256c3469af60c4fd1883132d47431d0ea))

## [0.10.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.10.0...sruja-v0.10.1) (2026-03-07)


### Bug Fixes

* **ci:** skip skill-lint steps when package not in workspace ([d31868d](https://github.com/sruja-ai/sruja/commit/d31868d97127cfd7c8c03fedb68e3c81bb230c1b))

## [0.10.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.9.1...sruja-v0.10.0) (2026-03-07)


### Features

* add context export command for AI tools ([557d8e9](https://github.com/sruja-ai/sruja/commit/557d8e95280d886ab5e1f53290a5be00c5a2c61c))
* add GitHub Action for architecture drift CI and update README with buyable features ([fd7680d](https://github.com/sruja-ai/sruja/commit/fd7680d29f49bd7afbe4a99a6d890ff750081db6))
* add quickstart --generate-baseline flag ([9f42ee7](https://github.com/sruja-ai/sruja/commit/9f42ee7fc20961000b7d330434ced84e9a93c59c))
* architecture intelligence, new crates, CLI refactor ([bef699f](https://github.com/sruja-ai/sruja/commit/bef699fb3fb0e941b172c4069cfae81aec80cdef))
* **architecture:** restore sruja WASM, vscode extension and LSP for IDE architecture drafting ([c6b55e3](https://github.com/sruja-ai/sruja/commit/c6b55e391539170ea775eb3772414552d2971350))
* **cli:** enhance advanced architecture insights and expose timeline ([d10f38b](https://github.com/sruja-ai/sruja/commit/d10f38b7a96361a1e097f181a1ec25425338fd2d))
* **cli:** enhance architecture baseline generation with C4 components, flows, and ADRs ([f132324](https://github.com/sruja-ai/sruja/commit/f132324019a1a26c443898c04beb300f90f9a04f))
* complete buyable features implementation ([b5d6af6](https://github.com/sruja-ai/sruja/commit/b5d6af613f1d6fb22c67495efe314e02dac56ff9))
* implement PR-scoped drift and one-click baseline generation ([a38b3f8](https://github.com/sruja-ai/sruja/commit/a38b3f8e7951eff63ef15d79f98cecb862a5d8cb))


### Bug Fixes

* CI merge-ready — scoped causal/feedback loops, layer fixes, format & clippy ([647436d](https://github.com/sruja-ai/sruja/commit/647436d8ba66d38bc06799db15a2b38c7bc16f82))
* **cli:** correct syntax rules for flow and external APIs in architecture prompt ([39a89a3](https://github.com/sruja-ai/sruja/commit/39a89a38a05d2d527874a150a6af9cc5306130f5))

## [0.9.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.9.0...sruja-v0.9.1) (2026-02-17)


### Bug Fixes

* remove unnecessary u32 cast in parser.rs ([76e8e2b](https://github.com/sruja-ai/sruja/commit/76e8e2b859315ef0464c08922b1f345923fb4fec))

## [0.9.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.8.0...sruja-v0.9.0) (2026-02-16)


### Features

* **book:** convert quizzes to markdown format and integrate into lessons ([bc4d082](https://github.com/sruja-ai/sruja/commit/bc4d082f34eac7782cf1b484ffa18756ea048833))

## [0.8.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.11...sruja-v0.8.0) (2026-02-15)


### Features

* add architecture agent skill examples ([a98ce42](https://github.com/sruja-ai/sruja/commit/a98ce42bda511ff65d6a59f8e2b62bab5f160d4b))
* fix multi-repo architecture examples and add version/dependencies ([dcfbf83](https://github.com/sruja-ai/sruja/commit/dcfbf83bd375827e65947741b5e5cb4c06b0c09b))


### Bug Fixes

* code formatting ([56af032](https://github.com/sruja-ai/sruja/commit/56af0325b74e282995650d514f974754cefc36b7))
* skills formatting ([778ea2d](https://github.com/sruja-ai/sruja/commit/778ea2d854ff6e0c9271dd5cca426cdc416889ba))

## [0.7.11](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.10...sruja-v0.7.11) (2026-02-14)


### Bug Fixes

* skill cli args ([09fe5ae](https://github.com/sruja-ai/sruja/commit/09fe5aeacfcf76644bfda36d347a34b52fd0ad27))
* skill files formatting ([4b4cdd4](https://github.com/sruja-ai/sruja/commit/4b4cdd4484693722ae817c34933ff378faca0117))

## [0.7.10](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.9...sruja-v0.7.10) (2026-02-10)


### Bug Fixes

* **book:** install mdbook-frontmatter-strip in deploy workflows ([da9927b](https://github.com/sruja-ai/sruja/commit/da9927b91e162d16480e05a94a18dd3d13b3c700))

## [0.7.9](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.8...sruja-v0.7.9) (2026-02-10)


### Bug Fixes

* **workflows:** use input version first and only use ref when it's a version tag ([a96792f](https://github.com/sruja-ai/sruja/commit/a96792f4aa26485f15f1331932e42b47fd0ead5a))

## [0.7.8](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.7...sruja-v0.7.8) (2026-02-10)


### Bug Fixes

* **examples:** wire all elements in airflow-architecture to satisfy lint ([6b73f11](https://github.com/sruja-ai/sruja/commit/6b73f114bbdb4d3f672d685120bb1b1fec81f7b8))

## [0.7.7](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.6...sruja-v0.7.7) (2026-02-10)


### Bug Fixes

* **release-cli:** use INPUT_VERSION when non-empty (workflow_call sees push) ([8a3c751](https://github.com/sruja-ai/sruja/commit/8a3c751cda4b078ceec5673c8f4c1a753be593d1))

## [0.7.6](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.5...sruja-v0.7.6) (2026-02-10)


### Bug Fixes

* **release-cli:** resolve tag in set-version step for workflow_call ([aa929fa](https://github.com/sruja-ai/sruja/commit/aa929fab11f1482282cdb48a600f4d245e8d83fe))

## [0.7.5](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.4...sruja-v0.7.5) (2026-02-10)


### Bug Fixes

* **ci:** release-cli tag for gh-release, pin external actions to commit SHA ([edf44eb](https://github.com/sruja-ai/sruja/commit/edf44eb39e696ac098f9d07b91b304185965d0dd))

## [0.7.4](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.3...sruja-v0.7.4) (2026-02-10)


### Bug Fixes

* **ci:** release-cli – drop macos-13, normalize version, concurrency, build-failed job ([86d7b99](https://github.com/sruja-ai/sruja/commit/86d7b99f1800d5c55968c3699bf78001e4f23c5d))

## [0.7.3](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.2...sruja-v0.7.3) (2026-02-10)


### Bug Fixes

* **ci:** parse sruja-v* tag for extension version, fail on invalid version ([97bc0dc](https://github.com/sruja-ai/sruja/commit/97bc0dcd27537282bc17fef86a9b633d491cfd22))

## [0.7.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.1...sruja-v0.7.2) (2026-02-10)


### Bug Fixes

* **release:** use workflow_call instead of gh workflow run to avoid 403 ([09368b5](https://github.com/sruja-ai/sruja/commit/09368b5ea8f365e1c9272b0dd56f3fd48164962a))

## [0.7.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.7.0...sruja-v0.7.1) (2026-02-10)


### Bug Fixes

* **release:** auto-trigger CLI build and extension publish from release-please ([00ba9b4](https://github.com/sruja-ai/sruja/commit/00ba9b41b6e603c774a60e3309aedfa2d143b0a8))

## [0.7.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.6.1...sruja-v0.7.0) (2026-02-10)


### Features

* better error messages for lint/validate (DSL roadmap [#1](https://github.com/sruja-ai/sruja/issues/1)) ([931e3e8](https://github.com/sruja-ai/sruja/commit/931e3e8cbb99f3adbab2cc706aaf25f1d91bf70f))
* **release:** build CLI for GitHub Releases and add install script ([a592ca8](https://github.com/sruja-ai/sruja/commit/a592ca8038a96c6b5306d59d7b72a5e1182e7829))


### Bug Fixes

* **extension:** open Markdown preview after Export to Markdown ([1748ce3](https://github.com/sruja-ai/sruja/commit/1748ce3985cf7fae05a9160a01cd2be4d872b336))

## [0.6.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.6.0...sruja-v0.6.1) (2026-02-10)


### Bug Fixes

* **ci:** avoid secrets context in step condition ([1536b06](https://github.com/sruja-ai/sruja/commit/1536b06e37c940172b8143e2e3bd4fe76bab98d6))
* **ci:** declare reusable-workflow secrets ([9878863](https://github.com/sruja-ai/sruja/commit/9878863c0d9141de5dbfa68a0f69659c4a1b36ff))
* **ci:** don’t block Marketplace publish on Open VSX ([45e8456](https://github.com/sruja-ai/sruja/commit/45e84563a3d1272b4550ac0318a65ac2874e5771))

## [0.6.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.5.0...sruja-v0.6.0) (2026-02-10)


### Features

* **book:** add Sruja 32x32 favicon for docs (sruja.ai) ([5fcaa87](https://github.com/sruja-ai/sruja/commit/5fcaa8706df292faf7a7e21ae3222d0b92e82866))

## [0.5.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.4.1...sruja-v0.5.0) (2026-02-10)


### Features

* **book:** add home icon to top nav linking to book index ([a80e123](https://github.com/sruja-ai/sruja/commit/a80e123f8b2701066a946afa1cddf66862aea742))

## [0.4.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.4.0...sruja-v0.4.1) (2026-02-10)


### Bug Fixes

* **wasm:** add --enable-sign-ext to wasm-opt for LLVM sign-extension ops ([c22db26](https://github.com/sruja-ai/sruja/commit/c22db26dd03c8acfdec0a22dfde0fb83fad6fd8d))

## [0.4.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.7...sruja-v0.4.0) (2026-02-10)


### Features

* **book:** add gzip/brotli WASM compression for smaller transfer ([3675aa6](https://github.com/sruja-ai/sruja/commit/3675aa658a789b12312f79081ad19c3376bd490a))
* WASM size reduction, extension icon, mdBook logo ([3ded042](https://github.com/sruja-ai/sruja/commit/3ded0429c58148ff5c2e1a5a84334c9659964776))

## [0.3.7](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.6...sruja-v0.3.7) (2026-02-09)


### Bug Fixes

* **extension:** use publisher SrujaAI so Marketplace listing updates (not new extension) ([2c3d480](https://github.com/sruja-ai/sruja/commit/2c3d480a8e6a9af38d2ec24123645bc979d7f34b))

## [0.3.6](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.5...sruja-v0.3.6) (2026-02-09)


### Bug Fixes

* **ci:** use dtolnay/rust-toolchain in publish-extension; internal-docs next-steps ([a04bc12](https://github.com/sruja-ai/sruja/commit/a04bc12aedae27fd09f2ce6fa7b957618f43377f))

## [0.3.5](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.4...sruja-v0.3.5) (2026-02-09)


### Bug Fixes

* **deploy:** accept mdBook output (index.html only), remove Astro check ([ffe75c4](https://github.com/sruja-ai/sruja/commit/ffe75c4b451c176747b73cda297f70a7761738d2))

## [0.3.4](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.3...sruja-v0.3.4) (2026-02-09)


### Bug Fixes

* **ci:** copy-wasm arg is relative to book/ so use 'book' not 'book/book' ([d6649ee](https://github.com/sruja-ai/sruja/commit/d6649eee592688a73c4e6c53d17edc06448d3b83))

## [0.3.3](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.2...sruja-v0.3.3) (2026-02-09)


### Bug Fixes

* **ci:** skill-lint path and wasm-opt bulk memory ([8006d83](https://github.com/sruja-ai/sruja/commit/8006d833786c1307bce4ecd7be36ebf9da11902c))

## [0.3.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.1...sruja-v0.3.2) (2026-02-09)


### Bug Fixes

* **ci:** ensure WASM is found when copying into book output ([6c34cd5](https://github.com/sruja-ai/sruja/commit/6c34cd529cf731543f3dc828187ad76adb493083))

## [0.3.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.3.0...sruja-v0.3.1) (2026-02-09)


### Bug Fixes

* **ci:** add deploy action to staging workflow path filters ([b0f4996](https://github.com/sruja-ai/sruja/commit/b0f49963ceaa3a4b3f0a370a98169d44048f539f))
* correct WASM output and copy paths in staging deployment ([524c92f](https://github.com/sruja-ai/sruja/commit/524c92f32f32ac9033bf142591d8247e006fdb80))

## [0.3.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.2.4...sruja-v0.3.0) (2026-02-09)


### Features

* **ci:** add deploy-staging workflow using existing website deploy secrets ([307bc71](https://github.com/sruja-ai/sruja/commit/307bc718373c09cd98570d0321476190e48b09d0))

## [0.2.4](https://github.com/sruja-ai/sruja/compare/sruja-v0.2.3...sruja-v0.2.4) (2026-02-09)


### Bug Fixes

* address clippy lints and remove dead code ([fe1588a](https://github.com/sruja-ai/sruja/commit/fe1588ada28a6de647b5d2e6f718eceb1294d811))
* **ci:** use valid actions/github-script tag ([2e985bc](https://github.com/sruja-ai/sruja/commit/2e985bc7d4717e9b7a3ba74c38042403dd8ba756))

## [0.2.3](https://github.com/sruja-ai/sruja/compare/sruja-v0.2.2...sruja-v0.2.3) (2026-02-08)


### Bug Fixes

* **ci:** resolve feedback_loops_basic and sruja_architecture_v2 lint failures ([10dd320](https://github.com/sruja-ai/sruja/commit/10dd320aed348b585ad4baa71df3f89bc5f5b24d))
* **parser:** resolve pattern_microservices parse failures ([19c2b3d](https://github.com/sruja-ai/sruja/commit/19c2b3d64803bdbc5a31fa93d6b9e999398ba553))

## [0.2.2](https://github.com/sruja-ai/sruja/compare/sruja-v0.2.1...sruja-v0.2.2) (2026-02-07)


### Bug Fixes

* skip cycle detection for variable-only loops, connect InventoryLoop ([2413fc7](https://github.com/sruja-ai/sruja/commit/2413fc70d938942f84e4f7e200d4f60f26843cd5))
* tests ([e5758d0](https://github.com/sruja-ai/sruja/commit/e5758d0c4f9e75be1eb87a4c1b6f55b724cc46de))

## [0.2.1](https://github.com/sruja-ai/sruja/compare/sruja-v0.2.0...sruja-v0.2.1) (2026-02-07)


### Bug Fixes

* resolve lint E205 orphans and E204 cycle in causal loops ([1fe168d](https://github.com/sruja-ai/sruja/commit/1fe168db16497be53aacdea3941e1b1eed45b4f6))

## [0.2.0](https://github.com/sruja-ai/sruja/compare/sruja-v0.1.0...sruja-v0.2.0) (2026-02-07)


### Features

* Add AI editor integration with skills.sh support ([b6f0071](https://github.com/sruja-ai/sruja/commit/b6f0071dd57031a4de5ad0b9de25013ba08ca395))
* add builder for playground ([580ea61](https://github.com/sruja-ai/sruja/commit/580ea61b4462e001b856f94df7542074fc88f995))
* Add deployment README to website repositories ([814d2e6](https://github.com/sruja-ai/sruja/commit/814d2e6dd9f8f0c6ad23935a12ddc0ff87f15cfa))
* add docs site ([2030bbd](https://github.com/sruja-ai/sruja/commit/2030bbd57889fbedbfdbf09883f7c6b607b66b84))
* add google tag manager ([7de5795](https://github.com/sruja-ai/sruja/commit/7de5795de47d660daf1dfd35ed3dcbe618775112))
* Add GPG signing for git tags in workflows ([d266c0f](https://github.com/sruja-ai/sruja/commit/d266c0f4ffcf4e6b6972e873641fafef7b01f0c1))
* Add LikeC4Canvas implementation and supporting files ([a8cbff8](https://github.com/sruja-ai/sruja/commit/a8cbff86a44e7f4584745b80ef0e9240400a9e95))
* add manual and programmatic triggers for staging deployment ([a76d3b9](https://github.com/sruja-ai/sruja/commit/a76d3b9e32913a9fb906dc5d84a753b070781613))
* Add path-based filtering to workflows for monorepo efficiency ([6504908](https://github.com/sruja-ai/sruja/commit/65049089e0839b50b73dab23dffea8c75dfb6af8))
* Add release candidate workflow for testing before production ([9ca7df5](https://github.com/sruja-ai/sruja/commit/9ca7df567985aa076467bbca2e424876874138e2))
* **ci:** add Open VSX publishing to staging workflow ([4e86cf2](https://github.com/sruja-ai/sruja/commit/4e86cf279425fd1bd40ab87bbbf7af0f773ffae1))
* Deploy designer to staging on main, production on release ([b1b4fad](https://github.com/sruja-ai/sruja/commit/b1b4fad92b616fcd0dd62a60b38b64f3f058e15d))
* **designer:** integrate overview/roles in inspector & fix scrolling ([5a1cc67](https://github.com/sruja-ai/sruja/commit/5a1cc672c63f51b5f0d24bba9fa84fd9953a44a1))
* **designer:** polish header navigation and clean up settings ([98de198](https://github.com/sruja-ai/sruja/commit/98de19824f10ff4c4d4a7d48ac3ab5c60f3cc899))
* diagram improvements, theme fixes, and infrastructure upgrades ([fc8ce84](https://github.com/sruja-ai/sruja/commit/fc8ce84e577039d6d6fa463e549e041b1d0c1252))
* **diagram:** improve layout quality, label positioning, and active refinement loop ([53525ea](https://github.com/sruja-ai/sruja/commit/53525ea3de8b3e355ca6aba3eb052c018d50bab7))
* implement container layout improvements with lhead/ltail and depth-based styling ([5eb9829](https://github.com/sruja-ai/sruja/commit/5eb9829b1ed0c68f5178a4fab0c6186b0d3faae5))
* implement context-aware inspector and global layout controls ([8751f69](https://github.com/sruja-ai/sruja/commit/8751f69800c16db508fb38c1ea3f53c73e82c5b0))
* implement manual diagram mode and fix layout issues ([f25b928](https://github.com/sruja-ai/sruja/commit/f25b928d833e0fe7bbd782d2b4e0ca7389fa4cc1))
* Implement staging and production deployment workflow ([5f53f16](https://github.com/sruja-ai/sruja/commit/5f53f169f958e3fb7a1ad3415d275490fe774184))
* improve Builder Wizard for beginners ([69207cb](https://github.com/sruja-ai/sruja/commit/69207cbbb5f32106a638699e8122d6c38a081d4e))
* improve diagram quality for complex examples ([cebe003](https://github.com/sruja-ai/sruja/commit/cebe0036bd8c6d49e2d75a14152d01eb70bab33a))
* improve diagram quality for complex examples ([027a2b7](https://github.com/sruja-ai/sruja/commit/027a2b7330c6f3249420c5d68301e1b6be3fbd54))
* integrate Sentry for error tracking and performance monitoring ([8f7eac4](https://github.com/sruja-ai/sruja/commit/8f7eac4c95b8aadac7f8fc77b783df2b23a224d1))
* integrate user stories, requirements, and firebase builder persistence ([0adb537](https://github.com/sruja-ai/sruja/commit/0adb53796f9e10c396ca9e526350893b93a3aae6))
* **layout:** implement LikeC4-inspired structural layout improvements ([24edea5](https://github.com/sruja-ai/sruja/commit/24edea53066433c3e515b57850b21b3fb9e2bf31))
* **playground:** add testing & polish (Phase 3) ([61e985a](https://github.com/sruja-ai/sruja/commit/61e985a1fe800c684c65273951d94848842bf8a5))
* **playground:** improve navigation and relation forms UX ([5d55afe](https://github.com/sruja-ai/sruja/commit/5d55afe68965cdaac94c41e1a9d126d0ed2a66f1))
* **playground:** use shared UI components and inline examples list ([53ca1d0](https://github.com/sruja-ai/sruja/commit/53ca1d0c268aa0a5b3b4c7644c16ea44435db472))
* pre-onboarding-refactor checkpoint ([5bff0a9](https://github.com/sruja-ai/sruja/commit/5bff0a9efa6c89fbcde0a56d278c3c51114434fe))
* scratch work ([e22d057](https://github.com/sruja-ai/sruja/commit/e22d057e0f225ae643f8ff4e0e763f0f7e7f198e))
* sruja language code ([a41cb24](https://github.com/sruja-ai/sruja/commit/a41cb24ac39dd585dcd63974b96045ff7838109b))
* upgrade top nav bar with central command palette trigger and refined styling ([3c1b6e2](https://github.com/sruja-ai/sruja/commit/3c1b6e207ba247470274094cea3c8e3d2d3c2c47))
* **website:** improve Try it live section and fix build/workflow issues ([8f42a56](https://github.com/sruja-ai/sruja/commit/8f42a56d7343fb7a17c6c8f4de6ca6af5df34de4))


### Bug Fixes

* add .vscodeignore to exclude parent directories from VSIX ([2bee097](https://github.com/sruja-ai/sruja/commit/2bee097cce7851fbdf60ee85d02d176e2c50e487))
* add apps/vscode-extension to staging deployment path filters ([264649e](https://github.com/sruja-ai/sruja/commit/264649e132a53cf275fad56aa0a20eda109ef2b3))
* Add checkout and explicit GITHUB_TOKEN to release-please ([eb902a7](https://github.com/sruja-ai/sruja/commit/eb902a776ea71e8f2d6fdb9ba8cc5fec8faf26f2))
* add checkout step to deploy job for local action access ([e7dd45b](https://github.com/sruja-ai/sruja/commit/e7dd45b6b21c9061f9879fcc8a9e02deffab75cc))
* add explicit type annotations to fix TypeScript errors in Astro pages ([60ebfd3](https://github.com/sruja-ai/sruja/commit/60ebfd3f1e5612c62b6310ed90a8823b5cb01464))
* add root package files to staging deployment path filters ([ed8cb13](https://github.com/sruja-ai/sruja/commit/ed8cb1353731fd8c6e11ffe68f8f804eede48c4d))
* add TypeScript path mapping for @sruja/designer ([4a492ce](https://github.com/sruja-ai/sruja/commit/4a492ce13ef8034aadc3bf2b568840f461c36568))
* Add verification step for designer checkout in unified-release ([7535dd7](https://github.com/sruja-ai/sruja/commit/7535dd7cccc52ba5a906d9a2572ed0dd22c5e968))
* Add verification step to designer deployment in unified-release ([b1e1144](https://github.com/sruja-ai/sruja/commit/b1e11446d737cba2a542d1ef6ff08c1cbfe4757a))
* Add version field to golangci-lint configuration ([070aef3](https://github.com/sruja-ai/sruja/commit/070aef327fd73ce18d5c8292a994945f2689e228))
* Add version field with value '1' to golangci.yml ([b442eb8](https://github.com/sruja-ai/sruja/commit/b442eb86a627f3d05222c505b9863d8d863644f5))
* build extension before running tests ([2308e23](https://github.com/sruja-ai/sruja/commit/2308e23c2131af6c0da73d20f1746a58ecc61e98))
* Build shared packages before building website in E2E tests ([c0cac38](https://github.com/sruja-ai/sruja/commit/c0cac38eebe5b8787e3008f134520986c995e404))
* Build shared packages before designer app build ([b9b68db](https://github.com/sruja-ai/sruja/commit/b9b68dbdf88b0e713c87b1f6fd18562f05ea9c17))
* Build shared packages before website build ([6e79f8a](https://github.com/sruja-ai/sruja/commit/6e79f8a34317edb53a68c684bfe30548fa440a3a))
* **ci:** add missing WASM build step to designer deployment job ([4bb87c4](https://github.com/sruja-ai/sruja/commit/4bb87c4f83c541c9332d7d48527c31a4c7b0f799))
* **ci:** add WASM build dependency to extension tests ([d1fe602](https://github.com/sruja-ai/sruja/commit/d1fe602c2ceaa5ea8276fcddd7ebbf0e7e406a07))
* **ci:** fix Algolia sync workflow and integrate with deployments ([6561e70](https://github.com/sruja-ai/sruja/commit/6561e705e0812040c3ae45fbaaa75a55843407df))
* **ci:** fix scripts, lint errors and frontend tests ([992605b](https://github.com/sruja-ai/sruja/commit/992605b6ae96a15f7630e5921a6d6fe118053520))
* **ci:** ignore relative and absolute internal links in markdown-link-check ([fa8536b](https://github.com/sruja-ai/sruja/commit/fa8536beed4d05a9e58bee98c86b92460b136106))
* **ci:** improve codacy coverage reporting ([2e95754](https://github.com/sruja-ai/sruja/commit/2e957540bb05543647bccdbb11b2b494f9d06e11))
* **ci:** use npm ci from root in docs-quality workflow ([87190f3](https://github.com/sruja-ai/sruja/commit/87190f3ab412f1146f733b1fc519e7a9c5cce39e))
* Complete GPG signing setup for hotfix tags ([5ff72ac](https://github.com/sruja-ai/sruja/commit/5ff72ac119b8ebb30535ac8c2aeda40a476896f7))
* Complete revert to manual RC tag creation ([62da557](https://github.com/sruja-ai/sruja/commit/62da557662a24056129cea052152a20d4971e9cc))
* convert RC version format for VS Code Marketplace compatibility ([0aae462](https://github.com/sruja-ai/sruja/commit/0aae46274d7740d195b3e7728d14f30a0b3d4eb2))
* correct extension development path in test runner ([467235b](https://github.com/sruja-ai/sruja/commit/467235b3977e17dbd3973863b6b9f6430e19ddf6))
* Correct GPG action inputs and improve error handling ([b26c049](https://github.com/sruja-ai/sruja/commit/b26c049490ffda96fd5e19f0885fe519fdad6c04))
* Correct GPG action inputs in release-candidate workflow ([ba49571](https://github.com/sruja-ai/sruja/commit/ba495719b8ae9353669588d5c34c28b46dcd3cb7))
* Correct job name in release candidate workflow ([946d6ea](https://github.com/sruja-ai/sruja/commit/946d6ea5324d1240f98e12ce8246c9f03609a783))
* correct output reference in deploy-staging workflow ([609510c](https://github.com/sruja-ai/sruja/commit/609510cf0ff4bb9214398ffe9ec2dd0df1ad0220))
* correct publisher ID from sruja-ai to srujaai ([f6107dc](https://github.com/sruja-ai/sruja/commit/f6107dc3717cd76071a4b4bf2d0eb37641895484))
* correct vsce publish flag from --skipDuplicate to --skip-duplicate ([3807040](https://github.com/sruja-ai/sruja/commit/3807040f336fa8f582245532d7af8360677c3ccc))
* **designer:** enable builder panel collapse and add builder tab ([07004f4](https://github.com/sruja-ai/sruja/commit/07004f48818943728371274cbdd164a4b40daef7))
* **designer:** fix editor issues, flickering, reloads and turbo upgrade ([069bbd5](https://github.com/sruja-ai/sruja/commit/069bbd54f8cd081017fe877b54a5a75cd0b4f099))
* **designer:** layout polish - examples dropdown and header cleanup ([f385c54](https://github.com/sruja-ai/sruja/commit/f385c54fb86280be52834b29e7538557ee857606))
* **designer:** remove unused variables and imports to fix build ([7e8db8c](https://github.com/sruja-ai/sruja/commit/7e8db8c741c81a15c5df35f5322219f4bdf32bad))
* eslint ([28d48e1](https://github.com/sruja-ai/sruja/commit/28d48e193f0ba853b0da028c530552d7f5a44f28))
* Fix diagram tab rendering by using LikeC4View instead of LikeC4Diagram ([21caca1](https://github.com/sruja-ai/sruja/commit/21caca120a87827d52695f5551496a7e67620ee6))
* Fix ESLint errors in TypeScript files ([729811e](https://github.com/sruja-ai/sruja/commit/729811e7be26f9b4dd7cbbe86b2f2c865b2f9046))
* Fix remaining ESLint errors in shared package ([ed633b5](https://github.com/sruja-ai/sruja/commit/ed633b512d9d1623eb8c91bc3ae74ede581184c4))
* github ci issues ([653ec59](https://github.com/sruja-ai/sruja/commit/653ec599e810884617995cfef05024c3b2e4c327))
* github pages ([b5797ca](https://github.com/sruja-ai/sruja/commit/b5797cad38bcaf8835ae01021dcf164089067ca4))
* github pages ([81bcd07](https://github.com/sruja-ai/sruja/commit/81bcd07efe3a986ff46e0d997c5fb6d01ae3feea))
* github pages ([b8a42b8](https://github.com/sruja-ai/sruja/commit/b8a42b83892fdc0bc688612ae13321c165dc5bdc))
* github pages ([9050414](https://github.com/sruja-ai/sruja/commit/9050414e2369112d26112c7d218a95407cb67952))
* go version in ci ([3cd3d71](https://github.com/sruja-ai/sruja/commit/3cd3d713ebccd1e7dffd99ebce940b9740d900fc))
* go version in ci ([e0d3595](https://github.com/sruja-ai/sruja/commit/e0d3595fba0bd20e94ee63c1844b02f06951233d))
* Improve git directory handling in deployment workflows ([d335522](https://github.com/sruja-ai/sruja/commit/d335522c17567e6e391a2dbc1dac6611917a23b4))
* Improve git repository verification in deployment workflows ([ee0f127](https://github.com/sruja-ai/sruja/commit/ee0f12787214696fd5f55a064e81f73a5660c3f4))
* Improve git verification for designer deployment ([88b9a39](https://github.com/sruja-ai/sruja/commit/88b9a39204c242c8755ce7013fcd86c431c9109b))
* Improve git verification in unified-release workflow ([de9bf38](https://github.com/sruja-ai/sruja/commit/de9bf38a1d6f26ef976e634fc8105f350c51e786))
* Improve version detection in release candidate workflow ([bee0c27](https://github.com/sruja-ai/sruja/commit/bee0c279bf9c7bff2593740621d6e7fae69a9cc2))
* improve workflow path filters and GPG action error handling ([f0cd1bd](https://github.com/sruja-ai/sruja/commit/f0cd1bd2192ea823c5bc63b0340cd619a75387c9))
* initialize release-please manifest ([8d5fdba](https://github.com/sruja-ai/sruja/commit/8d5fdbabab9d285f67b2c7dc48e7903edeaf0437))
* Install npm dependencies in release candidate workflow ([18a22a9](https://github.com/sruja-ai/sruja/commit/18a22a9b4886c32acdb9690076d5b74614c9648b))
* LikeC4 diagram rendering and interactivity improvements ([3caf8ac](https://github.com/sruja-ai/sruja/commit/3caf8ace8042c45905ca35c42e42dd725b90c20b))
* Make Go cache and Codacy upload more resilient ([8204175](https://github.com/sruja-ai/sruja/commit/82041753b367dab0cad738e3f7c243c8bf180b6b))
* Make TruffleHog secret scanning continue on error ([8db5738](https://github.com/sruja-ai/sruja/commit/8db573840737d74f4b993b6833aa5e9ef20192cc))
* Only create README for root site deployments ([f7b96e8](https://github.com/sruja-ai/sruja/commit/f7b96e8cb654531a229af5e2163127c0ff14ea58))
* open workspace folder for extension tests ([ef860e8](https://github.com/sruja-ai/sruja/commit/ef860e839db606797509fd4792ac89eba940b4af))
* Remove ./tests from build commands in CI workflow ([1f14904](https://github.com/sruja-ai/sruja/commit/1f14904e8fc5ced9d04edab4453e64c0d1d21d93))
* remove custom assets config to fix Astro asset 404 errors on staging ([5094663](https://github.com/sruja-ai/sruja/commit/509466372961d128044578056ff52ea92f1be0ce))
* Remove duplicate workflow executions ([420f432](https://github.com/sruja-ai/sruja/commit/420f432ea5582bc049b1b3bd9e29c9ddacbb072e))
* remove files property, use .vscodeignore only ([89e91f7](https://github.com/sruja-ai/sruja/commit/89e91f723294a31e52741c92ecf60a7a8be374f4))
* Remove invalid environment reference from deploy job ([c7408e3](https://github.com/sruja-ai/sruja/commit/c7408e3511509a0f7b48479fac0ba6fd1da2f9b9))
* Remove invalid inputs from release-please workflow ([2621e7d](https://github.com/sruja-ai/sruja/commit/2621e7dd5a709e51f3899f9f9fa563e6d5893816))
* Remove invalid prerelease inputs from release-please-action ([78dc95b](https://github.com/sruja-ai/sruja/commit/78dc95bedd00acb5659299b89994ade27b1b3eca))
* remove template syntax from action description ([0414df0](https://github.com/sruja-ai/sruja/commit/0414df0bfb42fa8aecdf267ddab590d2b8208777))
* Remove unnecessary error return from loadFromStdLibFS ([abee6d9](https://github.com/sruja-ai/sruja/commit/abee6d9c52247fb77ed352aa0b601f39e1fea991))
* Remove unreachable code in import.go ([ac69a11](https://github.com/sruja-ai/sruja/commit/ac69a1168f4b143a30cdff2d30a9eb417164c405))
* Remove version field from golangci.yml ([2f7f973](https://github.com/sruja-ai/sruja/commit/2f7f9739cd419375c2df704e1c313cb7b5f84d32))
* Remove version field from golangci.yml to fix schema validation ([ae635a6](https://github.com/sruja-ai/sruja/commit/ae635a6035148f56fe5f6e6cff48971c7da416f2))
* Replace all 'any' types with proper types in posthog.test.ts ([3e49951](https://github.com/sruja-ai/sruja/commit/3e4995165e2020fac33909666833887864a79209))
* replace console.log with allowed console methods in E2E tests ([a018506](https://github.com/sruja-ai/sruja/commit/a0185065546224f185efb83d4920e5855c0fbd3f))
* Resolve all golangci-lint errors ([67c4605](https://github.com/sruja-ai/sruja/commit/67c46052411594ce8110ed02337c9de75a354bbe))
* resolve duplicate permissions key and ensure authenticated pushes ([911e34e](https://github.com/sruja-ai/sruja/commit/911e34e834bb7b705a3f7b057cd042cb2d24d9e3))
* resolve explicit any lint errors in website and designer utilities ([34fb853](https://github.com/sruja-ai/sruja/commit/34fb8536272be0e5510b601f2225c9f4a8a8706a))
* resolve header visual overlaps and improve responsiveness ([8b00ff2](https://github.com/sruja-ai/sruja/commit/8b00ff2582001519b962c2c92fe4bbd9deac9819))
* resolve test regressions in pkg/language ([628ad6f](https://github.com/sruja-ai/sruja/commit/628ad6fc937eb40c67219acb0c5cbd3416112e82))
* resolve website build resolution for @sruja/ui ([6f3ea49](https://github.com/sruja-ai/sruja/commit/6f3ea4923399cbe852d4f378d24aa59feba1c179))
* Restore strPtr function and remove unused fmt import ([c07f74c](https://github.com/sruja-ai/sruja/commit/c07f74c50271af8c8b0cd4c7eb033084f57a054f))
* rust migration cleanup, diagram edges, and build compression ([b8723e4](https://github.com/sruja-ai/sruja/commit/b8723e47be9215bf9864872367d5bf852c1f64be))
* **shared:** export convertDslToJson as alias for convertDslToModel ([2a146df](https://github.com/sruja-ai/sruja/commit/2a146df860437ff03b781ec75c98075a96f6c374))
* skip husky in CI to prevent npm ci failures ([97bba8c](https://github.com/sruja-ai/sruja/commit/97bba8c5f7e4ce2ed285daed33e38c8492166e93))
* sruja site ([5300f5f](https://github.com/sruja-ai/sruja/commit/5300f5f90f68565edccb5d67459389f844246720))
* staging deploy ([2fce43f](https://github.com/sruja-ai/sruja/commit/2fce43f92052d16eb7194455dc1330ab75ce63fa))
* standalone algolia generation script and tsx integration ([7b499c3](https://github.com/sruja-ai/sruja/commit/7b499c320c575fab0da10569c04468b135ac07e1))
* Standardize workflow actions and remove duplicates ([5788d83](https://github.com/sruja-ai/sruja/commit/5788d83661890d86d67262f304ea8013d6d7e89f))
* stories for storybook ([0888158](https://github.com/sruja-ai/sruja/commit/08881588c9e3c5c4cef0338fb6bfc0c030f321a8))
* storybook syntax error and missing import in DetailsView.stories.tsx ([3edc536](https://github.com/sruja-ai/sruja/commit/3edc53640ea1444a2abe17e0b906c6156c42f70b))
* strengthen .vscodeignore to prevent parent directory inclusion ([d39f72c](https://github.com/sruja-ai/sruja/commit/d39f72c63e5cce8d62c6d0b4f5e145e4f6ebb4e9))
* **tests:** correct course content syntax to match language definition ([0863829](https://github.com/sruja-ai/sruja/commit/08638299008d9b8ca5d2f2331939c6252d821075))
* Update build-examples job to use existing example files ([589abdc](https://github.com/sruja-ai/sruja/commit/589abdc1c70e8b3a8be6060ac966c509318c3462))
* Update CI workflow to exclude node_modules from Go commands ([79c57e0](https://github.com/sruja-ai/sruja/commit/79c57e020d5ca45b1ab453267d0dbd5422319e0d))
* update E2E tests to match current hero content ([87b891e](https://github.com/sruja-ai/sruja/commit/87b891ecf274225eb2e12f94c99a37d740706685))
* update extension ID from sruja-ai.sruja to srujaai.sruja ([fd41bd8](https://github.com/sruja-ai/sruja/commit/fd41bd8ce71215321be1df55b5899d01f870f82e))
* update golangci-lint version to 'latest' for compatibility with action v9 ([86be338](https://github.com/sruja-ai/sruja/commit/86be33872408aa45ee1c8bf381d559ddb9447fcc))
* Update golangci.yml to version 2 format with correct schema ([5082ec1](https://github.com/sruja-ai/sruja/commit/5082ec1c4f89f4123a3e9d52e067c1002b378414))
* update libasound2 to libasound2t64 for Ubuntu 24.04 ([27d5df8](https://github.com/sruja-ai/sruja/commit/27d5df8b5a7cfe96f4f94a505c2a7ef9a7c7dc12))
* Update unified-release.yml to use secret for app-id ([bb6e9f7](https://github.com/sruja-ai/sruja/commit/bb6e9f7466a79c6b12869388fffc3836b484b9cc))
* use alpha pre-release identifier for VS Code Marketplace ([d1f702e](https://github.com/sruja-ai/sruja/commit/d1f702e9658224a2ec529525331fd7d5e190fa5d))
* use base semver version for VS Code Marketplace pre-releases ([41ac434](https://github.com/sruja-ai/sruja/commit/41ac4345bfd3876dfa3acd41daeade71733164af))
* Use composite action for Go setup in create-go-release job ([05378d8](https://github.com/sruja-ai/sruja/commit/05378d8d8c3f97f678dfd5877b668cb28af4fc63))
* use cp -a in production and staging prepare steps for _astro ([72bcf81](https://github.com/sruja-ai/sruja/commit/72bcf81ce0ba505dfd5cc55b244839ed9f8782bf))
* use cp instead of rsync to ensure _astro directory is copied correctly ([71b9cc5](https://github.com/sruja-ai/sruja/commit/71b9cc55a52a87f75e29029401560e9d5e2d42ab))
* use dynamic extension ID in publish logging ([0d4c4bc](https://github.com/sruja-ai/sruja/commit/0d4c4bc5ba1a069a6ef65ca64cf7d4bcc34dcfa7))
* use existing example file in semantic tokens test ([7d163b4](https://github.com/sruja-ai/sruja/commit/7d163b482a928b82ba45be5fe4c460fda040e6b5))
* Use explicit Go directories in Lint + Test step ([39ef048](https://github.com/sruja-ai/sruja/commit/39ef048664356a8386c11902ca10a8048c7049e6))
* Use release-please format for RC tags (v{version}-rc.{number}) ([5e689a8](https://github.com/sruja-ai/sruja/commit/5e689a8b61cb65a12813524aba805f3e8ae0f5c7))
* use release-please suggested version directly for extension ([d1c5300](https://github.com/sruja-ai/sruja/commit/d1c5300238623690044ec23de0cc49fe88fe51d4))
* Use root package-lock.json for npm cache in monorepo ([03cb254](https://github.com/sruja-ai/sruja/commit/03cb2544a7574d3209dd71c9d570d2d713720ac4))
* Use secret instead of variable for GitHub App ID ([5854f47](https://github.com/sruja-ai/sruja/commit/5854f47577980b0e4909c18af7d555e7561775b2))
* **vscode-extension:** fix invalid property access in staging tests ([5bc651b](https://github.com/sruja-ai/sruja/commit/5bc651ba5a47c7bb5e5dafce3cd2c023c508ef54))
* **vscode-extension:** fix test paths and glob version mismatch ([be35b3a](https://github.com/sruja-ai/sruja/commit/be35b3a5b8ea6ace3ac3147f64c5fe4097a36d88))
* **website:** improve suppression of glob-loader duplicate ID warnings ([cff0ebf](https://github.com/sruja-ai/sruja/commit/cff0ebf5f04c03c7db7203d8c7455a7a9e2e6aa8))

## [Unreleased]

## [0.1.0] - 2025-01-XX

### Added
- Initial release of Sruja language
- Core DSL: workspace, model, system, container, component, relations
- Requirements and ADRs as first-class language constructs
- Lexer and Parser implementation
- D2 Export support
- Validation engine with 4 core rules:
  - Unique ID validation
  - Valid reference checking
  - Cycle detection
  - Orphan detection
- CLI tools:
  - `sruja export d2` - Export to D2
  - `sruja lint` - Validate code
  - `sruja fmt` - Auto-format code
  - `sruja tree` - Visualize hierarchy
  - `sruja list` - List elements
  - `sruja explain` - Explain elements
- GitHub Actions CI/CD workflows
- Cross-platform release binaries (Linux, macOS, Windows)

### Documentation
- README with quickstart
- Example `.sruja` files

### Removed
- Legacy commands: `compile`, `notebook`, `mcp`, `install`, `update`
- Unused packages: `pkg/compiler`, `pkg/notebook`, `pkg/mcp`, `pkg/kernel`, `pkg/extensions`
- Node.js dependencies and VS Code extension (moved to separate repo)

[Unreleased]: https://github.com/sruja-ai/sruja/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sruja-ai/sruja/releases/tag/v0.1.0
