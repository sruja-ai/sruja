---
title: "Relations"
weight: 20
summary: "Relations describe how elements interact with each other."
---

# Relations

**Relations** describe how elements interact with each other. They are the lines connecting the boxes in your diagram.

## Syntax

```sruja
element system
element container
element datastore
element person

// Relations use element IDs
Source -> Destination "Label"
// When referring to nested elements, use fully qualified names:
System.Container -> System.Container.Component "Label"
```

Or with a technology/protocol:

```sruja
Source -> Destination "Label" {
technology "HTTPS/JSON"
}
```

## Example

```sruja
element system
element container
element datastore
element person

BankingSystem = system "Internet Banking System" {
WebApp = container "Web Application"
DB = datastore "Database"
}

User = person "User"

User -> BankingSystem.WebApp "Visits"
BankingSystem.WebApp -> BankingSystem.DB "Reads Data"
```

Use clear, unique IDs to reference relation endpoints.

## See Also

- [Scenario](/docs/concepts/scenario)
- [Validation](/docs/concepts/validation)
