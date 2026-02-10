# Changelog

All notable changes to Sruja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
