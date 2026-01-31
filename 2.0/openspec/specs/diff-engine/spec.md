# Diff Engine Specification

## Purpose
Compare two architecture versions to identify changes, track evolution of the system over time, and generate detailed diff reports. The diff engine supports multiple output formats and can identify breaking changes that require review.

## Requirements

### Requirement: Compare architecture versions

The system SHALL compare two architecture models and identify all differences.

#### Scenario: Compare two different versions
- GIVEN an old architecture Model (v1.0.0)
- AND a new architecture Model (v2.0.0)
- WHEN the diff engine compares the models
- THEN a DiffResult is generated
- AND the result includes all detected changes
- AND the result includes a summary of changes

#### Scenario: Compare identical versions
- GIVEN an old architecture Model
- AND a new architecture Model with identical content
- WHEN the diff engine compares the models
- THEN a DiffResult is generated
- AND the result indicates no changes were detected
- AND all change arrays are empty

### Requirement: Detect added elements

The system SHALL identify elements that exist in the new version but not in the old version.

#### Scenario: Single added element
- GIVEN the old Model has elements ["service-a", "service-b"]
- AND the new Model has elements ["service-a", "service-b", "service-c"]
- WHEN the diff engine compares the models
- THEN the added_elements array contains "service-c"
- AND the element details are preserved

#### Scenario: Multiple added elements
- GIVEN the old Model has 3 elements
- AND the new Model has 5 elements
- WHEN the diff engine compares the models
- THEN the added_elements array contains 2 elements
- AND both elements are correctly identified

### Requirement: Detect deleted elements

The system SHALL identify elements that exist in the old version but not in the new version.

#### Scenario: Single deleted element
- GIVEN the old Model has elements ["service-a", "service-b", "service-c"]
- AND the new Model has elements ["service-a", "service-b"]
- WHEN the diff engine compares the models
- THEN the deleted_elements array contains "service-c"
- AND the element details from the old version are preserved

#### Scenario: Multiple deleted elements
- GIVEN the old Model has 5 elements
- AND the new Model has 3 elements
- WHEN the diff engine compares the models
- THEN the deleted_elements array contains 2 elements
- AND both elements are correctly identified

### Requirement: Detect modified elements

The system SHALL identify elements that exist in both versions but have changed properties.

#### Scenario: Element name changed
- GIVEN the old Model has an element "service" with name "Payment Service"
- AND the new Model has an element "service" with name "Payment Gateway"
- WHEN the diff engine compares the models
- THEN the modified_elements array contains "service"
- AND the changes include NameChanged with old and new values

#### Scenario: Element technology changed
- GIVEN the old Model has a container "api" with technology "Node.js"
- AND the new Model has a container "api" with technology "Python"
- WHEN the diff engine compares the models
- THEN the changes include TechnologyChanged
- AND the old value is "Node.js"
- AND the new value is "Python"

#### Scenario: Element description changed
- GIVEN the old Model has an element with description "Handles user authentication"
- AND the new Model has the same element with description "Manages user identity and access"
- WHEN the diff engine compares the models
- THEN the changes include DescriptionChanged
- AND both old and new descriptions are preserved

#### Scenario: Multiple changes to one element
- GIVEN the old Model has an element with name "Auth", technology "Java", and description "Auth service"
- AND the new Model has the same element with name "Identity", technology "Kotlin", and description "Identity service"
- WHEN the diff engine compares the models
- THEN the element appears in modified_elements once
- AND three changes are detected: NameChanged, TechnologyChanged, DescriptionChanged

### Requirement: Detect added links

The system SHALL identify new links added to existing elements.

#### Scenario: ADR link added
- GIVEN the old Model has an element "service" with 0 links
- AND the new Model has an element "service" with 1 ADR link
- WHEN the diff engine compares the models
- THEN the changes include LinksAdded
- AND the new ADR link is listed

#### Scenario: Multiple links added
- GIVEN the old Model has an element with 2 links
- AND the new Model has the same element with 5 links
- WHEN the diff engine compares the models
- THEN the changes include LinksAdded
- AND 3 new links are listed

### Requirement: Detect removed links

The system SHALL identify links removed from existing elements.

#### Scenario: Single link removed
- GIVEN the old Model has an element "service" with 3 links
- AND the new Model has the same element with 2 links
- WHEN the diff engine compares the models
- THEN the changes include LinksRemoved
- AND the removed link is listed

#### Scenario: All links removed
- GIVEN the old Model has an element with 5 links
- AND the new Model has the same element with 0 links
- WHEN the diff engine compares the models
- THEN the changes include LinksRemoved
- AND 5 removed links are listed

### Requirement: Detect added relationships

The system SHALL identify relationships that exist in the new version but not in the old version.

#### Scenario: Single added relationship
- GIVEN the old Model has 5 relationships
- AND the new Model has 6 relationships
- WHEN the diff engine compares the models
- THEN the added_relationships array contains 1 relationship
- AND the relationship details are preserved

#### Scenario: Relationship added between existing elements
- GIVEN the old Model has elements "service-a" and "service-b"
- AND the old Model has no relationship between them
- AND the new Model has a relationship from "service-a" to "service-b"
- WHEN the diff engine compares the models
- THEN the relationship appears in added_relationships

### Requirement: Detect removed relationships

The system SHALL identify relationships that exist in the old version but not in the new version.

#### Scenario: Single removed relationship
- GIVEN the old Model has 6 relationships
- AND the new Model has 5 relationships
- WHEN the diff engine compares the models
- THEN the removed_relationships array contains 1 relationship
- AND the relationship details from the old version are preserved

