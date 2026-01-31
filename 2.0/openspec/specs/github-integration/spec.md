# GitHub Integration Specification

## Purpose
Integrate Sruja with GitHub's CI/CD platform to automate architecture validation, detect breaking changes in pull requests, and provide architectural feedback directly in the review process. The integration ensures architectural changes are reviewed alongside code changes.

## Requirements

### Requirement: Validate architecture in GitHub Actions

The system SHALL provide a GitHub Actions workflow to validate architecture files on every push and pull request.

#### Scenario: Validate on push to main
- GIVEN a GitHub repository with Sruja integration installed
- AND a push is made to the main branch
- WHEN the GitHub Actions workflow triggers
- THEN the workflow runs the `sruja validate` command
- AND validation results are displayed in the Actions log
- AND the workflow fails if validation errors are detected
- AND the workflow succeeds if validation passes

#### Scenario: Validate on pull request
- GIVEN a pull request is created or updated
- WHEN the GitHub Actions workflow triggers
- THEN the workflow runs validation on the modified architecture files
- AND validation results are displayed in the Actions summary
- AND the PR is marked as failed if validation errors exist
- AND the PR is marked as passing if validation succeeds

#### Scenario: Validate specific file paths
- GIVEN the GitHub Actions workflow is configured with a `paths` filter
- AND only files matching `**/*.sruja` are changed
- WHEN the workflow runs
- THEN validation is performed only on the matching files
- AND other files are ignored

#### Scenario: Workflow with multiple Rust versions
- GIVEN the workflow is configured with a matrix strategy
- AND Rust stable, beta, and nightly versions are specified
- WHEN the workflow runs
- THEN validation is performed on all three Rust versions
- AND all three builds must pass for the workflow to succeed

#### Scenario: Install Sruja in workflow
- GIVEN the GitHub Actions workflow needs to use Sruja
- WHEN the workflow runs
- THEN Sruja is installed via a curl or wget command
- OR Sruja is installed via cargo install
- AND the installation is cached for subsequent runs

#### Scenario: Cache dependencies for faster runs
- GIVEN the workflow is configured with caching
- WHEN the workflow runs
- THEN Cargo dependencies are cached
- AND Sruja's compiled binary is cached if built from source
- AND subsequent workflow runs complete faster

### Requirement: Check for breaking changes in PRs

The system SHALL detect and report breaking changes introduced in pull requests.

#### Scenario: Compare PR against base branch
- GIVEN a pull request is opened from `feature/new-service` to `main`
- AND the PR modifies architecture files
- WHEN the breaking changes check runs
- THEN architecture files in the PR are compared against `main`
- AND breaking changes are detected if critical elements are removed
- AND breaking changes are detected if critical relationships are removed
- AND a summary is displayed in the Actions log

#### Scenario: Fail PR with breaking changes
- GIVEN a pull request introduces breaking changes
- WHEN the breaking changes check runs
- THEN the workflow fails with exit code 20
- AND the PR status check shows as failed
- AND reviewers cannot merge until the issue is addressed

#### Scenario: Allow specific breaking changes
- GIVEN a pull request introduces breaking changes
- AND the workflow is configured with `--allow BREAK-001`
- AND BREAK-001 corresponds to the detected breaking change
- WHEN the breaking changes check runs
- THEN the allowed breaking change is not treated as an error
- AND the workflow succeeds
- AND a warning is displayed for the allowed change

#### Scenario: Multiple breaking changes detected
- GIVEN a pull request removes three critical services
- WHEN the breaking changes check runs
- THEN all three breaking changes are reported
- AND each change is described with its impact
- AND the workflow fails with a comprehensive summary

#### Scenario: No breaking changes detected
- GIVEN a pull request adds new services without removing any
- WHEN the breaking changes check runs
- THEN no breaking changes are found
- AND the workflow succeeds
- AND a success message indicates the architecture is backward-compatible

### Requirement: Generate and comment PR diffs

The system SHALL generate architecture diffs and comment them on pull requests.

#### Scenario: Comment on PR creation
- GIVEN a pull request is opened
- AND architecture files are modified
- WHEN the diff generation step runs
- THEN a diff is computed between the base branch and PR head
- AND the diff is formatted as Markdown
- AND a comment is posted on the PR with the diff
- AND the comment includes a summary of changes

#### Scenario: Comment on PR update
- GIVEN an existing PR has a diff comment
- AND the PR is updated with new architecture changes
- WHEN the diff generation step runs
- THEN a new diff is computed
- AND the existing diff comment is updated with the new content
- OR a new comment is added and the old comment is resolved

#### Scenario: Diff includes element changes
- GIVEN the PR adds a new container "payment-service"
- AND removes container "legacy-payment"
- AND modifies container "api-gateway"
- WHEN the diff is generated
- THEN the diff shows "Added Elements" including "payment-service"
- AND the diff shows "Deleted Elements" including "legacy-payment"
- AND the diff shows "Modified Elements" including "api-gateway"
- AND each change lists the specific modifications

