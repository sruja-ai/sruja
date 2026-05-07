---
title: "Lesson 2: Data Flow Diagrams"
weight: 2
summary: "Modeling DFD-style data flows and lineage"
time: "5 min"
---

# Following the Trail: Data Flow Diagrams

Think of an oil pipeline. Crude oil goes in one end, flows through refineries where it's heated, distilled, and chemically treated, and comes out the other end as gasoline, diesel, or jet fuel. At each stage, the oil transforms into something more valuable.

Data flows work the same way. Raw data enters your system, flows through transformations where it's validated, normalized, enriched, and aggregated, and comes out as insights, reports, or visualizations.

In this lesson, you'll learn to create DFD-style data flows in Sruja. You'll discover how to track data lineage, document transformations, and model the pipelines that power your analytics and reporting.

Let's start by understanding what data flows are and why they matter.

## Learning Goals

By the end of this lesson, you'll be able to:

- Create DFD-style data flows in Sruja
- Model data lineage from source to destination
- Document data transformations and how data changes shape
- Design ETL and analytics pipelines
- Track where data comes from and where it ultimately goes

## What Are Data Flow Diagrams, Really?

Data Flow Diagrams (DFDs) show how data moves through your system—where it originates, how it's stored, how it transforms, and where it ends up.

Think of it like tracing a river's path:
- **Source**: Where the river starts (a spring, a mountain lake)
- **Flow**: The river's journey through valleys and cities
- **Transformations**: Tributaries joining, diversions splitting, dams changing flow
- **Destination**: Where the river ends (ocean, another river)

In data terms:
- **Source**: Where data originates (user input, database, API, file)
- **Flow**: The path data takes through your system
- **Transformations**: Validation, normalization, enrichment, aggregation
- **Destination**: Where data ultimately goes (warehouse, dashboard, report)

## Why Data Flows Matter: The Real Benefits

I've built countless data systems over the years, and data flows are always the first thing I create. Here's why.

### 1. Data Lineage: Where Did This Come From?

Data flows tell you the complete history of data—where it started and every transformation it went through.

```sruja
// partial
CustomerAnalyticsFlow = flow "Customer Data Lineage" {
  // Source: Where data starts
  Customer -> CRMSystem "Creates customer profile"
  
  // Transformation 1: Data extraction
  CRMSystem -> ETLService "Extracts customer records"
  
  // Transformation 2: Data normalization
  ETLService -> NormalizedData "Cleans and standardizes formats"
  
  // Transformation 3: Data enrichment
  NormalizedData -> EnrichmentService "Adds behavioral data from clickstream"
  
  // Transformation 4: Aggregation
  EnrichmentService -> AggregatedData "Creates daily customer segments"
  
  // Destination: Where data ends up
  AggregatedData -> DataWarehouse "Stores for reporting"
  DataWarehouse -> BusinessDashboard "Displays customer segments"
}
```

This flow tells you the complete story: customer data originates in CRM, gets extracted by ETL, normalized (cleaned up), enriched with behavioral data, aggregated into segments, stored in the warehouse, and ultimately shows up on a business dashboard.

Without this flow, would you know customer data comes from the CRM? Would you know it gets enriched with clickstream data? Would you know it's aggregated daily? Probably not.

I once worked on a project where nobody knew where analytics data came from. We spent weeks tracking down data lineage every time we found an issue. We added data flows, and suddenly everyone knew the complete path.

### 2. Process Understanding: What Actually Happens?

Data flows reveal the processing steps your data goes through—the "how" not just the "what."

```sruja
// partial
ETLPipelineFlow = flow "ETL Pipeline Steps" {
  // Step 1: Extraction
  SourceDatabase -> IngestionService "Pulls raw transactions"
  
  // Step 2: Validation
  IngestionService -> ValidationService "Validates schema and data types"
  
  // Step 3: Transformation
  ValidationService -> TransformationService "Normalizes dates, currencies, formats"
  
  // Step 4: Loading
  TransformationService -> DataWarehouse "Loads transformed data"
}
```

This shows you the complete ETL process: extract, validate, transform, load. You can see exactly what each service does and in what order.

When something breaks—a data quality issue, a failed load, a malformed record—you know exactly where to look. Is it in ingestion? In validation? In transformation? The flow tells you.

