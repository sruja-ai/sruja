# Changelog

All notable changes to Sruja will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/sruja-ai/sruja/compare/sruja-monorepo-v0.1.1...sruja-monorepo-v0.2.0) (2026-01-03)


### Features

* add builder for playground ([c66436f](https://github.com/sruja-ai/sruja/commit/c66436f77518312a6d573bd5e4143a29fb0d7c65))
* Add deployment README to website repositories ([42388af](https://github.com/sruja-ai/sruja/commit/42388af32ea06b594e21e962ca9576c62425620f))
* add docs site ([d6c9e89](https://github.com/sruja-ai/sruja/commit/d6c9e89832ef42649cc547ab7d14efbb849eb4bc))
* add google tag manager ([6d84ea7](https://github.com/sruja-ai/sruja/commit/6d84ea7bfcc1712f3f9842053f76fc64c62a5b7a))
* Add GPG signing for git tags in workflows ([0a12392](https://github.com/sruja-ai/sruja/commit/0a12392a73c90446c6175fe76662ed8d4e3f61c7))
* Add LikeC4Canvas implementation and supporting files ([7310084](https://github.com/sruja-ai/sruja/commit/73100849d304cca1e63eed26b53e4a5d660c0ef4))
* add manual and programmatic triggers for staging deployment ([6b72644](https://github.com/sruja-ai/sruja/commit/6b72644ff1c5ae80a8b1b1fa6aa53984ee7f7994))
* Add path-based filtering to workflows for monorepo efficiency ([c4b1c59](https://github.com/sruja-ai/sruja/commit/c4b1c596ade9dab6cb6c223e200fef351fb25d0b))
* Add release candidate workflow for testing before production ([1186582](https://github.com/sruja-ai/sruja/commit/1186582059baf8c2edb03b54fdb08c46f35190f1))
* **ci:** add Open VSX publishing to staging workflow ([e571ce0](https://github.com/sruja-ai/sruja/commit/e571ce030c8a17b52f43ef4280becbe3b4832bca))
* Deploy designer to staging on main, production on release ([29de643](https://github.com/sruja-ai/sruja/commit/29de6434ab0996deeb6317a22245153a2904e8dc))
* diagram improvements, theme fixes, and infrastructure upgrades ([182583a](https://github.com/sruja-ai/sruja/commit/182583ab360b02f04f325551aa7bec2c7cb7de16))
* **diagram:** improve layout quality, label positioning, and active refinement loop ([f460e96](https://github.com/sruja-ai/sruja/commit/f460e96f95c57cc4dfa9dee669cff2d5b85b5abc))
* implement container layout improvements with lhead/ltail and depth-based styling ([a7f5e56](https://github.com/sruja-ai/sruja/commit/a7f5e565218362d21d5421be080d01d5e1e85394))
* Implement staging and production deployment workflow ([999f88d](https://github.com/sruja-ai/sruja/commit/999f88d02e1e35a933d6a4cea3e74cee85795658))
* improve diagram quality for complex examples ([dd9a36d](https://github.com/sruja-ai/sruja/commit/dd9a36d31ac9d5ac05248ea488ecf3773b714cec))
* improve diagram quality for complex examples ([4a0cb8d](https://github.com/sruja-ai/sruja/commit/4a0cb8d09f88726a56605be248719d9a26362476))
* integrate Sentry for error tracking and performance monitoring ([c71aca4](https://github.com/sruja-ai/sruja/commit/c71aca4fb82f2b79c81f3f39d2bbe021a5cd8217))
* integrate user stories, requirements, and firebase builder persistence ([699ef3d](https://github.com/sruja-ai/sruja/commit/699ef3d8c052cfaf5847d77c2dcaabc866eb44e5))
* **layout:** implement LikeC4-inspired structural layout improvements ([e7e40bc](https://github.com/sruja-ai/sruja/commit/e7e40bc2f6883ee373fcc98b3bce9b79b88868f4))
* **playground:** add testing & polish (Phase 3) ([36d3b14](https://github.com/sruja-ai/sruja/commit/36d3b145f30a39b94d5d57adbdc898428ca31333))
* **playground:** improve navigation and relation forms UX ([4019c2d](https://github.com/sruja-ai/sruja/commit/4019c2d74c49ee47b800f1025e10f82035179fc3))
* **playground:** use shared UI components and inline examples list ([0654c2c](https://github.com/sruja-ai/sruja/commit/0654c2cdb3b1d20b8b57e2d3d1ceb97698160134))
* scratch work ([79c57d3](https://github.com/sruja-ai/sruja/commit/79c57d38e3e5cf1b9bc9a1c2fe99eda8b59697f1))
* sruja language code ([be3a0e7](https://github.com/sruja-ai/sruja/commit/be3a0e756cd24d7415de31decb02f8288b8b343a))


### Bug Fixes

* add .vscodeignore to exclude parent directories from VSIX ([1201bf2](https://github.com/sruja-ai/sruja/commit/1201bf22b3f71f9449bc20322751c78a0f5fc5f5))
* add apps/vscode-extension to staging deployment path filters ([b61e4f7](https://github.com/sruja-ai/sruja/commit/b61e4f72b7473cc13859a1697d1bcdb40759316e))
* Add checkout and explicit GITHUB_TOKEN to release-please ([72d0da0](https://github.com/sruja-ai/sruja/commit/72d0da070d9713ab0be7c8ec8ec4b8bdbc1a5d39))
* add checkout step to deploy job for local action access ([4b519b2](https://github.com/sruja-ai/sruja/commit/4b519b2b576b11e191b8e25d796f4fef2465f3b6))
* add explicit type annotations to fix TypeScript errors in Astro pages ([dfcd1bb](https://github.com/sruja-ai/sruja/commit/dfcd1bb3a4f00b72f551e9701368214ad34be5f1))
* add root package files to staging deployment path filters ([ed1666f](https://github.com/sruja-ai/sruja/commit/ed1666f253fdf30a4bd988a3835004c3345a3b68))
* add TypeScript path mapping for @sruja/designer ([a379d36](https://github.com/sruja-ai/sruja/commit/a379d364f5d9f618e18b0e260cadc0185bfbf7ec))
* Add verification step for designer checkout in unified-release ([47bd662](https://github.com/sruja-ai/sruja/commit/47bd662133a4fa7a837e30031ff169bcf0c207c9))
* Add verification step to designer deployment in unified-release ([d73ab58](https://github.com/sruja-ai/sruja/commit/d73ab58d6347171114904d3a9fc4d8c6e2f77cfe))
* Add version field to golangci-lint configuration ([b1e67e0](https://github.com/sruja-ai/sruja/commit/b1e67e08519b3f2fc2126f5e20f9203e9e8f5c6d))
* Add version field with value '1' to golangci.yml ([d7eec59](https://github.com/sruja-ai/sruja/commit/d7eec596bb5f56ef0151f43701e96b2c447b735e))
* build extension before running tests ([44bd3a7](https://github.com/sruja-ai/sruja/commit/44bd3a718f1c4593445dc65dd09c343d8c5d8ef1))
* Build shared packages before building website in E2E tests ([3185901](https://github.com/sruja-ai/sruja/commit/31859011d0972162ce44711fde74b3858cacbb50))
* Build shared packages before designer app build ([2381453](https://github.com/sruja-ai/sruja/commit/23814535dafe41ae2451bdc422b515af2df253ee))
* Build shared packages before website build ([3b005ad](https://github.com/sruja-ai/sruja/commit/3b005ad81db0a5f87b96f190923d26454584afa1))
* **ci:** fix Algolia sync workflow and integrate with deployments ([c807862](https://github.com/sruja-ai/sruja/commit/c8078620fbed30f9c5371357da0da08099bcb591))
* **ci:** fix scripts, lint errors and frontend tests ([3c9112f](https://github.com/sruja-ai/sruja/commit/3c9112f90b8f1f10a56e08bb85d2e35664057af7))
* **ci:** ignore relative and absolute internal links in markdown-link-check ([3f63028](https://github.com/sruja-ai/sruja/commit/3f63028cdb04add416eeaeae921ab525a9d84dfd))
* **ci:** improve codacy coverage reporting ([28151c2](https://github.com/sruja-ai/sruja/commit/28151c2300c6d6d28e4d8bdb6f868814498a76d3))
* **ci:** use npm ci from root in docs-quality workflow ([889aa1a](https://github.com/sruja-ai/sruja/commit/889aa1af552dfbd770578b9f9eba55f8d7a26b97))
* Complete GPG signing setup for hotfix tags ([0a1d718](https://github.com/sruja-ai/sruja/commit/0a1d71866c2a74f83f6b8daa0f6649d4bf733c02))
* Complete revert to manual RC tag creation ([b273368](https://github.com/sruja-ai/sruja/commit/b2733689dc418cf8d8bc81aa8734449d513030ab))
* convert RC version format for VS Code Marketplace compatibility ([53edd6e](https://github.com/sruja-ai/sruja/commit/53edd6e253d14862d9d9527eb9a520e6392ca26e))
* correct extension development path in test runner ([a7361d2](https://github.com/sruja-ai/sruja/commit/a7361d21d95375074c806181e516e10ea8ef1146))
* Correct GPG action inputs and improve error handling ([80e9602](https://github.com/sruja-ai/sruja/commit/80e96020c85c71f96764e29a3d76e5a1379db734))
* Correct GPG action inputs in release-candidate workflow ([5e6ea35](https://github.com/sruja-ai/sruja/commit/5e6ea3568f7c31380a5a61885215ed2e72a1b7ab))
* Correct job name in release candidate workflow ([693409c](https://github.com/sruja-ai/sruja/commit/693409c803d40e4a9b711ba63e8e1a5ed6b42fe8))
* correct output reference in deploy-staging workflow ([920fd78](https://github.com/sruja-ai/sruja/commit/920fd784e908906f7f9f5af9e98d82ecbf38de88))
* correct publisher ID from sruja-ai to srujaai ([d6666c9](https://github.com/sruja-ai/sruja/commit/d6666c9f515fd43bfee1c27420ed0401d680cbf0))
* correct vsce publish flag from --skipDuplicate to --skip-duplicate ([40f6250](https://github.com/sruja-ai/sruja/commit/40f6250f6d32611528cfda6952ec1d04c0275cdd))
* eslint ([62d228b](https://github.com/sruja-ai/sruja/commit/62d228b03456062e88b103badf2ff4692cd4df89))
* Fix diagram tab rendering by using LikeC4View instead of LikeC4Diagram ([294f30a](https://github.com/sruja-ai/sruja/commit/294f30a19cb3c54a6e937606d1b093c55c463412))
* Fix ESLint errors in TypeScript files ([027dbce](https://github.com/sruja-ai/sruja/commit/027dbce9e4298e5a4b0903bc5ab768adbd092b66))
* Fix remaining ESLint errors in shared package ([0952d66](https://github.com/sruja-ai/sruja/commit/0952d66e338b5f9e5e02445d0b97e87d9af9098a))
* github pages ([4aa6c21](https://github.com/sruja-ai/sruja/commit/4aa6c217e241cb6146b7760e09fdf2138d2e2f94))
* github pages ([58e6095](https://github.com/sruja-ai/sruja/commit/58e6095136bf3f7acfa690e86292f595fec0858e))
* github pages ([a570af1](https://github.com/sruja-ai/sruja/commit/a570af18a2a2a204199944ff6b303cfbedd73e2a))
* github pages ([dbb619d](https://github.com/sruja-ai/sruja/commit/dbb619d4eb0be7e51d86bf3c1692f4c2529be703))
* go version in ci ([84e2407](https://github.com/sruja-ai/sruja/commit/84e2407eaf74d65927e54ca3ef1e7b4571e196f9))
* go version in ci ([ecd5f9b](https://github.com/sruja-ai/sruja/commit/ecd5f9b95d5572086a33fcebe428ac76be768393))
* Improve git directory handling in deployment workflows ([b19bf3b](https://github.com/sruja-ai/sruja/commit/b19bf3b835f5d728989a8a2c79f51e55ec520d2a))
* Improve git repository verification in deployment workflows ([08f0ac4](https://github.com/sruja-ai/sruja/commit/08f0ac458e8792051d9c5d2e5e012f8297dcea64))
* Improve git verification for designer deployment ([e413c50](https://github.com/sruja-ai/sruja/commit/e413c506519193caa3f8ff8f17b9ec52aba47e1b))
* Improve git verification in unified-release workflow ([d995ca5](https://github.com/sruja-ai/sruja/commit/d995ca5e5afff9da3b0165bfb0cd45947cb6c304))
* Improve version detection in release candidate workflow ([7e1daa4](https://github.com/sruja-ai/sruja/commit/7e1daa48f70a4e9cab38cc7ddf03beb57b701911))
* improve workflow path filters and GPG action error handling ([202b4a9](https://github.com/sruja-ai/sruja/commit/202b4a90da49691cce41c1d4911bd4c31d389e72))
* initialize release-please manifest ([2793371](https://github.com/sruja-ai/sruja/commit/2793371de370e3cad21178b3746aaf50987eab61))
* Install npm dependencies in release candidate workflow ([bc7f05a](https://github.com/sruja-ai/sruja/commit/bc7f05ad273ebdf4af45753c67b6c5722c88c778))
* LikeC4 diagram rendering and interactivity improvements ([2af1b8e](https://github.com/sruja-ai/sruja/commit/2af1b8e56e48332529bcea70894e94b2138aa3c7))
* Make Go cache and Codacy upload more resilient ([3fe40f0](https://github.com/sruja-ai/sruja/commit/3fe40f002939655aef495cc03f24948518c65957))
* Make TruffleHog secret scanning continue on error ([cd2db36](https://github.com/sruja-ai/sruja/commit/cd2db366d2828322b092f8598b7a252785b3784c))
* Only create README for root site deployments ([339ba42](https://github.com/sruja-ai/sruja/commit/339ba42c474415549a469510b52389130bcf25fe))
* open workspace folder for extension tests ([1d85eed](https://github.com/sruja-ai/sruja/commit/1d85eedeaea8fe2780d9f3067b97aeb246e847cd))
* Remove ./tests from build commands in CI workflow ([d8ebbae](https://github.com/sruja-ai/sruja/commit/d8ebbaeaf21fbf4e29ad525e53a305cc180e348d))
* remove custom assets config to fix Astro asset 404 errors on staging ([930b64a](https://github.com/sruja-ai/sruja/commit/930b64a1e8bab90cc60ada7e17f086716bbde6e3))
* Remove duplicate workflow executions ([ce809d0](https://github.com/sruja-ai/sruja/commit/ce809d0cfa4607b1b204672329f2d44947489cca))
* remove files property, use .vscodeignore only ([ba46050](https://github.com/sruja-ai/sruja/commit/ba46050ee461e8439d72de76db18a3a75b0647bb))
* Remove invalid environment reference from deploy job ([25ac976](https://github.com/sruja-ai/sruja/commit/25ac976269d23ecedbf2ab855ff5133e99ef068a))
* Remove invalid inputs from release-please workflow ([5a36fda](https://github.com/sruja-ai/sruja/commit/5a36fda035eb3cc01373a294d08afa98ed3c6a8c))
* Remove invalid prerelease inputs from release-please-action ([1395b0b](https://github.com/sruja-ai/sruja/commit/1395b0be9ad48106f007e9f1601cf2ca9dc5330a))
* remove template syntax from action description ([280bcaa](https://github.com/sruja-ai/sruja/commit/280bcaacd70e1b1057c2e9cb4edc63085cc6f2bd))
* Remove unnecessary error return from loadFromStdLibFS ([ca3bd87](https://github.com/sruja-ai/sruja/commit/ca3bd87eb52effe4c5156232c29717628500a921))
* Remove unreachable code in import.go ([f3c96ec](https://github.com/sruja-ai/sruja/commit/f3c96ec03144f69b456e8730396ea8afcf8e8702))
* Remove version field from golangci.yml ([64ab76d](https://github.com/sruja-ai/sruja/commit/64ab76da7b3a42ce8db843cea225f0939fc1ec24))
* Remove version field from golangci.yml to fix schema validation ([73edfdd](https://github.com/sruja-ai/sruja/commit/73edfdd16e80ce2d1e335286a11290103d90b726))
* Replace all 'any' types with proper types in posthog.test.ts ([014a4b6](https://github.com/sruja-ai/sruja/commit/014a4b63202ab878990c8d48b15527bf2da0b61a))
* replace console.log with allowed console methods in E2E tests ([ee284e1](https://github.com/sruja-ai/sruja/commit/ee284e1f155ca52823ff42f65b128eeecd72640a))
* Resolve all golangci-lint errors ([1d63d75](https://github.com/sruja-ai/sruja/commit/1d63d754297094f6794156b206d8d341ab16890c))
* resolve duplicate permissions key and ensure authenticated pushes ([19fc8eb](https://github.com/sruja-ai/sruja/commit/19fc8ebfbca07fe8998bd3dab08bc00dd60af70d))
* resolve explicit any lint errors in website and designer utilities ([45ba712](https://github.com/sruja-ai/sruja/commit/45ba712f54aaf1919c2812738c58cee9d7ad9903))
* resolve test regressions in pkg/language ([6fac380](https://github.com/sruja-ai/sruja/commit/6fac380b1ca5d1953607086b90c539e49e48c57b))
* resolve website build resolution for @sruja/ui ([527834c](https://github.com/sruja-ai/sruja/commit/527834cded5154116c5e9bdd2d5689d1c3b91b29))
* Restore strPtr function and remove unused fmt import ([428806d](https://github.com/sruja-ai/sruja/commit/428806dd2e39ec6c92a7c937c443226d05fd29aa))
* skip husky in CI to prevent npm ci failures ([4a0fad2](https://github.com/sruja-ai/sruja/commit/4a0fad2690de383d0432cc4fbb495e80012d0cd7))
* sruja site ([223bf4e](https://github.com/sruja-ai/sruja/commit/223bf4eb11f98d9e139046092a09e404a50a8f1c))
* staging deploy ([bdf69db](https://github.com/sruja-ai/sruja/commit/bdf69db3787cf9add093c3aa99529ce2225b7c05))
* standalone algolia generation script and tsx integration ([3f95ce3](https://github.com/sruja-ai/sruja/commit/3f95ce3e2564196a5b2f018c246816e3df24c109))
* Standardize workflow actions and remove duplicates ([5dd01cb](https://github.com/sruja-ai/sruja/commit/5dd01cb4fc907747055a859b177ce295901eb057))
* stories for storybook ([ef2282d](https://github.com/sruja-ai/sruja/commit/ef2282d6730e4989511a27c932cc8039a97a1bc7))
* storybook syntax error and missing import in DetailsView.stories.tsx ([1f8a5a1](https://github.com/sruja-ai/sruja/commit/1f8a5a1d098d6a4344c98eaa4b6337b6a80318e2))
* strengthen .vscodeignore to prevent parent directory inclusion ([13b10c1](https://github.com/sruja-ai/sruja/commit/13b10c1a8cd8e7526e116fa22e96808b1822e02a))
* **tests:** correct course content syntax to match language definition ([daf557d](https://github.com/sruja-ai/sruja/commit/daf557ddfb94527bc36d5758642af611f8581bec))
* Update build-examples job to use existing example files ([38d3ee5](https://github.com/sruja-ai/sruja/commit/38d3ee50ec0f66c893f124153c2df16649b78a24))
* Update CI workflow to exclude node_modules from Go commands ([b0d3972](https://github.com/sruja-ai/sruja/commit/b0d3972751903d2aa468ff1c189aea1370fda26b))
* update E2E tests to match current hero content ([72f8c02](https://github.com/sruja-ai/sruja/commit/72f8c02c6ec3ae96efd26a839a2348d77e8ddb0f))
* update extension ID from sruja-ai.sruja to srujaai.sruja ([adfcdca](https://github.com/sruja-ai/sruja/commit/adfcdca4a4e6e970d406bccd8bd95b96da03dfa9))
* update golangci-lint version to 'latest' for compatibility with action v9 ([47df480](https://github.com/sruja-ai/sruja/commit/47df4804bc4e9e9c09de3205f045a7854e939026))
* Update golangci.yml to version 2 format with correct schema ([eab0fb4](https://github.com/sruja-ai/sruja/commit/eab0fb4c3dda32c3ab5d86c8ceb44c8598004d5a))
* update libasound2 to libasound2t64 for Ubuntu 24.04 ([8585245](https://github.com/sruja-ai/sruja/commit/8585245bdd98e5ba9f9115c2bc589e7602cf271e))
* Update unified-release.yml to use secret for app-id ([7647b48](https://github.com/sruja-ai/sruja/commit/7647b48d8b67ad74a38b4684e085d20fde183963))
* use alpha pre-release identifier for VS Code Marketplace ([2c36b1e](https://github.com/sruja-ai/sruja/commit/2c36b1e8f6697b1affddde473589430a82ba51c6))
* use base semver version for VS Code Marketplace pre-releases ([3e268b3](https://github.com/sruja-ai/sruja/commit/3e268b36153d6bb024790b48e816898b8e170fd3))
* Use composite action for Go setup in create-go-release job ([2dcae02](https://github.com/sruja-ai/sruja/commit/2dcae022c894ccccf731cfd8a30bf9270aa2eac7))
* use cp -a in production and staging prepare steps for _astro ([6af90c3](https://github.com/sruja-ai/sruja/commit/6af90c360236328e5b856114e75b1532fffbd1ed))
* use cp instead of rsync to ensure _astro directory is copied correctly ([c8286f2](https://github.com/sruja-ai/sruja/commit/c8286f2410ff2e4a743087d0efea982e4d7296f2))
* use dynamic extension ID in publish logging ([a3f9e68](https://github.com/sruja-ai/sruja/commit/a3f9e68cf441898566c61d3296dcab39ea547a02))
* use existing example file in semantic tokens test ([13cfb2b](https://github.com/sruja-ai/sruja/commit/13cfb2b5ad08bc37d2b282c99efdeddf92fe1e2d))
* Use explicit Go directories in Lint + Test step ([b9f8544](https://github.com/sruja-ai/sruja/commit/b9f8544b9aa91f54a8052f3651b22d5e0f77a8ec))
* Use release-please format for RC tags (v{version}-rc.{number}) ([04a7cdd](https://github.com/sruja-ai/sruja/commit/04a7cdd72bd63df0254bf3fea66c87f1374d36a1))
* use release-please suggested version directly for extension ([d7430f7](https://github.com/sruja-ai/sruja/commit/d7430f77e05a8192c025800d4301e6becb4d4ba1))
* Use root package-lock.json for npm cache in monorepo ([50bb427](https://github.com/sruja-ai/sruja/commit/50bb427f9ef538410c5bd72eb5dd8a664a6a4275))
* Use secret instead of variable for GitHub App ID ([9b0f955](https://github.com/sruja-ai/sruja/commit/9b0f9551dba550035c8fce7db1aff6ffc78c902a))
* **vscode-extension:** fix invalid property access in staging tests ([a1f504e](https://github.com/sruja-ai/sruja/commit/a1f504ef76a95751b565db292c63ea5fd91b95bd))
* **vscode-extension:** fix test paths and glob version mismatch ([bf1395c](https://github.com/sruja-ai/sruja/commit/bf1395cec66c6113e8448a30dd3e0b41bfe857ef))
* **website:** improve suppression of glob-loader duplicate ID warnings ([3018545](https://github.com/sruja-ai/sruja/commit/30185457e92ac24bbbadefa083563b8bc28dd429))

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
