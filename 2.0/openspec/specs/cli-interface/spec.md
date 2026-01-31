# CLI Interface Specification

## Purpose
Provide a command-line interface that enables developers to interact with Sruja's core functionality through terminal commands. The CLI should be intuitive, provide helpful feedback, and support both interactive and scripted usage patterns.

## Requirements

### Requirement: Parse global flags

The system SHALL support global flags that apply to all CLI commands.

#### Scenario: Parse verbose flag
- GIVEN the CLI is invoked with the `--verbose` flag
- WHEN the command executes
- THEN verbose logging is enabled
- AND detailed debug information is output to the console

#### Scenario: Parse short verbose flag
- GIVEN the CLI is invoked with the `-v` flag
- WHEN the command executes
- THEN verbose logging is enabled
- AND the behavior is identical to `--verbose`

#### Scenario: Parse quiet flag
- GIVEN the CLI is invoked with the `--quiet` flag
- WHEN the command executes
- THEN all non-error output is suppressed
- AND only error messages are displayed

#### Scenario: Parse config flag
- GIVEN the CLI is invoked with `--config /path/to/config.yaml`
- WHEN the command executes
- THEN the specified configuration file is loaded
- AND settings override defaults

#### Scenario: Parse no-color flag
- GIVEN the CLI is invoked with `--no-color`
- WHEN the command executes
- THEN all output is in plain text without ANSI color codes
- AND the output is suitable for piping to files

### Requirement: Initialize new Sruja project

The system SHALL provide an `init` command to set up a new Sruja project.

#### Scenario: Initialize in current directory
- GIVEN the current directory is empty or contains no conflicting files
- WHEN `sruja init` is executed
- THEN a `.sruja` directory is created
- AND a default `architecture.sruja` file is created
- AND a `config.yaml` file is created with default settings
- AND a success message indicates the project was initialized

#### Scenario: Initialize in specified directory
- GIVEN a directory path `./my-project` is specified
- AND the directory does not exist
- WHEN `sruja init ./my-project` is executed
- THEN the directory `./my-project` is created
- AND Sruja project structure is initialized in the new directory

#### Scenario: Initialize with template
- GIVEN the `--template simple` flag is provided
- WHEN `sruja init --template simple` is executed
- THEN the architecture file is created using the simple template
- AND the template includes example elements and relationships

#### Scenario: Fail initialization in non-empty directory
- GIVEN the target directory already contains files
- AND the `--force` flag is not provided
- WHEN `sruja init` is executed
- THEN initialization fails with an error
- AND the error message indicates the directory is not empty
- AND a suggestion to use `--force` is provided

#### Scenario: Force initialization in non-empty directory
- GIVEN the target directory already contains files
- AND the `--force` flag is provided
- WHEN `sruja init --force` is executed
- THEN Sruja files are created
- AND existing files are not modified
- AND a warning is displayed about the non-empty directory

### Requirement: Validate architecture files

The system SHALL provide a `validate` command to check architecture files for errors.

#### Scenario: Validate default architecture file
- GIVEN a valid `architecture.sruja` file exists in `.sruja/`
- WHEN `sruja validate` is executed
- THEN the file is parsed and validated
- AND a success message indicates validation passed
- AND statistics are shown (element count, relationship count)

#### Scenario: Validate specified file
- GIVEN an architecture file `custom-arch.sruja` exists
- WHEN `sruja validate custom-arch.sruja` is executed
- THEN the specified file is validated
- AND results are displayed

#### Scenario: Validate multiple files
- GIVEN multiple architecture files exist
- WHEN `sruja validate arch1.sruja arch2.sruja arch3.sruja` is executed
- THEN all specified files are validated
- AND results are shown for each file
- AND an overall summary indicates pass/fail status

#### Scenario: Detect validation errors
- GIVEN an architecture file contains invalid syntax
- WHEN `sruja validate` is executed
- THEN validation fails
- AND error messages are displayed
- AND each error includes line number and description
- AND suggestions for fixing the errors are provided