### 3. Transformation Documentation: How Does Data Change?

Data flows document how data transforms at each step—what shape it takes, what format it's in.

```sruja
// partial
TransformationFlow = flow "Data Transformations" {
  RawSource -> ETLService "Raw CSV file"
  
  // Transformation 1: Validation
  ETLService -> ValidatedData "Validated (removed invalid records)"
  
  // Transformation 2: Normalization
  ValidatedData -> NormalizedData "Normalized (standardized formats)"
  
  // Transformation 3: Enrichment
  NormalizedData -> EnrichedData "Enriched (added location data)"
  
  // Transformation 4: Aggregation
  EnrichedData -> FinalData "Aggregated (daily metrics)"
}
```

Each arrow shows a transformation:
- Raw CSV → Validated (invalid records removed)
- Validated → Normalized (formats standardized)
- Normalized → Enriched (location data added)
- Enriched → Aggregated (metrics computed)

This documentation is invaluable. When someone asks, "What happened to this data?" you can point to the flow and show them each transformation step.

I once inherited a system where nobody documented data transformations. We found mysterious records in the warehouse—dates in the wrong format, currencies mixed up, values that made no sense. We spent months reverse-engineering what transformations were happening. Document it upfront.

### 4. Bottleneck Identification: Where Will Things Slow Down?

Data flows make bottlenecks obvious—where processing might slow down, where queues might form, where latency will be worst.

```sruja
// partial
AnalyticsFlow = flow "Analytics Pipeline" {
  UserActions -> TrackingService "Captures events" [fast]
  TrackingService -> EventStream "Publishes events" [fast]
  EventStream -> BatchProcessor "Consumes and processes" [slow, batch job]
  BatchProcessor -> DataWarehouse "Loads aggregated data" [medium]
  DataWarehouse -> Dashboard "Queries for display" [fast]
}
```

Look at the labels: fast, fast, **slow**, medium, fast. The batch processor is marked as slow because it's a scheduled job that runs once daily. This tells you immediately: if you're looking for real-time analytics, you'll be disappointed. The bottleneck is the batch processor.

When users complain about stale data ("why does the dashboard show yesterday's numbers?"), you know exactly why. The flow tells you.

## Creating Data Flows in Sruja

Sruja gives you the `flow` keyword for creating DFD-style data flows. It's designed specifically for data-oriented flows.

### Using `flow` for Data Pipelines

```sruja
// partial
OrderDataFlow = flow "Order Data Processing" {
  Customer -> WebApp "Order form submission"
  WebApp -> API "Order JSON payload"
  API -> Database "Persist order record"
  Database -> AnalyticsExtractor "Extract order events"
  AnalyticsExtractor -> EventStream "Publish to analytics"
  EventStream -> DataWarehouse "Aggregate and store"
  DataWarehouse -> ReportingTool "Query for reports"
}
```

### Using Metadata for Transformations

You can add metadata to document what each step does:

```sruja
// partial
ETLService = container "ETL Service" {
  metadata {
    transformations [
      "Validate schema and data types",
      "Normalize dates to ISO 8601",
      "Standardize currency codes to ISO 4217",
      "Remove invalid or corrupt records"
    ]
    output_format "JSON"
    output_schema "v2.1"
    batch_window "Daily at 2AM UTC"
  }
}
```

This metadata tells anyone reading the flow:
- What transformations happen
- What output format to expect
- What schema version
- When the batch runs

## Common Data Flow Patterns

After building data systems for years, I've noticed patterns that repeat constantly. Let me show you the ones I see most often.

### Pattern 1: ETL Pipeline

Extract, Transform, Load—the classic pattern for moving data from operational systems to analytics.

```sruja
// partial
ETLPipelineFlow = flow "Classic ETL Pipeline" {
  // Extract: Pull from source systems
  TransactionDB -> DataCollector "Extracts daily transactions"
  CustomerDB -> DataCollector "Extracts customer profiles"
  
  // Transform: Clean and normalize
  DataCollector -> ValidationService "Validates schemas"
  ValidationService -> CleaningService "Removes duplicates and errors"
  CleaningService -> TransformationService "Normalizes formats"
  
  // Load: Push to warehouse
  TransformationService -> DataWarehouse "Loads transformed data"
  DataWarehouse -> ReportingEngine "Available for queries"
}
```

