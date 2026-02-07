---
title: "Healthcare Portal: Connect Patient to System"
summary: "A patient needs to book an appointment! Model the complete flow: Patient uses the Portal, Portal calls the Appointment API, and API stores data in the database."
difficulty: beginner
topic: relations
estimatedTime: "5-10 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Patient = person "Healthcare Patient"

  HealthPortal = system "Healthcare Portal" {
    Portal = container "Patient Portal"
    AppointmentAPI = container "Appointment Service"
    RecordsDB = datastore "Patient Records Database"
  }

  // TODO: Connect Patient -> Portal -> AppointmentAPI -> RecordsDB
  // Think about what each interaction represents
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: relationExists
    source: Patient
    target: Portal
    message: "Add relation Patient -> Portal"
  - type: relationExists
    source: Portal
    target: AppointmentAPI
    message: "Add relation Portal -> AppointmentAPI"
  - type: relationExists
    source: AppointmentAPI
    target: RecordsDB
    message: "Add relation AppointmentAPI -> RecordsDB"
hints:
  - 'Start with Patient -> Portal "Books appointment"'
  - 'Then Portal -> AppointmentAPI "Requests appointment"'
  - 'Finally AppointmentAPI -> RecordsDB "Stores appointment"'
  - "Remember: all relation labels must be in quotes"
solution: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  datastore = kind "Datastore"

  Patient = person "Healthcare Patient"

  HealthPortal = system "Healthcare Portal" {
    Portal = container "Patient Portal"
    AppointmentAPI = container "Appointment Service"
    RecordsDB = datastore "Patient Records Database"
  }

  Patient -> HealthPortal.Portal "Books appointment"
  HealthPortal.Portal -> HealthPortal.AppointmentAPI "Requests appointment"
  HealthPortal.AppointmentAPI -> HealthPortal.RecordsDB "Stores appointment"
---