#### Scenario: Strict validation mode
- GIVEN an architecture file has warnings but no errors
- WHEN `sruja validate --strict` is executed
- THEN warnings are treated as errors
- AND validation fails
- AND all warnings are displayed as errors

#### Scenario: Output validation as JSON
- GIVEN validation is requested with `--format json`
- WHEN `sruja validate --format json` is executed
- THEN results are output as JSON
- AND the JSON includes a "valid" boolean field
- AND errors and warnings are in JSON arrays
- AND the output is parseable by JSON tools

### Requirement: Compare architecture versions

The system SHALL provide a `diff` command to compare two architecture versions.

#### Scenario: Compare two files
- GIVEN two architecture files `old.sruja` and `new.sruja` exist
- WHEN `sruja diff old.sruja new.sruja` is executed
- THEN a diff is computed and displayed
- AND added, deleted, and modified elements are shown
- AND added and removed relationships are shown

#### Scenario: Compare against git branch
- GIVEN a git repository with architecture files
- WHEN `sruja diff main -- .sruja/architecture.sruja` is executed
- THEN the current file is compared against the main branch
- AND only the specified file is compared
- AND git is used to retrieve the old version

#### Scenario: Generate diff as Markdown
- GIVEN a diff is requested with `--format markdown`
- WHEN `sruja diff old.sruja new.sruja --format markdown` is executed
- THEN the diff is formatted as Markdown
- AND proper Markdown headings and lists are used
- AND the output is suitable for documentation

#### Scenario: Output diff to file
- GIVEN `--output CHANGES.md` is specified
- WHEN `sruja diff old.sruja new.sruja --output CHANGES.md` is executed
- THEN the diff is written to `CHANGES.md`
- AND no output is printed to the console
- AND a success message indicates the file was created

#### Scenario: Highlight breaking changes only
- GIVEN `--breaking` flag is specified
- WHEN `sruja diff old.sruja new.sruja --breaking` is executed
- THEN only breaking changes are displayed
- AND non-breaking changes are filtered out
- AND the output is concise

### Requirement: Export architecture to various formats

The system SHALL provide an `export` command to generate diagrams and documentation.

#### Scenario: Export to Mermaid
- GIVEN an architecture file exists
- WHEN `sruja export mermaid` is executed
- THEN valid Mermaid diagram syntax is output
- AND all elements and relationships are included
- AND the output can be rendered by Mermaid tools

#### Scenario: Export with specific view
- GIVEN `--view containers` is specified
- WHEN `sruja export mermaid --view containers` is executed
- THEN only Container-level elements are included
- AND System and Person elements are excluded
- AND the hierarchy is preserved

#### Scenario: Export to SVG
- GIVEN `sruja export svg` is executed
- AND Mermaid CLI is available
- WHEN the export command runs
- THEN an SVG file is generated
- AND the file contains vector graphics of the diagram

#### Scenario: Export with theme
- GIVEN `--theme dark` is specified
- WHEN `sruja export mermaid --theme dark` is executed
- THEN the diagram uses dark color scheme
- AND styling matches the dark theme

#### Scenario: Export to Markdown
- GIVEN `sruja export markdown` is executed
- WHEN the export command runs
- THEN a Markdown document is generated
- AND the document includes architecture overview
- AND all elements and relationships are documented

#### Scenario: Export to JSON
- GIVEN `sruja export json` is executed
- WHEN the export command runs
- THEN a JSON representation of the architecture is output
- AND all fields are serialized properly

#### Scenario: Export to PNG
- GIVEN `sruja export png` is executed
- AND diagram generation tools are available
- WHEN the export command runs
- THEN a PNG raster image is generated
- AND the image shows the architecture diagram

### Requirement: Check for breaking changes

The system SHALL provide a `check` command to validate changes don't introduce breaking changes.

#### Scenario: Check staged changes
- GIVEN a git repository has staged changes to architecture files
- WHEN `sruja check` is executed
- THEN staged files are compared against the base branch
- AND breaking changes are detected
- AND a report indicates if breaking changes exist

#### Scenario: Check against specific branch
- GIVEN `--base develop` is specified
- WHEN `sruja check --base develop` is executed
- THEN changes are compared against the develop branch
- AND breaking changes relative to develop are reported