**Characteristics:**
- Scheduled batch processing (daily, hourly)
- Source systems are OLTP (transactional)
- Destination is OLAP (analytics)
- Focus on data quality and consistency

**Use when:** Building traditional data warehouses, moving from transactional systems to analytics.

### Pattern 2: Event Sourcing

Every change to data is captured as an event, and different services project events into read models.

```sruja
// partial
EventSourcingFlow = flow "Event Sourcing Pattern" {
  // Events captured
  OrderAPI -> EventStore "Persist OrderCreated event"
  OrderAPI -> EventStore "Persist OrderPaid event"
  OrderAPI -> EventStore "Persist OrderShipped event"
  
  // Multiple projections
  EventStore -> OrderReadModel "Project to order summary view"
  EventStore -> CustomerReadModel "Project to customer order history view"
  EventStore -> AnalyticsReadModel "Project to order metrics view"
  
  // Read models queried
  OrderReadModel -> OrderService "Fetch order details"
  CustomerReadModel -> CustomerService "Fetch customer orders"
  AnalyticsReadModel -> AnalyticsService "Fetch order metrics"
}
```

**Characteristics:**
- Events are immutable (never change)
- Multiple read models for different use cases
- Rebuildable (can replay events)
- Eventually consistent

**Use when:** Building systems where audit trails matter, where you need multiple views of same data, where rebuildability is important.

### Pattern 3: Real-Time Analytics Pipeline

Events flow through a real-time processing pipeline for immediate insights.

```sruja
// partial
RealTimeAnalyticsFlow = flow "Real-Time Analytics Pipeline" {
  // Events captured
  UserActions -> EventCollector "Captures clickstream events"
  EventCollector -> KafkaStream "Publishes to Kafka"
  
  // Real-time processing
  KafkaStream -> StreamProcessor "Processes events in real-time"
  StreamProcessor -> RedisCache "Updates user session data"
  StreamProcessor -> Elasticsearch "Indexes events for search"
  
  // Real-time consumption
  RedisCache -> WebApp "Serves session data"
  Elasticsearch -> Dashboard "Shows real-time user activity"
}
```

**Characteristics:**
- Real-time (seconds to minutes latency)
- Stream processing (Kafka, Kinesis, Pulsar)
- Eventually consistent (some delay acceptable)
- Focus on speed and availability

**Use when:** Building real-time dashboards, fraud detection, personalized recommendations, live monitoring.

### Pattern 4: Lambda Architecture

Batch processing for comprehensive analytics plus real-time for speed.

```sruja
// partial
LambdaArchitectureFlow = flow "Lambda Architecture" {
  // Speed layer: Real-time
  Events -> StreamProcessing "Real-time processing"
  StreamProcessing -> SpeedLayer "Serves fast views"
  
  // Batch layer: Comprehensive
  Events -> BatchProcessing "Comprehensive processing"
  BatchProcessing -> BatchLayer "Serves accurate views"
  
  // Serving layer: Merges both
  SpeedLayer -> QueryService "Provides fast results"
  BatchLayer -> QueryService "Provides accurate results"
  QueryService -> API "Serves merged views"
}
```

**Characteristics:**
- Two paths: fast (speed layer) and accurate (batch layer)
- Speed layer provides quick but possibly incomplete results
- Batch layer provides comprehensive but delayed results
- Query service merges both for best of both worlds

**Use when:** You need both real-time responsiveness and comprehensive accuracy.

## Documenting Data Transformations

One of the most important things you can do in data flows is document transformations clearly.

### Using Relationship Labels

```sruja
// partial
TransformFlow = flow "Data Transformations" {
  RawSource -> ETLService "Raw CSV data"
  ETLService -> ValidatedData "Validated (removed invalids)"
  ValidatedData -> NormalizedData "Normalized (standardized)"
  NormalizedData -> EnrichedData "Enriched (added location)"
  EnrichedData -> FinalData "Aggregated (daily metrics)"
}
```

Each label describes what transformation happened at that step.

### Adding Metadata

