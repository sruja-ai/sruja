# Lesson 2: Data Flow Diagrams

## Learning Goals

- Create DFD-style data flows in Sruja
- Model data lineage and transformations
- Document ETL and analytics pipelines

## What Are Data Flow Diagrams?

Data Flow Diagrams (DFDs) show how data moves through a system, including:

- Where data originates
- Where it's stored
- How it's transformed
- Where it ultimately goes

## DFD Elements in Sruja

### `flow` for Data-Oriented Flows

```sruja
OrderDataFlow = flow "Order Data Processing" {
  Customer -> Shop.WebApp "Order form data"
  Shop.WebApp -> Shop.API "Order JSON"
  Shop.API -> Shop.Database "Order record"
  Shop.Database -> Analytics "Order event"
  Analytics -> Dashboard "Aggregated metrics"
}
```

## DFD Patterns

### Pattern 1: ETL Pipeline

```sruja
ETLPipeline = flow "Data ETL Pipeline" {
  SourceSystem -> DataCollector "Raw data"
  DataCollector -> MessageQueue "Data events"
  MessageQueue -> DataProcessor "Consumes data"
  DataProcessor -> DataWarehouse "Transformed data"
  DataWarehouse -> ReportingEngine "Query results"
  ReportingEngine -> Dashboard "Visualizations"
}
```

### Pattern 2: Event Sourcing

```sruja
EventFlow = flow "Event Sourcing Pipeline" {
  API -> EventStore "Persist events"
  EventStore -> ProjectorA "Project to read model A"
  EventStore -> ProjectorB "Project to read model B"
  ProjectorA -> ReadDatabaseA "Read model A"
  ProjectorB -> ReadDatabaseB "Read model B"
}
```

### Pattern 3: Analytics Pipeline

```sruja
AnalyticsFlow = flow "User Analytics Pipeline" {
  UserApp -> TrackingService "User actions"
  TrackingService -> EventStream "Raw events"
  EventStream -> BatchProcessor "Daily batch"
  BatchProcessor -> DataWarehouse "Aggregated data"
  DataWarehouse -> ReportingTool "Analytics queries"
  ReportingTool -> BusinessTeam "Insights"
}
```

### Pattern 4: Real-Time Processing

```sruja
RealTimeFlow = flow "Real-Time Fraud Detection" {
  TransactionAPI -> IngestService "Transaction data"
  IngestService -> KafkaStream "Event stream"
  KafkaStream -> FraudDetectionService "Consume events"
  FraudDetectionService -> AlertService "Fraud alerts"
  AlertService -> SecurityTeam "Notifications"
}
```

## Documenting Data Transformations

### Use Relationship Labels

```sruja
DataFlow = flow "Data Transformation" {
  RawSource -> ETLService "Raw CSV data"
  ETLService -> CleanedData "Validated, normalized data"
  CleanedData -> Aggregator "Aggregated metrics"
  Aggregator -> DataWarehouse "Hourly aggregations"
}
```

### Add Metadata for Details

```sruja
ETLService = container "ETL Service" {
  metadata {
    transformations [
      "Remove invalid records",
      "Normalize phone numbers",
      "Standardize dates"
    ]
    output_format "JSON"
    output_schema "v2"
  }
}
```

## Complete DFD Example

```sruja
import { * } from 'sruja.ai/stdlib'

Customer = person "Customer"

Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "PostgreSQL"
}

Analytics = system "Analytics Platform" {
  Ingestion = container "Data Ingestion"
  Processing = container "Data Processing"
  Warehouse = database "Data Warehouse"
  Reporting = container "Reporting Engine"
}

Dashboard = system "Analytics Dashboard" {
  UI = container "Dashboard UI"
}

// Data flow: Order processing and analytics
OrderAnalyticsFlow = flow "Order Analytics Pipeline" {
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Order data"
  Shop.API -> Shop.Database "Persist order"

  // Real-time data capture
  Shop.API -> Analytics.Ingestion "Order event"
  Analytics.Ingestion -> Analytics.Processing "Validates and enriches"
  Analytics.Processing -> Analytics.Warehouse "Stores aggregated data"

  // Query and visualization
  Dashboard.UI -> Analytics.Reporting "Query metrics"
  Analytics.Reporting -> Analytics.Warehouse "Fetch data"
  Analytics.Reporting -> Dashboard.UI "Return results"
}

view index {
  include *
}
```

## Data Lineage Tracing

### Forward Tracing

```sruja
// Where does this data go?
OrderFlow = flow "Order Data Lineage" {
  OrderAPI -> Database "Save order"
  Database -> ReplicationService "Replicate to secondary"
  ReplicationService -> AnalyticsDB "Stream to analytics"
  AnalyticsDB -> ReportGenerator "Generate reports"
}
```

### Backward Tracing

```sruja
// Where does this data come from?
ReportFlow = flow "Report Data Source" {
  UserActivityReport <- AnalyticsDB "Aggregated data"
  AnalyticsDB <- EventStream "Raw events"
  EventStream <- UserApp "User actions"
}
```

## Error Handling in Flows

### Document Error Paths

```sruja
OrderFlow = scenario "Order Processing with Errors" {
  Customer -> Shop.API "Submit order"
  Shop.API -> PaymentGateway "Process payment"

  // Success path
  PaymentGateway -> Shop.API "Payment success"
  Shop.API -> Shop.Database "Save order"

  // Error path
  PaymentGateway -> Shop.API "Payment failed"
  Shop.API -> Shop.WebApp "Return error"
  Shop.WebApp -> Customer "Show error message"
}
```

### Retry Logic

```sruja
Shop.API = container "API Service" {
  metadata {
    retry_policy {
      max_attempts 3
      backoff "exponential"
      initial_delay "1s"
    }
  }
}
```

## Performance Considerations

### Document Latency Expectations

```sruja
PaymentGateway = system "Payment Gateway" {
  metadata {
    expected_latency "500ms"
    timeout "5s"
  }
}

OrderFlow = scenario "Order Processing" {
  Customer -> Shop.WebApp "Submit order" [user_interaction]
  Shop.WebApp -> Shop.API "Send order" [internal_fast]
  Shop.API -> PaymentGateway "Process payment" [external_slower]
}
```

### Identify Bottlenecks

```sruja
ProcessingFlow = flow "File Processing Pipeline" {
  Upload -> Storage "Store file" [fast]
  Storage -> Processor "Process file" [bottleneck]
  Processor -> Notification "Notify user" [fast]
}
```

## Exercise

Create a DFD for:

> "A fitness tracking app where users log workouts. Workout data is sent to an API, stored in a database, and also sent to a real-time analytics service. The analytics service processes events and updates user dashboards. Daily, a batch job aggregates data and generates reports stored in a data warehouse for business analysis."

**Create:**

- Main data flow
- At least one data transformation
- Real-time and batch processing paths

## Key Takeaways

1. **Use `flow` for DFDs**: Data-oriented flows
2. **Show transformations**: How data changes
3. **Document lineage**: Where data comes from and goes
4. **Handle errors**: Show success and failure paths
5. **Identify bottlenecks**: Where processing slows down

## Next Lesson

In [Lesson 3](lesson-3.md), you'll learn how to model user journeys and behavioral scenarios.