#### Scenario: Check specific files
- GIVEN specific files are provided as arguments
- WHEN `sruja check architecture.sruja` is executed
- THEN only the specified files are checked
- AND other architecture files are ignored

#### Scenario: Allow specific breaking changes
- GIVEN `--allow BREAK-001 BREAK-002` is specified
- AND breaking changes BREAK-001 and BREAK-002 are detected
- WHEN `sruja check --allow BREAK-001 BREAK-002` is executed
- THEN the allowed breaking changes are not considered errors
- AND only other breaking changes are reported

#### Scenario: Fail on warnings
- GIVEN `--fail-on-warning` is specified
- AND warnings are detected but no breaking changes
- WHEN `sruja check --fail-on-warning` is executed
- THEN the check fails
- AND warnings are treated as errors

### Requirement: Trace element dependencies

The system SHALL provide a `trace` command to analyze dependencies.

#### Scenario: Trace upstream dependencies
- GIVEN an element ID "database" is provided
- WHEN `sruja trace database` is executed
- THEN all elements depending on "database" are listed
- AND upstream dependencies are shown
- AND the dependency depth is indicated

#### Scenario: Trace downstream dependencies
- GIVEN `--direction downstream` is specified
- WHEN `sruja trace api-gateway --direction downstream` is executed
- THEN all elements "api-gateway" depends on are listed
- AND downstream dependencies are shown

#### Scenario: Trace with depth limit
- GIVEN `--depth 3` is specified
- WHEN `sruja trace service-a --depth 3` is executed
- THEN dependencies are traced to a maximum depth of 3
- AND deeper dependencies are excluded

#### Scenario: Trace both directions
- GIVEN `--direction both` is specified
- WHEN `sruja trace service-a --direction both` is executed
- THEN both upstream and downstream dependencies are listed
- AND each direction is clearly labeled

#### Scenario: Generate trace as dot format
- GIVEN `--format dot` is specified
- WHEN `sruja trace service-a --format dot` is executed
- THEN Graphviz DOT format is output
- AND the dependency graph is represented in DOT syntax

### Requirement: Generate documentation

The system SHALL provide a `docs` command to generate project documentation.

#### Scenario: Generate HTML documentation
- GIVEN `sruja docs` is executed
- WHEN the docs command runs
- THEN HTML documentation is generated in the default directory
- AND the documentation includes architecture overview
- AND all elements and relationships are documented

#### Scenario: Generate Markdown documentation
- GIVEN `--format markdown` is specified
- WHEN `sruja docs --format markdown` is executed
- THEN Markdown documentation is generated
- AND the output is suitable for static site generators

#### Scenario: Specify output directory
- GIVEN `--directory ./output/docs` is specified
- WHEN `sruja docs --directory ./output/docs` is executed
- THEN documentation is written to the specified directory
- AND the directory is created if it doesn't exist

#### Scenario: Serve documentation locally
- GIVEN `--serve` flag is specified
- WHEN `sruja docs --serve` is executed
- THEN a local web server starts
- AND documentation is accessible at localhost:8080
- AND the server continues running until interrupted

#### Scenario: Serve on custom port
- GIVEN `--serve --port 3000` is specified
- WHEN `sruja docs --serve --port 3000` is executed
- THEN the web server starts on port 3000
- AND documentation is accessible at localhost:3000

### Requirement: Display version information

The system SHALL provide a `version` command to show version details.

#### Scenario: Show full version
- GIVEN `sruja version` is executed
- WHEN the version command runs
- THEN the full version string is displayed
- AND additional build information may be shown

#### Scenario: Show short version
- GIVEN `--short` flag is specified
- WHEN `sruja version --short` is executed
- THEN only the version number is displayed
- AND additional information is omitted

#### Scenario: Show detailed version
- GIVEN `--detailed` flag is specified
- WHEN `sruja version --detailed` is executed
- THEN version, commit hash, and build date are shown
- AND platform and toolchain information is included

### Requirement: Provide help information