```sruja
// partial
ETLService = container "ETL Service" {
  metadata {
    transformations [
      "Remove duplicate records",
      "Normalize phone numbers to E.164 format",
      "Standardize dates to ISO 8601 (UTC)",
      "Geocode addresses to lat/long"
    ]
    input_format "CSV"
    output_format "JSON"
    output_schema "v2.1"
  }
}
```

This metadata provides complete documentation of what transformations happen.

## Complete Data Flow Example

Let me show you a complete example that brings everything together.

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// People
Customer = person "Customer"

// Systems
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

// Complete data flow: Order to analytics
OrderAnalyticsFlow = flow "Order Analytics Pipeline" {
  // Source: Customer creates order
  Customer -> Shop.WebApp "Submits order"
  Shop.WebApp -> Shop.API "Order data"
  Shop.API -> Shop.Database "Persist order"
  
  // Extraction: Pull orders for analytics
  Shop.Database -> Analytics.Ingestion "Extract order events"
  
  // Transformation: Validate and enrich
  Analytics.Ingestion -> Analytics.Processing "Validate and normalize"
  Analytics.Processing -> Analytics.Processing "Enrich with customer data"
  Analytics.Processing -> Analytics.Processing "Aggregate metrics"
  
  // Loading: Store in warehouse
  Analytics.Processing -> Analytics.Warehouse "Store aggregated data"
  
  // Consumption: Query and display
  Dashboard.UI -> Analytics.Reporting "Query metrics"
  Analytics.Reporting -> Analytics.Warehouse "Fetch data"
  Analytics.Reporting -> Dashboard.UI "Return results"
}

view index {
  include *
}
```

This flow shows the complete path from customer action to analytics dashboard. Anyone reading this diagram understands how data moves through the system.

## What to Remember

Data flows tell the story of how data moves through your system—from origin to destination, including every transformation along the way. When you create data flows:

- **Document lineage** — Where data comes from and where it goes
- **Show transformations** — How data changes shape at each step
- **Use metadata** — Document what each service actually does
- **Identify bottlenecks** — Mark slow steps and understand their impact
- **Choose right pattern** — ETL, event sourcing, real-time, or lambda
- **Track both paths** — Success and failure paths

If you take away one thing, let it be this: **data flows are your best documentation of how data actually moves through your system**. When someone asks, "Where did this data come from?" or "What happened to this data?" your data flow has the answer.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're modeling a fitness tracking app's data flow. The app tracks user workouts, syncs them to a cloud API, where they're stored in a database. A daily ETL job extracts workouts, calculates daily metrics (calories burned, workout duration), and stores results in a data warehouse for business analytics. Which flow type is most appropriate?

> "Users log workouts on their phones. Workout data syncs to the cloud API, gets stored in the main database. Every night at 2 AM, an ETL job pulls all workouts from the database, calculates aggregated metrics (total calories, total minutes, workout counts per user), and loads the results into a data warehouse. Business analysts query the warehouse for reports on user engagement and app usage."

**A)** User Journey / Scenario
**B)** Control Flow
**C)** Data Flow (DFD Style)
**D)** Event Flow

<details>
<summary>Click to see the answer</summary>

**Answer: C) Data Flow (DFD Style)**

Let's analyze each option:

**A)** Incorrect. A user journey shows how a user interacts with a system to achieve a goal. This scenario describes **data movement and transformations**, not user interactions. The user creates a workout, but the rest of the flow (syncing, extracting, calculating, aggregating, loading) happens automatically without user involvement. A user journey wouldn't capture these data processing steps effectively.

**B)** Incorrect. A control flow shows decision points and branching logic (if/else). This scenario describes a **pipeline** where data flows through sequential steps (sync → store → extract → calculate → aggregate → load). There's no conditional logic—every workout follows the same path through the ETL pipeline. Control flows are better for modeling things like "if workout is跑步类型 A, calculate calories differently" or "if user is premium, store additional metrics."

**C)** Correct! A data flow (DFD-style) is the right choice here because:
- The scenario describes **data lineage** — where data starts (user workout), where it goes (cloud API, database, data warehouse), and what happens along the way
- It shows **data transformations** — raw workout → synced workout → extracted workout → calculated metrics → aggregated metrics
- It models a **pipeline** — sequential steps that process data (extract, transform, load)
- It's focused on **data movement**, not user actions or business logic

Data flows (DFD-style) are perfect for showing how data moves through a system, including where it originates, how it's stored, how it's transformed, and where it ultimately goes. This scenario is a classic ETL pipeline—extract from operational database, transform (calculate metrics), load into data warehouse.

**D)** Incorrect. An event flow would show how events propagate through an event-driven system (pub/sub patterns). This scenario describes a **scheduled batch ETL job**, not real-time event propagation. The ETL job pulls data daily at 2 AM, transforms it, and loads it. There's no event bus, no event streaming, no multiple consumers processing the same event. Event flows would show things like "workout completed event published → analytics service consumes event → notification service consumes event → recommendation service consumes event."

**Key insight:** Choose flow types based on what you're modeling. Showing where data comes from, how it transforms, and where it ends up? Use a data flow. Modeling user interactions and experience? Use a user journey. Modeling decision logic and branches? Use a control flow. Designing event-driven architecture? Use an event flow.

</details>

---

### Question 2

You're creating a data flow for an e-commerce platform. Which structure best documents the data transformations that happen in the ETL pipeline?

**A)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  TransactionDB -> ETLService "Extract"
  ETLService -> DataWarehouse "Load"
}
```

