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
architecture "Web Application" {
  web = container "Web Frontend" {
    technology "React"
    description "User interface"
  }

  api = container "API Service" {
    technology "Node.js + Express"
    description "RESTful API"
  }

  database = database "Database" {
    technology "PostgreSQL"
    description "Data storage"
  }

  message_queue = queue "Message Queue" {
    technology "RabbitMQ"
    description "Event streaming"
  }
}

# ✅ Specific, descriptive labels
web -> api "HTTPS"
api -> database "PostgreSQL (JDBC)"
api -> message_queue "AMQP (publishes events)"
```

### Example 2: Purpose-Specific Labels

```sruja
architecture "Order System" {
  order_api = container "Order API" {
    technology "Go"
    description "Order processing"
  }

  payment_service = container "Payment Service" {
    technology "Python"
    description "Payment processing"
  }

  inventory_service = container "Inventory Service" {
    technology "Java"
    description "Inventory management"
  }

  notification_service = container "Notification Service" {
    technology "Node.js"
    description "Notifications"
  }
}

# ✅ Clear purpose in labels
order_api -> payment_service "REST API (process payment)"
order_api -> inventory_service "REST API (reserve stock)"
payment_service -> notification_service "REST API (send receipt)"
```

### Example 3: Data Flow Direction

```sruja
architecture "Analytics System" {
  collector = container "Data Collector" {
    technology "Python"
    description "Collects events from multiple sources"
  }

  processor = container "Stream Processor" {
    technology "Kafka Streams"
    description "Processes and transforms data"
  }

  database = database "Time Series DB" {
    technology "InfluxDB"
    description "Stores time-series data"
  }

  api = container "Analytics API" {
    technology "Go"
    description "Query interface for analytics"
  }
}

# ✅ Clear data flow
collector -> processor "publishes events"
processor -> database "writes metrics"
api -> database "reads time-series data"
```

## Incorrect Approach

```sruja
# ❌ Vague, non-descriptive labels
web -> api "uses"
api -> database "connects to"
api -> message_queue "sends data"

# ❌ Inconsistent naming
web -> api "HTTP"
api -> database "JDBC"
api -> message_queue "publishes"
```

## Common Mistakes

1. **Vague Labels**: "uses", "connects to", "talks to"
   - ❌ `web -> api "uses"`
   - ✅ `web -> api "HTTPS"`

2. **Missing Protocol**: Not specifying communication protocol
   - ❌ `api -> database "reads"`
   - ✅ `api -> database "PostgreSQL (JDBC)"`

3. **Inconsistent Naming**: Mix of styles
   - ❌ "HTTPS", "connects", "API call", "publishes events"
   - ✅ "HTTPS", "REST API", "PostgreSQL", "AMQP"

4. **Purpose Not Clear**: Labels don't explain what's happening
   - ❌ `service_a -> service_b "API"`
   - ✅ `service_a -> service_b "REST API (fetches orders)"`

## Best Practices

### 1. Include Protocol

```sruja
# ✅ Good
frontend -> api "HTTPS"
api -> database "PostgreSQL"
worker -> queue "AMQP"

# ❌ Avoid
frontend -> api "uses"
api -> database "connects"
```

### 2. Specify Purpose

```sruja
# ✅ Good
api -> payment_service "REST API (process payment)"
api -> inventory_service "REST API (check stock)"

# ❌ Avoid
api -> payment_service "API"
api -> inventory_service "API"
```

### 3. Show Data Flow Direction

```sruja
# ✅ Good
producer -> queue "publishes events"
consumer -> queue "consumes events"
api -> database "reads orders"
database -> api "returns results"

# ❌ Avoid
producer -> queue "uses"
consumer -> queue "uses"
api -> database "query"
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
# ✅ Consistent style
service_a -> service_b "REST API"
service_b -> service_c "REST API"
service_c -> database "PostgreSQL"

# ❌ Inconsistent
service_a -> service_b "HTTP"
service_b -> service_c "REST API"
service_c -> database "connects"
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