# Validator Specification

## Purpose
Validate architecture models parsed from the DSL to ensure structural integrity, reference consistency, and adherence to architectural best practices. The validator produces errors for invalid models and warnings for potential issues.

## Requirements

### Requirement: Validate unique element IDs

The system SHALL ensure that all element IDs within an architecture are unique.

#### Scenario: All IDs are unique
- GIVEN a parsed Model contains elements with IDs "user", "system", "database"
- WHEN the validator checks element IDs
- THEN no duplicate ID errors are reported
- AND validation passes

#### Scenario: Duplicate element IDs
- GIVEN a parsed Model contains two elements with ID "service"
- WHEN the validator checks element IDs
- THEN a duplicate ID error is reported
- AND the error indicates the conflicting ID "service"
- AND the error shows the line numbers or locations of both elements

### Requirement: Validate relationship references

The system SHALL ensure all relationships reference valid element IDs.

#### Scenario: All relationships have valid references
- GIVEN a parsed Model has a relationship from "web-app" to "api-gateway"
- AND both elements exist in the model
- WHEN the validator checks relationship references
- THEN no reference errors are reported
- AND validation passes

#### Scenario: Invalid from reference
- GIVEN a parsed Model has a relationship from "non-existent" to "api-gateway"
- AND the element "non-existent" does not exist
- WHEN the validator checks relationship references
- THEN a reference error is reported
- AND the error indicates "non-existent" is not a valid element
- AND the error shows the relationship ID

#### Scenario: Invalid to reference
- GIVEN a parsed Model has a relationship from "web-app" to "missing-element"
- AND the element "missing-element" does not exist
- WHEN the validator checks relationship references
- THEN a reference error is reported
- AND the error indicates "missing-element" is not a valid element

#### Scenario: Both references invalid
- GIVEN a parsed Model has a relationship from "bad-from" to "bad-to"
- AND neither element exists
- WHEN the validator checks relationship references
- THEN two reference errors are reported
- AND each error indicates the invalid reference

### Requirement: Detect orphaned elements

The system SHALL identify elements that are not connected to any other elements.

#### Scenario: Connected elements
- GIVEN a parsed Model has Person, System, and Container elements
- AND relationships connect all elements
- WHEN the validator checks for orphans
- THEN no orphan warnings are reported

#### Scenario: Orphaned element detected
- GIVEN a parsed Model has an element "isolated-service"
- AND no relationships reference "isolated-service"
- WHEN the validator checks for orphans
- THEN a warning is reported for "isolated-service"
- AND the warning indicates the element has no connections
- AND the warning suggests this may be intentional or an error

#### Scenario: Multiple orphaned elements
- GIVEN a parsed Model has three elements with no connections
- WHEN the validator checks for orphans
- THEN three orphan warnings are reported
- AND each warning lists a different orphaned element

### Requirement: Validate system structure

The system SHALL ensure Systems contain at least one Container unless marked as external.

#### Scenario: System with containers
- GIVEN a parsed Model has a System element "backend-system"
- AND the System contains two Container elements
- WHEN the validator validates system structure
- THEN no structure errors are reported

#### Scenario: Empty system without external flag
- GIVEN a parsed Model has a System element "empty-system"
- AND the System has no containers
- AND the System is not marked as external
- WHEN the validator validates system structure
- THEN an error is reported
- AND the error indicates Systems should contain containers or be marked as external

#### Scenario: External system without containers
- GIVEN a parsed Model has a System element "third-party-api"
- AND the System has no containers
- AND the System is marked as external: true
- WHEN the validator validates system structure
- THEN no structure errors are reported

### Requirement: Validate relationship direction

The system SHALL ensure bidirectional relationships have corresponding reverse relationships.

#### Scenario: Unidirectional relationship
- GIVEN a parsed Model has a unidirectional relationship from "client" to "server"
- WHEN the validator validates relationship direction
- THEN no direction errors are reported

#### Scenario: Bidirectional relationship with reverse
- GIVEN a parsed Model has a bidirectional relationship from "service-a" to "service-b"
- AND there is a corresponding relationship from "service-b" to "service-a"
- WHEN the validator validates relationship direction
- THEN no direction errors are reported

#### Scenario: Bidirectional relationship without reverse
- GIVEN a parsed Model has a bidirectional relationship from "service-a" to "service-b"
- AND there is no relationship from "service-b" to "service-a"
- WHEN the validator validates relationship direction
- THEN a warning is reported
- AND the warning indicates bidirectional relationships should have reverse relationships

### Requirement: Validate protocol consistency

The system SHALL warn about relationships with missing or invalid protocols.

#### Scenario: Relationship with protocol
- GIVEN a parsed Model has a relationship with protocol "HTTP"
- WHEN the validator validates protocol
- THEN no protocol warnings are reported

#### Scenario: Relationship without protocol
- GIVEN a parsed Model has a relationship without a protocol field
- WHEN the validator validates protocol
- THEN a warning is reported
- AND the warning suggests adding a protocol for clarity

#### Scenario: Relationship with common protocol
- GIVEN a parsed Model has a relationship with protocol "HTTPS"
- OR protocol "gRPC"
- OR protocol "AMQP"
- WHEN the validator validates protocol
- THEN the protocol is recognized as valid

### Requirement: Validate container technology

The system SHALL ensure Containers specify their technology stack.

#### Scenario: Container with technology
- GIVEN a parsed Model has a Container element
- AND the Container specifies technology "Spring Boot"
- WHEN the validator validates container technology
- THEN no technology errors are reported

#### Scenario: Container without technology
- GIVEN a parsed Model has a Container element
- AND the Container does not specify technology
- WHEN the validator validates container technology
- THEN an error is reported
- AND the error indicates technology is required for Containers

#### Scenario: Non-container without technology
- GIVEN a parsed Model has a Person element
- AND the Person does not specify technology
- WHEN the validator validates container technology
- THEN no technology errors are reported
- AND the technology field is optional for non-containers

### Requirement: Support strict validation mode

The system SHALL provide a strict mode that treats warnings as errors.

#### Scenario: Strict mode with warnings
- GIVEN strict mode is enabled
- AND a parsed Model has an orphaned element
- WHEN the validator processes the model
- THEN the validation fails
- AND the orphan warning is treated as an error

#### Scenario: Normal mode with warnings
- GIVEN strict mode is disabled (default)
- AND a parsed Model has an orphaned element
- WHEN the validator processes the model
- THEN validation passes
- AND the orphan is reported as a warning

### Requirement: Collect validation statistics

The system SHALL provide summary statistics after validation.

#### Scenario: Successful validation statistics
- GIVEN a parsed Model has 10 elements and 15 relationships
- WHEN the validator processes the model successfully
- THEN a ValidationResult is returned
- AND the result includes statistics showing 10 elements and 15 relationships
- AND the result shows counts by element type (Person, System, Container)

#### Scenario: Failed validation statistics
- GIVEN a parsed Model has validation errors
- WHEN the validator processes the model
- THEN a ValidationResult is returned
- AND the result lists all errors
- AND the result indicates validation failed
- AND statistics are still provided

### Requirement: Provide error locations

The system SHALL include line numbers and locations in error messages.

#### Scenario: Error with location
- GIVEN a parsed Model has a duplicate ID on line 15
- WHEN the validator reports an error
- THEN the error includes line number 15
- AND the error shows the offending element or relationship

#### Scenario: Multiple errors with locations
- GIVEN a parsed Model has errors on lines 5, 20, and 35
- WHEN the validator reports errors
- THEN each error includes its line number
- AND errors are sorted by line number for clarity