**B)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  TransactionDB -> ETLService "Extract transactions"
  ETLService -> ValidatedData "Validate and remove errors"
  ValidatedData -> NormalizedData "Normalize formats"
  NormalizedData -> EnrichedData "Add customer data"
  EnrichedData -> DataWarehouse "Load to warehouse"
}
```

**C)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  TransactionDB -> DataWarehouse "Move data"
}
```

**D)**
```sruja
// partial
ETLPipeline = flow "ETL Pipeline" {
  TransactionDB -> ETLService "Extract"
  ETLService -> ValidatedData "?"
  ValidatedData -> NormalizedData "?"
  NormalizedData -> EnrichedData "?"
  EnrichedData -> DataWarehouse "?"
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: B) Shows each transformation step clearly**

Let's analyze each option:

**A)** Incorrect. This flow has only two steps: extract and load. It completely skips the transformation step. ETL stands for **Extract, Transform, Load**—the transformation is the middle T! This flow doesn't show what transformations happen. Are transactions validated? Are formats normalized? Is data enriched? The flow provides no information about these crucial steps. Anyone reading this diagram wouldn't understand what actually happens to the data.

**B)** Correct! This flow documents each transformation step clearly:
- **Extract transactions** — Pulls raw data from transactional database
- **Validate and remove errors** — First transformation: validates data quality, removes corrupt or invalid records
- **Normalize formats** — Second transformation: standardizes dates, currencies, phone numbers, etc. to consistent formats
- **Add customer data** — Third transformation: enriches transactions with customer information (name, tier, location, etc.)
- **Load to warehouse** — Final step: loads transformed, enriched data into warehouse

Each relationship label describes what transformation happens at that step. Anyone reading this diagram understands the complete ETL process and what happens to the data at each stage.

**C)** Incorrect. This is too abstract. "Move data" tells you nothing about what happens. Is data validated? Is it transformed? Is it enriched? How does the format change? What transformations are applied? The flow provides no useful information. It's the equivalent of saying "data goes from point A to point B" without explaining the journey.

**D)** Incorrect. While this has the right number of steps, the labels are meaningless ("?"). What does the first "?" mean? What about the second "?"? The third "?"? The fourth "?"? These labels provide no information about what transformations are happening. Each step is a black box—you know there are transformations, but you don't know what they are.

**Key insight:** Document transformations clearly using descriptive relationship labels. Don't just show that data moves—show **how** it transforms. Label each step with what actually happens: "validate," "normalize," "enrich," "aggregate," "calculate." This makes your data flows informative and useful, not just correct.

</details>

---

## What's Next?

Now you understand how to create data flow diagrams. You can model data lineage, document transformations, and design ETL and analytics pipelines.

But data flows are just one type of flow. There's another crucial type—**user journeys** (or behavioral flows)—which show how users interact with your system from their perspective.

In the next lesson, you'll learn about user journeys. You'll discover how to model BDD-style scenarios, document happy paths and error paths, and capture the complete user experience from start to finish.

See you there!