#### Scenario: Diff includes relationship changes
- GIVEN the PR adds a relationship from "web-app" to "new-service"
- AND removes a relationship from "web-app" to "old-service"
- WHEN the diff is generated
- THEN the diff shows "Added Relationships" with details
- AND the diff shows "Removed Relationships" with details
- AND relationship properties (protocol, async) are shown

#### Scenario: Highlight breaking changes in diff comment
- GIVEN the diff contains breaking changes
- WHEN the diff is posted as a PR comment
- THEN breaking changes are prominently displayed
- AND a warning banner indicates breaking changes exist
- AND each breaking change is marked with a warning icon
- AND the comment suggests reviewing the changes carefully

#### Scenario: Use collapsible sections for large diffs
- GIVEN the diff contains more than 20 changes
- WHEN the diff is posted as a PR comment
- THEN the diff uses Markdown collapsible sections (<details>)
- AND each change category (added, deleted, modified) is collapsible
- AND the comment starts in a collapsed state for readability

#### Scenario: Include link to full workflow run
- GIVEN a PR comment is posted with the diff
- WHEN the comment is created
- THEN the comment includes a link to the GitHub Actions workflow run
- AND clicking the link shows the full validation logs
- AND the link helps developers debug validation issues

### Requirement: Install Sruja via package managers

The system SHALL provide installation methods for Sruja that can be used in GitHub Actions workflows.

#### Scenario: Install via Homebrew
- GIVEN a GitHub Actions workflow is running on macOS
- WHEN the workflow installs Sruja via Homebrew
- THEN the command `brew install sruja/tap/sruja` is executed
- AND Sruja is installed to /usr/local/bin or /opt/homebrew/bin
- AND `sruja --version` succeeds

#### Scenario: Install via NPM
- GIVEN a GitHub Actions workflow is running on any platform
- WHEN the workflow installs Sruja via NPM
- THEN the command `npm install -g @sruja/cli` is executed
- AND Sruja is installed to the global npm bin directory
- AND the `sruja` command is available

#### Scenario: Install via cargo
- GIVEN a GitHub Actions workflow has Rust toolchain installed
- WHEN the workflow installs Sruja via cargo
- THEN the command `cargo install sruja-cli` is executed
- AND Sruja is compiled from source
- AND the binary is installed to ~/.cargo/bin
- AND installation completes within acceptable time limits

#### Scenario: Cache Homebrew installation
- GIVEN Homebrew is used to install Sruja
- WHEN the workflow runs
- THEN the Homebrew cache is utilized if available
- AND subsequent runs skip the download step
- AND installation completes faster

#### Scenario: Use pre-built binary for faster installation
- GIVEN a GitHub Actions workflow needs fast startup
- WHEN the workflow installs Sruja
- THEN a pre-built binary is downloaded instead of compiling from source
- AND the binary matches the runner's platform (linux, macos, windows)
- AND the binary is verified with checksums

### Requirement: Support workflow configuration

The system SHALL support configuration via workflow YAML files and repository settings.

#### Scenario: Configure base branch for comparison
- GIVEN the repository's default branch is not `main`
- WHEN the workflow is configured
- THEN the base branch can be set to `develop`, `master`, or any other branch
- AND all comparisons use the configured base branch

#### Scenario: Configure strict validation mode
- GIVEN strict validation is enabled in workflow configuration
- WHEN validation runs
- THEN `--strict` flag is passed to `sruja validate`
- AND warnings are treated as errors
- AND the workflow fails if any warnings exist

#### Scenario: Configure custom architecture directory
- GIVEN architecture files are in a non-standard directory
- WHEN the workflow is configured
- THEN the directory path can be set (e.g., `docs/architecture`)
- AND validation only checks files in the specified directory

#### Scenario: Configure allowed breaking changes
- GIVEN certain breaking changes are acceptable in the project
- WHEN the workflow is configured
- THEN allowed breaking change IDs are listed
- AND these IDs are passed via `--allow` flag
- AND matching changes do not cause workflow failure

#### Scenario: Configure fail-on-warning behavior
- GIVEN the project wants warnings to fail the workflow
- When the workflow is configured with `fail-on-warning: true`
- THEN the workflow fails if validation warnings exist
- AND the exit code reflects the warning as an error

#### Scenario: Configure notification settings
- GIVEN the team wants notifications on architecture changes
- When the workflow is configured
- THEN notifications can be enabled for Slack, Discord, or email
- AND notifications are sent when breaking changes are detected
- AND notifications include links to the PR and diff

### Requirement: Handle workflow errors gracefully

The system SHALL provide clear error messages and suggestions when workflow steps fail.

