# Module 3: Tactical Design

## Tactical Design

Tactical Design provides a set of patterns for creating the domain model itself. It deals with the details of the implementation.

## Core Patterns

### Entity
An object defined by its **identity**, not just its attributes. Two entities with different attributes but the same ID are the same entity.
- Example: `Customer`, `Order`.

### Value Object
An object defined by its **attributes**, with no conceptual identity. They are immutable.
- Example: `Address`, `Money`, `DateRange`.

### Aggregate
A cluster of associated objects that are treated as a unit for the purpose of data changes.
- **Aggregate Root**: The only entity that external objects can hold a reference to.
- Ensures consistency boundaries.

## Sruja Syntax

```sruja
context Sales {
    aggregate Order {
        entity OrderLineItem {
            name string
            quantity int
        }
        
        valueObject ShippingAddress {
            street string
            city string
            zipCode string
        }
    }
}
```