The system SHALL provide a `help` command to display usage information.

#### Scenario: Show general help
- GIVEN `sruja help` or `sruja --help` is executed
- WHEN the help command runs
- THEN a list of all available commands is shown
- AND global flags are documented
- AND usage examples are provided

#### Scenario: Show command-specific help
- GIVEN `sruja help validate` is executed
- WHEN the help command runs
- THEN detailed help for the validate command is shown
- AND all flags for the command are documented
- AND examples are provided

#### Scenario: Display error for unknown command
- GIVEN an unknown command `sruja unknown-command` is executed
- WHEN the CLI attempts to execute
- THEN an error message is displayed
- AND a suggestion to use `--help` is provided
- AND the exit code indicates a usage error

### Requirement: Handle errors gracefully

The system SHALL provide clear, actionable error messages for all failure scenarios.

#### Scenario: File not found error
- GIVEN a specified file does not exist
- WHEN the CLI attempts to read the file
- THEN an error message is displayed
- AND the error indicates which file was not found
- AND suggestions for fixing the issue are provided

#### Scenario: Permission denied error
- GIVEN a file cannot be read due to permissions
- WHEN the CLI attempts to read the file
- THEN an error message is displayed
- AND the error indicates a permission issue
- AND suggestions for fixing permissions are provided

#### Scenario: Invalid flag error
- GIVEN an invalid flag is provided
- WHEN the CLI parses the command
- THEN an error message is displayed
- AND the invalid flag is shown
- AND a list of valid flags is suggested

#### Scenario: Invalid argument error
- GIVEN an invalid argument value is provided
- WHEN the CLI validates the arguments
- THEN an error message is displayed
- AND the invalid value is shown
- AND expected values are suggested

### Requirement: Support exit codes

The system SHALL use appropriate exit codes for different outcomes.

#### Scenario: Success exit code
- GIVEN a command executes successfully
- WHEN the command completes
- THEN the exit code is 0
- AND this indicates success to calling scripts

#### Scenario: Usage error exit code
- GIVEN invalid command-line arguments are provided
- WHEN the command fails to parse
- THEN the exit code is 2
- AND this indicates a usage error

#### Scenario: Validation error exit code
- GIVEN validation fails with errors
- WHEN the validate command completes
- THEN the exit code is 10
- AND this indicates validation errors were found

#### Scenario: Breaking changes exit code
- GIVEN breaking changes are detected during check
- WHEN the check command completes
- THEN the exit code is 20
- AND this indicates breaking changes were found

#### Scenario: Generic error exit code
- GIVEN an unexpected error occurs
- WHEN the command fails
- THEN the exit code is 1
- AND this indicates a generic error

### Requirement: Support auto-completion

The system SHALL provide shell completion for commands and flags.

#### Scenario: Bash completion
- GIVEN bash completion is installed
- WHEN the user types `sruja <TAB>`
- THEN available commands are suggested
- AND flags are suggested after commands
- AND file paths are completed where appropriate

#### Scenario: Zsh completion
- GIVEN zsh completion is installed
- WHEN the user types `sruja <TAB>`
- THEN available commands are suggested
- AND the completion respects zsh configuration

#### Scenario: Fish completion
- GIVEN fish completion is installed
- WHEN the user types `sruja <TAB>`
- THEN available commands are suggested
- AND descriptions are shown alongside suggestions

### Requirement: Respect environment variables

The system SHALL support configuration via environment variables.

#### Scenario: Set config path via environment variable
- GIVEN `SRUJA_CONFIG` environment variable is set
- WHEN the CLI is invoked
- THEN the config file at the specified path is loaded
- AND environment variable overrides CLI config flag

#### Scenario: Set API key via environment variable
- GIVEN `SRUJA_API_KEY` environment variable is set
- WHEN the CLI makes API calls
- THEN the API key from the environment is used
- AND no API key prompt is shown

#### Scenario: Set verbosity via environment variable
- GIVEN `SRUJA_VERBOSE=1` environment variable is set
- WHEN the CLI is invoked
- THEN verbose logging is enabled
- AND the behavior matches the `--verbose` flag