#### Scenario: Sruja installation fails
- GIVEN the Sruja installation command fails
- WHEN the workflow runs
- THEN the error is logged with details
- AND a helpful error message is displayed in the Actions summary
- AND suggestions for fixing the installation are provided
- AND the workflow fails with a clear status

#### Scenario: Architecture file has syntax errors
- GIVEN validation runs on a file with syntax errors
- WHEN the workflow runs
- THEN validation fails
- AND the errors are displayed in the Actions log
- AND errors include line numbers and descriptions
- AND suggestions for fixing the errors are provided

#### Scenario: Git commands fail in workflow
- GIVEN a git command to compare versions fails
- WHEN the workflow runs
- THEN the error is logged
- AND the workflow fails gracefully
- AND the error message indicates which git command failed
- AND common issues (shallow clone, missing ref) are suggested

#### Scenario: Diff comment creation fails
- GIVEN the diff is generated successfully
- BUT posting the comment to the PR fails
- WHEN the workflow runs
- THEN the error is logged
- AND the workflow may continue or fail based on configuration
- AND the error message indicates the API call failure
- AND the diff is still available in the workflow logs

### Requirement: Support manual workflow dispatch

The system SHALL allow manual triggering of validation and diff workflows.

#### Scenario: Manually trigger validation
- GIVEN the workflow includes `workflow_dispatch` trigger
- AND a user clicks "Run workflow" in the Actions tab
- AND selects a branch and configuration
- WHEN the workflow runs
- THEN validation is performed on the selected branch
- AND results are displayed in the Actions log
- AND a PR comment is not created (manual run)

#### Scenario: Manually trigger with custom configuration
- GIVEN the workflow is manually triggered
- AND the user provides custom base branch and file path inputs
- WHEN the workflow runs
- THEN the custom configuration is used
- AND validation compares against the specified base branch
- AND only the specified files are validated

#### Scenario: Manually trigger diff generation
- GIVEN a workflow exists for generating diffs manually
- AND a user specifies two commits or branches to compare
- WHEN the workflow runs
- THEN a diff is generated between the specified versions
- AND the diff is posted as a new issue or comment
- AND the manual run is clearly marked

### Requirement: Support workflow reuse

The system SHALL provide reusable workflow components for common patterns.

#### Scenario: Use reusable validation workflow
- GIVEN a repository imports the Sruja validation workflow
- WHEN the workflow runs
- THEN the reusable workflow is executed
- AND validation is performed on the repository's files
- AND the workflow can be configured with custom parameters

#### Scenario: Provide workflow templates
- GIVEN a new project wants to integrate Sruja
- WHEN the team applies the Sruja workflow template
- THEN a complete workflow YAML is generated
- AND the template includes best practices
- AND the team can customize the template as needed

#### Scenario: Share workflow configuration across repositories
- GIVEN multiple repositories need the same Sruja configuration
- WHEN the configuration is centralized
- THEN all repositories use the same workflow
- AND updates to the shared workflow apply to all repositories
- AND individual repositories can override specific settings

### Requirement: Support matrix builds

The system SHALL support testing across multiple environments using GitHub Actions matrix strategy.

#### Scenario: Test on multiple OS platforms
- GIVEN a workflow is configured with a matrix
- AND the matrix includes ubuntu-latest, macos-latest, and windows-latest
- WHEN the workflow runs
- THEN validation is performed on all three platforms
- AND each platform must pass for the workflow to succeed
- AND platform-specific differences are handled

#### Scenario: Test on multiple Rust versions
- GIVEN a workflow is configured with a Rust matrix
- AND the matrix includes 1.70.0, stable, beta, and nightly
- WHEN the workflow runs
- THEN validation is performed on all four Rust versions
- AND each version must pass for the workflow to succeed
- AND version-specific issues can be identified

#### Scenario: Fail-fast configuration
- GIVEN a workflow is configured with a matrix
- AND fail-fast is set to false
- AND one matrix job fails
- WHEN the workflow runs
- THEN other matrix jobs continue running
- AND all jobs complete regardless of failures
- AND the workflow fails if any job fails

### Requirement: Generate workflow artifacts

The system SHALL generate downloadable artifacts from workflow runs.

#### Scenario: Export diagrams as artifacts
- GIVEN a workflow includes an export step
- AND exports are generated (Mermaid, SVG, PNG)
- WHEN the workflow completes
- THEN the exported files are uploaded as artifacts
- AND users can download artifacts from the Actions UI
- AND artifacts are retained for the configured retention period

#### Scenario: Export validation reports as artifacts
- GIVEN validation generates a JSON report
- WHEN the workflow completes
- THEN the report is uploaded as an artifact
- AND the report includes detailed validation results
- AND the artifact can be used for debugging

#### Scenario: Archive workflow logs
- GIVEN a workflow completes
- WHEN the workflow finishes
- THEN all workflow logs are archived
- AND logs are available for download
- AND logs include full command output and error messages