#### Scenario: Relationship between existing elements removed
- GIVEN the old Model has elements "service-a" and "service-b"
- AND the old Model has a relationship from "service-a" to "service-b"
- AND the new Model has no relationship between them
- WHEN the diff engine compares the models
- THEN the relationship appears in removed_relationships

### Requirement: Detect breaking changes

The system SHALL identify changes that are potentially breaking to the system's functionality.

#### Scenario: Container removed (breaking)
- GIVEN the old Model has a container "payment-service"
- AND the new Model does not have this container
- WHEN the diff engine detects breaking changes
- THEN a breaking change is reported
- AND the change indicates "payment-service" was deleted

#### Scenario: Relationship to critical element removed (breaking)
- GIVEN the old Model has a relationship to "database"
- AND the new Model does not have this relationship
- WHEN the diff engine detects breaking changes
- THEN a breaking change is reported
- AND the change indicates a critical dependency was removed

#### Scenario: Element technology changed (potentially breaking)
- GIVEN the old Model has a container with technology "Java"
- AND the new Model has the same container with technology "Go"
- WHEN the diff engine detects breaking changes
- THEN a breaking change may be reported
- AND the change indicates technology stack migration

#### Scenario: Non-breaking change
- GIVEN the old Model has an element with description "Payment processor"
- AND the new Model has the same element with description "Handles payments"
- WHEN the diff engine detects breaking changes
- THEN no breaking change is reported
- AND the change is considered informational only

### Requirement: Generate diff in multiple formats

The system SHALL support multiple output formats for the diff report.

#### Scenario: Generate text format
- GIVEN a diff result is computed
- AND the requested format is "text"
- WHEN the diff engine generates output
- THEN a human-readable text report is produced
- AND changes are listed by type
- AND the report is readable in a terminal

#### Scenario: Generate JSON format
- GIVEN a diff result is computed
- AND the requested format is "json"
- WHEN the diff engine generates output
- THEN a JSON object is produced
- AND the JSON includes all diff data
- AND the JSON is machine-parseable

#### Scenario: Generate Markdown format
- GIVEN a diff result is computed
- AND the requested format is "markdown"
- WHEN the diff engine generates output
- THEN a Markdown document is produced
- AND the document uses proper headings for sections
- AND changes are formatted with Markdown lists

#### Scenario: Generate HTML format
- GIVEN a diff result is computed
- AND the requested format is "html"
- WHEN the diff engine generates output
- THEN an HTML document is produced
- AND the document is styled for readability
- AND changes are highlighted appropriately

### Requirement: Calculate diff statistics

The system SHALL provide summary statistics for the diff result.

#### Scenario: Calculate change counts
- GIVEN a diff result is computed
- AND there are 2 added elements, 1 deleted element, 3 modified elements
- AND there are 5 added relationships and 2 removed relationships
- WHEN the diff engine generates statistics
- THEN the summary includes count of added elements (2)
- AND the summary includes count of deleted elements (1)
- AND the summary includes count of modified elements (3)
- AND the summary includes count of added relationships (5)
- AND the summary includes count of removed relationships (2)

#### Scenario: Determine if breaking changes exist
- GIVEN a diff result is computed
- AND breaking changes are detected
- WHEN the diff engine generates statistics
- THEN the summary indicates has_breaking_changes: true
- AND a list of breaking changes is provided

#### Scenario: No breaking changes detected
- GIVEN a diff result is computed
- AND no breaking changes are detected
- WHEN the diff engine generates statistics
- THEN the summary indicates has_breaking_changes: false

### Requirement: Preserve element and relationship details

The system SHALL include full details of added, deleted, and modified elements and relationships.

#### Scenario: Preserve added element details
- GIVEN a new element is added
- WHEN the diff engine generates the result
- THEN the added_elements array includes the full element object
- AND all fields (id, name, kind, description, etc.) are present

#### Scenario: Preserve deleted element details
- GIVEN an element is deleted
- WHEN the diff engine generates the result
- THEN the deleted_elements array includes the full element from the old version
- AND all fields are preserved

#### Scenario: Preserve relationship details
- GIVEN a relationship is added or removed
- WHEN the diff engine generates the result
- THEN the relationship includes all fields (from, to, label, direction, protocol, async)
- AND all relationships in the result are complete

### Requirement: Handle renamed elements

The system SHALL detect when an element is renamed (appears as deleted and added).

#### Scenario: Element renamed
- GIVEN the old Model has an element "payment-service" with description "Process payments"
- AND the new Model has no element named "payment-service"
- AND the new Model has an element "payment-gateway" with the same description
- WHEN the diff engine compares the models
- THEN "payment-service" appears in deleted_elements
- AND "payment-gateway" appears in added_elements
- AND the system may suggest this is a rename based on similarity

### Requirement: Support partial diffs

The system SHALL allow filtering the diff to focus on specific changes.

#### Scenario: Filter by element type
- GIVEN a diff result is computed
- AND the user requests only Container changes
- WHEN the diff engine filters the result
- THEN only Container-related changes are shown
- AND Person and System changes are excluded

#### Scenario: Filter by change type
- GIVEN a diff result is computed
- AND the user requests only added elements
- WHEN the diff engine filters the result
- THEN only added_elements are shown
- AND other change types are excluded

#### Scenario: Filter by breaking changes
- GIVEN a diff result is computed
- AND the user requests only breaking changes
- WHEN the diff engine filters the result
- THEN only breaking changes are shown
- AND informational changes are excluded