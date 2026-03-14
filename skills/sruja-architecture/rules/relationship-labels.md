# relationship-labels

## Why It Matters

Clear, descriptive relationship labels make architectures self-documenting and unambiguous. They communicate protocols, data flow direction, and purpose at a glance, reducing misinterpretation and improving communication.

## When to Apply

Always use specific, descriptive labels when:

- Defining relationships between components
- Documenting APIs and integrations
- Reviewing architectures for clarity
- Explaining system behavior to stakeholders

## Correct Approach

### Example 1: Protocol-Specific Labels

```sruja
Web = container "Web Frontend" {
  technology "React"
  description "User interface"
}

API = container "API Service" {
  technology "Node.js + Express"
  description "RESTful API"
}

Database = database "Database" {
  technology "PostgreSQL"
  description "Data storage"
}

MessageQueue = queue "Message Queue" {
  technology "RabbitMQ"
  description "Event streaming"
}

// ✅ Specific, descriptive labels
Web -> API "HTTPS"
API -> Database "PostgreSQL (JDBC)"
API -> MessageQueue "AMQP (publishes events)"
```

### Example 2: Purpose-Specific Labels

```sruja
OrderAPI = container "Order API" {
  technology "Go"
  description "Order processing"
}

PaymentService = container "Payment Service" {
  technology "Python"
  description "Payment processing"
}

InventoryService = container "Inventory Service" {
  technology "Java"
  description "Inventory management"
}

NotificationService = container "Notification Service" {
  technology "Node.js"
  description "Notifications"
}

// ✅ Clear purpose in labels
OrderAPI -> PaymentService "REST API (process payment)"
OrderAPI -> InventoryService "REST API (reserve stock)"
PaymentService -> NotificationService "REST API (send receipt)"
```

### Example 3: Data Flow Direction

```sruja
Collector = container "Data Collector" {
  technology "Python"
  description "Collects events from multiple sources"
}

Processor = container "Stream Processor" {
  technology "Kafka Streams"
  description "Processes and transforms data"
}

TimeSeriesDB = database "Time Series DB" {
  technology "InfluxDB"
  description "Stores time-series data"
}

AnalyticsAPI = container "Analytics API" {
  technology "Go"
  description "Query interface for analytics"
}

// ✅ Clear data flow
Collector -> Processor "publishes events"
Processor -> TimeSeriesDB "writes metrics"
AnalyticsAPI -> TimeSeriesDB "reads time-series data"
```

## Incorrect Approach

```sruja
// ❌ Vague, non-descriptive labels
Web -> API "uses"
API -> Database "connects to"
API -> MessageQueue "sends data"

// ❌ Inconsistent naming
Web -> API "HTTP"
API -> Database "JDBC"
API -> MessageQueue "publishes"
```

## Common Mistakes

1. **Vague Labels**: "uses", "connects to", "talks to"
   - ❌ `Web -> API "uses"`
   - ✅ `Web -> API "HTTPS"`

2. **Missing Protocol**: Not specifying communication protocol
   - ❌ `API -> Database "reads"`
   - ✅ `API -> Database "PostgreSQL (JDBC)"`

3. **Inconsistent Naming**: Mix of styles
   - ❌ "HTTPS", "connects", "API call", "publishes events"
   - ✅ "HTTPS", "REST API", "PostgreSQL", "AMQP"

4. **Purpose Not Clear**: Labels don't explain what's happening
   - ❌ `ServiceA -> ServiceB "API"`
   - ✅ `ServiceA -> ServiceB "REST API (fetches orders)"`

## Best Practices

### 1. Include Protocol

```sruja
// ✅ Good
Frontend -> API "HTTPS"
API -> Database "PostgreSQL"
Worker -> Queue "AMQP"

// ❌ Avoid
Frontend -> API "uses"
API -> Database "connects"
```

### 2. Specify Purpose

```sruja
// ✅ Good
API -> PaymentService "REST API (process payment)"
API -> InventoryService "REST API (check stock)"

// ❌ Avoid
API -> PaymentService "API"
API -> InventoryService "API"
```

### 3. Show Data Flow Direction

```sruja
// ✅ Good
Producer -> Queue "publishes events"
Consumer -> Queue "consumes events"
API -> Database "reads orders"
Database -> API "returns results"

// ❌ Avoid
Producer -> Queue "uses"
Consumer -> Queue "uses"
API -> Database "query"
```

### 4. Use Standard Protocol Names

```sruja
# ✅ Standard protocols
"HTTPS"
"HTTP/2"
"gRPC"
"REST API"
"WebSocket"
"PostgreSQL"
"JDBC"
"AMQP"
"Kafka"
"Redis"

# ❌ Avoid custom names
"web protocol"
"database connection"
"message passing"
```

### 5. Be Consistent

```sruja
// ✅ Consistent style
ServiceA -> ServiceB "REST API"
ServiceB -> ServiceC "REST API"
ServiceC -> Database "PostgreSQL"

// ❌ Inconsistent
ServiceA -> ServiceB "HTTP"
ServiceB -> ServiceC "REST API"
ServiceC -> Database "connects"
```

## Label Templates

### Synchronous Communication

- `HTTPS`
- `HTTP`
- `gRPC`
- `REST API`
- `GraphQL`
- `WebSocket`

### Database Access

- `PostgreSQL (JDBC)`
- `MySQL (ODBC)`
- `MongoDB (driver)`
- `Redis (client)`
- `reads from`
- `writes to`
- `reads/writes`

### Asynchronous/Messaging

- `AMQP`
- `Kafka`
- `publishes events to`
- `subscribes to`
- `consumes events from`
- `emits events to`

### External Integrations

- `REST API (3rd party)`
- `GraphQL API (external)`
- `OAuth 2.0`
- `S3 API`
- `Stripe API`

## Additional Context

Good labels are critical for:

- Architecture documentation
- Code generation from architecture
- Understanding data flow
- Security reviews
- Performance analysis

Related rules:

- `relationship-synchronous` - When to use synchronous communication
- `relationship-asynchronous` - When to use async messaging
- `relationship-direction` - Showing clear data flow
- `tradeoff-sync-vs-async` - Choosing communication patterns

## References

- REST API Design Best Practices
- gRPC Design Patterns
- Event-Driven Architecture Patterns
- API Documentation Standards