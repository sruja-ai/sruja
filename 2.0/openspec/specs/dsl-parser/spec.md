# DSL Parser Specification

## Purpose
Parse and convert Sruja's Domain Specific Language (DSL) into a structured model that can be validated, analyzed, and exported. The DSL serves as the source of truth for architecture descriptions.

## Requirements

### Requirement: Parse DSL file

The system SHALL parse a DSL file and convert it into a structured Model object.

#### Scenario: Parse simple architecture file
- GIVEN a DSL file exists at `architecture.sruja`
- AND the file contains a valid Person element definition
- WHEN the parser processes the file
- THEN a Model object is created successfully
- AND the Model contains the Person element
- AND the element has the correct ID, name, and description

#### Scenario: Parse file with multiple element types
- GIVEN a DSL file contains Person, System, and Container definitions
- AND the System contains nested Containers
- WHEN the parser processes the file
- THEN all elements are parsed correctly
- AND the hierarchy (System -> Containers) is preserved
- AND all element IDs are unique

#### Scenario: Parse relationships
- GIVEN a DSL file defines two elements
- AND a relationship connects them
- WHEN the parser processes the file
- THEN the Relationship is created
- AND the from/to references point to valid element IDs
- AND the label is preserved

### Requirement: Handle syntax errors

The system SHALL provide clear error messages when DSL syntax is invalid.

#### Scenario: Invalid element type
- GIVEN a DSL file contains an element with unknown type
- WHEN the parser processes the file
- THEN parsing fails with a syntax error
- AND the error message indicates the line number
- AND the error message suggests valid element types

#### Scenario: Missing required fields
- GIVEN a DSL file defines an element without a name
- WHEN the parser processes the file
- THEN parsing fails with a clear error
- AND the error indicates which field is missing
- AND the error shows the element's location

#### Scenario: Duplicate element IDs
- GIVEN a DSL file contains two elements with the same ID
- WHEN the parser processes the file
- THEN parsing fails with a duplicate ID error
- AND the error shows the conflicting IDs
- AND the error indicates line numbers for both occurrences

### Requirement: Support traceability links

The system SHALL parse and preserve links to external resources (ADRs, issues, PRs, URLs).

#### Scenario: Parse ADR link
- GIVEN an element includes a link to an ADR
- AND the link specifies a file path
- WHEN the parser processes the file
- THEN the link is stored in the element's links array
- AND the link type is set to "adr"
- AND the path is preserved

#### Scenario: Parse issue/PR link
- GIVEN an element includes a link to a GitHub issue
- AND the link specifies a URL
- WHEN the parser processes the file
- THEN the link is stored correctly
- AND the link type is set to "issue"
- AND the URL is preserved

### Requirement: Parse metadata

The system SHALL parse optional metadata including version, title, author, and timestamps.

#### Scenario: Parse file with metadata
- GIVEN a DSL file includes a version field
- AND includes title, author, and created timestamp
- WHEN the parser processes the file
- THEN metadata is extracted into the Model
- AND version follows semantic versioning format
- AND timestamps are in ISO 8601 format

#### Scenario: Parse file without metadata
- GIVEN a DSL file does not include metadata
- WHEN the parser processes the file
- THEN parsing succeeds
- AND default metadata values are used

### Requirement: Handle comments and whitespace

The system SHALL ignore comments and irrelevant whitespace during parsing.

#### Scenario: Parse file with single-line comments
- GIVEN a DSL file contains `//` style comments
- WHEN the parser processes the file
- THEN comments are ignored
- AND parsing succeeds normally

#### Scenario: Parse file with block comments
- GIVEN a DSL file contains `/* */` style multi-line comments
- WHEN the parser processes the file
- THEN comments are ignored
- AND parsing succeeds normally

### Requirement: Support quoted and unquoted strings

The system SHALL accept both quoted and unquoted string values for text fields.

#### Scenario: Parse quoted strings
- GIVEN an element name is enclosed in double quotes: `"Customer Service"`
- WHEN the parser processes the file
- THEN the quotes are stripped
- AND the value is stored as "Customer Service"

#### Scenario: Parse unquoted strings
- GIVEN an element name is not enclosed in quotes: CustomerService
- WHEN the parser processes the file
- THEN the value is stored as "CustomerService"

#### Scenario: Parse strings with spaces
- GIVEN an element name contains spaces: "Web Application"
- WHEN the parser processes the file
- THEN the name must be quoted
- AND the full string is preserved

### Requirement: Validate ID format

The system SHALL enforce a consistent ID format for all elements and relationships.

#### Scenario: Valid ID format
- GIVEN an element ID starts with a letter
- AND contains only alphanumeric characters, hyphens, and underscores
- WHEN the parser processes the file
- THEN the ID is accepted

#### Scenario: Invalid ID format
- GIVEN an element ID starts with a number
- OR contains special characters other than hyphen and underscore
- WHEN the parser processes the file
- THEN parsing fails
- AND the error message indicates invalid ID format
- AND suggestions for valid formats are provided