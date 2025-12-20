# Sruja Technical Architecture Specification

## Overview

This document defines the technical architecture for evolving Sruja into an Integrated SDLC Intelligence Platform. It builds upon the existing LikeC4 foundation while adding the capabilities needed for the Architecture Knowledge Graph (AKG), continuous data ingestion, and advanced analytics.

## Architecture Principles

1. **Incremental Evolution**: Build upon existing LikeC4 foundation without breaking changes
2. **Plugin-First Design**: Extensible through plugins and integrations
3. **Real-Time Processing**: Support live data streaming and updates
4. **Multi-Modal Storage**: Use appropriate databases for different data types
5. **API-First**: All functionality accessible through well-defined APIs
6. **Cloud-Native**: Designed for containerization and orchestration
7. **Security by Default**: Zero-trust architecture with proper authentication/authorization

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Presentation Layer                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Web UI     │ │   VS Code    │ │   CLI/API    │             │
│  │   (React)    │ │   Extension  │ │   Client     │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                        API Gateway                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │    REST      │ │   GraphQL    │ │  WebSocket   │             │
│  │   Gateway    │ │    Server    │ │   Server     │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                      Service Layer                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   AKG Core   │ │   Analytics   │ │ Integration   │             │
│  │   Service    │ │   Service     │ │   Service     │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Risk       │ │   Scenario   │ │   Security    │             │
│  │   Service    │ │   Service     │ │   Service     │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                      Data Processing Layer                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Stream     │ │   Batch      │ │   Real-time  │             │
│  │   Processor  │ │   Processor  │ │   Validator  │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                        Data Layer                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Graph DB   │ │ Time Series  │ │  Document    │             │
│  │   (Neo4j)    │ │   (Influx)   │ │   Store      │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │   Cache      │ │   Search     │ │   Object     │             │
│  │   (Redis)    │ │  (Elastic)   │ │   Store      │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. AKG Core Service

**Location**: `pkg/akg/`

**Responsibilities**:

- Manage the Architecture Knowledge Graph
- Handle CRUD operations for architectural entities
- Maintain graph consistency and integrity
- Provide query interfaces for complex graph traversals

**Key APIs**:

```typescript
// Entity Management
POST   /api/v1/entities
GET    /api/v1/entities/:id
PUT    /api/v1/entities/:id
DELETE /api/v1/entities/:id

// Relationship Management
POST   /api/v1/relationships
GET    /api/v1/relationships/:id
DELETE /api/v1/relationships/:id

// Graph Queries
POST   /api/v1/graph/query
GET    /api/v1/graph/traverse/:entityId
```

**Data Model**:

```typescript
interface Entity {
  id: string;
  type: "system" | "service" | "component" | "database" | "queue";
  name: string;
  description: string;
  metadata: {
    owner: string;
    team: string;
    technology: string;
    version: string;
    lifecycle: "dev" | "staging" | "prod" | "deprecated";
    slos: SLO[];
    dataClassification: "public" | "internal" | "confidential" | "pii";
  };
  createdAt: Date;
  updatedAt: Date;
}

interface Relationship {
  id: string;
  sourceId: string;
  targetId: string;
  type: "api_call" | "data_flow" | "async_message" | "dependency";
  metadata: {
    protocol: string;
    version: string;
    frequency: number;
    latency: number;
  };
}
```

### 2. Integration Service

**Location**: `pkg/integrations/`

**Responsibilities**:

- Connect to external data sources
- Ingest and normalize data
- Handle authentication and rate limiting
- Provide plugin framework for custom integrations

**Integration Types**:

#### Git Integration

```typescript
interface GitIntegration {
  provider: "github" | "gitlab" | "bitbucket";
  repository: string;
  branch: string;
  webhookUrl: string;

  // Events to process
  events: ["push", "pull_request", "merge"];

  // Data extraction
  extractCodeStructure: boolean;
  trackCommits: boolean;
  analyzeDependencies: boolean;
}
```

#### CI/CD Integration

```typescript
interface CICDIntegration {
  provider: "jenkins" | "github-actions" | "gitlab-ci";
  pipeline: string;

  // Metrics to track
  metrics: ["build_time", "success_rate", "deployment_frequency"];

  // Webhook configuration
  webhookUrl: string;
  events: ["build_started", "build_completed", "deployment"];
}
```

#### Observability Integration

```typescript
interface ObservabilityIntegration {
  metrics: {
    provider: "prometheus" | "graphite";
    endpoint: string;
    queries: MetricQuery[];
  };

  logs: {
    provider: "elasticsearch" | "loki";
    endpoint: string;
    indexes: string[];
  };

  traces: {
    provider: "jaeger" | "zipkin";
    endpoint: string;
  };
}
```

### 3. Analytics Service

**Location**: `pkg/analytics/`

**Responsibilities**:

- Perform risk analysis and detection
- Calculate architectural metrics
- Generate insights and recommendations
- Support what-if scenario analysis

**Risk Detection Engine**:

```typescript
interface RiskDetection {
  // Architectural risks
  detectCouplingHotspots(): Risk[];
  detectCyclicDependencies(): Risk[];
  detectSPOFs(): Risk[];
  detectArchitecturalDrift(): Risk[];

  // Delivery risks
  detectLongLeadTimes(): Risk[];
  detectFragilePipelines(): Risk[];
  detectLowDeploymentFrequency(): Risk[];

  // Reliability risks
  detectSLOViolations(): Risk[];
  detectHighMTTR(): Risk[];
  detectUnownedServices(): Risk[];

  // Security risks
  detectVulnerabilities(): Risk[];
  detectInsecureDataFlows(): Risk[];
  detectTrustBoundaryViolations(): Risk[];
}
```

**Scenario Analysis**:

```typescript
interface ScenarioAnalysis {
  // Change simulation
  simulateEntityChange(entityId: string, change: EntityChange): Impact;
  simulateRelationshipChange(relationshipId: string, change: RelationshipChange): Impact;

  // What-if analysis
  analyzeDecommissioning(entityId: string): DecommissioningImpact;
  analyzeNewService(service: Entity): ServiceImpact;
  analyzeTeamReorganization(teams: Team[]): OrganizationalImpact;
}
```

### 4. Data Processing Layer

**Location**: `pkg/processing/`

**Responsibilities**:

- Stream processing for real-time data
- Batch processing for historical analysis
- Data validation and normalization
- Event routing and distribution

**Stream Processing**:

```typescript
interface StreamProcessor {
  // Event processing
  processGitEvent(event: GitEvent): Promise<void>;
  processCICDEvent(event: CICDEvent): Promise<void>;
  processObservabilityEvent(event: ObservabilityEvent): Promise<void>;

  // Data enrichment
  enrichWithMetadata(data: RawData): Promise<EnrichedData>;
  correlateWithEntities(data: EnrichedData): Promise<CorrelatedData>;
}
```

**Batch Processing**:

```typescript
interface BatchProcessor {
  // Historical analysis
  processHistoricalMetrics(timeRange: TimeRange): Promise<Metrics[]>;
  generateTrendReports(entityId: string): Promise<TrendReport>;

  // Data synchronization
  syncWithExternalSystems(): Promise<SyncResult>;
  validateDataIntegrity(): Promise<ValidationResult>;
}
```

## Database Schema

### Graph Database (Neo4j)

**Nodes**:

```cypher
// Entity nodes
(:Entity {
  id: string,
  type: string,
  name: string,
  description: string,
  owner: string,
  team: string,
  technology: string,
  lifecycle: string,
  createdAt: datetime,
  updatedAt: datetime
})

// Risk nodes
(:Risk {
  id: string,
  type: string,
  severity: string,
  description: string,
  detectedAt: datetime,
  status: string
})
```

**Relationships**:

```cypher
// Architectural relationships
(:Entity)-[:DEPENDS_ON {type: string, protocol: string}]->(:Entity)
(:Entity)-[:EXPOSES_API {version: string, endpoint: string}]->(:Entity)
(:Entity)-[:READS_FROM]->(:Entity)
(:Entity)-[:WRITES_TO]->(:Entity)

// Risk relationships
(:Entity)-[:HAS_RISK]->(:Risk)
(:Risk)-[:AFFECTS]->(:Entity)
```

### Time Series Database (InfluxDB)

**Measurements**:

```sql
-- Performance metrics
performance,entity_id=service-1 cpu_usage=0.75,memory_usage=0.60 1609459200000000000

-- SLO metrics
slo_attainment,entity_id=service-1,slo_type=availability value=99.95 1609459200000000000

-- Delivery metrics
delivery_metrics,entity_id=service-1 lead_time=1800,deployment_frequency=24 1609459200000000000
```

### Document Store (MongoDB)

**Collections**:

```json
// Integration configurations
{
  "_id": "integration-1",
  "type": "git",
  "provider": "github",
  "config": {
    "repository": "org/repo",
    "webhookUrl": "https://api.sruja.io/webhooks/git"
  },
  "status": "active",
  "lastSync": "2023-12-01T10:00:00Z"
}

// User preferences
{
  "_id": "user-1",
  "persona": "architect",
  "preferences": {
    "defaultView": "system-overview",
    "riskThreshold": "medium",
    "notifications": ["email", "slack"]
  }
}
```

## API Specification

### REST API Endpoints

#### Authentication

```typescript
POST / api / v1 / auth / login;
POST / api / v1 / auth / logout;
POST / api / v1 / auth / refresh;
GET / api / v1 / auth / profile;
```

#### Entities

```typescript
GET    /api/v1/entities                    // List all entities
POST   /api/v1/entities                    // Create new entity
GET    /api/v1/entities/:id               // Get specific entity
PUT    /api/v1/entities/:id               // Update entity
DELETE /api/v1/entities/:id               // Delete entity
GET    /api/v1/entities/:id/dependencies  // Get entity dependencies
GET    /api/v1/entities/:id/metrics       // Get entity metrics
```

#### Relationships

```typescript
GET    /api/v1/relationships              // List all relationships
POST   /api/v1/relationships              // Create new relationship
GET    /api/v1/relationships/:id         // Get specific relationship
DELETE /api/v1/relationships/:id         // Delete relationship
```

#### Analytics

```typescript
GET    /api/v1/analytics/risks            // Get all risks
GET    /api/v1/analytics/risks/:id        // Get specific risk
POST   /api/v1/analytics/scenarios        // Run scenario analysis
GET    /api/v1/analytics/metrics/:entityId // Get entity metrics
POST   /api/v1/analytics/what-if          // Run what-if analysis
```

#### Integrations

```typescript
GET    /api/v1/integrations               // List all integrations
POST   /api/v1/integrations               // Create new integration
GET    /api/v1/integrations/:id          // Get specific integration
PUT    /api/v1/integrations/:id          // Update integration
DELETE /api/v1/integrations/:id          // Delete integration
POST   /api/v1/integrations/:id/sync      // Trigger manual sync
```

### GraphQL Schema

```graphql
type Entity {
  id: ID!
  type: EntityType!
  name: String!
  description: String
  owner: String!
  team: String!
  technology: String
  lifecycle: Lifecycle!
  slos: [SLO!]
  dependencies: [Entity!]
  dependents: [Entity!]
  risks: [Risk!]
  metrics: [Metric!]
  createdAt: DateTime!
  updatedAt: DateTime!
}

type Risk {
  id: ID!
  type: RiskType!
  severity: RiskSeverity!
  description: String!
  entity: Entity!
  detectedAt: DateTime!
  status: RiskStatus!
  recommendations: [String!]
}

type Query {
  entity(id: ID!): Entity
  entities(filter: EntityFilter): [Entity!]
  risks(filter: RiskFilter): [Risk!]
  analytics: AnalyticsQuery!
}

type Mutation {
  createEntity(input: CreateEntityInput!): Entity!
  updateEntity(id: ID!, input: UpdateEntityInput!): Entity!
  deleteEntity(id: ID!): Boolean!
  createRelationship(input: CreateRelationshipInput!): Relationship!
  runScenarioAnalysis(input: ScenarioAnalysisInput!): ScenarioResult!
}
```

## Deployment Architecture

### Container Strategy

```yaml
# docker-compose.yml
version: "3.8"

services:
  # API Gateway
  api-gateway:
    image: sruja/api-gateway:latest
    ports:
      - "8080:8080"
    environment:
      - GRAPHQL_ENDPOINT=http://graphql-service:8081
      - REST_ENDPOINT=http://rest-service:8082

  # GraphQL Service
  graphql-service:
    image: sruja/graphql-service:latest
    ports:
      - "8081:8081"
    environment:
      - NEO4J_URI=bolt://neo4j:7687
      - REDIS_URI=redis://redis:6379

  # REST Service
  rest-service:
    image: sruja/rest-service:latest
    ports:
      - "8082:8082"
    environment:
      - NEO4J_URI=bolt://neo4j:7687
      - MONGODB_URI=mongodb://mongodb:27017

  # Analytics Service
  analytics-service:
    image: sruja/analytics-service:latest
    ports:
      - "8083:8083"
    environment:
      - NEO4J_URI=bolt://neo4j:7687
      - INFLUXDB_URI=http://influxdb:8086

  # Integration Service
  integration-service:
    image: sruja/integration-service:latest
    ports:
      - "8084:8084"
    environment:
      - KAFKA_URI=kafka:9092
      - REDIS_URI=redis://redis:6379

  # Web UI
  web-ui:
    image: sruja/web-ui:latest
    ports:
      - "3000:3000"
    environment:
      - API_ENDPOINT=http://api-gateway:8080

  # Databases
  neo4j:
    image: neo4j:4.4
    ports:
      - "7474:7474"
      - "7687:7687"
    environment:
      - NEO4J_AUTH=neo4j/password
    volumes:
      - neo4j_data:/data

  mongodb:
    image: mongo:5.0
    ports:
      - "27017:27017"
    volumes:
      - mongodb_data:/data/db

  influxdb:
    image: influxdb:2.0
    ports:
      - "8086:8086"
    environment:
      - INFLUXDB_DB=sruja
    volumes:
      - influxdb_data:/var/lib/influxdb2

  redis:
    image: redis:6.2
    ports:
      - "6379:6379"

  # Message Queue
  kafka:
    image: confluentinc/cp-kafka:latest
    ports:
      - "9092:9092"
    environment:
      - KAFKA_ZOOKEEPER_CONNECT=zookeeper:2181
      - KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://localhost:9092

  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    ports:
      - "2181:2181"
    environment:
      - ZOOKEEPER_CLIENT_PORT=2181

volumes:
  neo4j_data:
  mongodb_data:
  influxdb_data:
```

### Kubernetes Deployment

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sruja

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: sruja-config
  namespace: sruja
data:
  NEO4J_URI: "bolt://neo4j-service:7687"
  MONGODB_URI: "mongodb://mongodb-service:27017"
  INFLUXDB_URI: "http://influxdb-service:8086"
  REDIS_URI: "redis://redis-service:6379"
  KAFKA_URI: "kafka-service:9092"

---
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sruja-api-gateway
  namespace: sruja
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sruja-api-gateway
  template:
    metadata:
      labels:
        app: sruja-api-gateway
    spec:
      containers:
        - name: api-gateway
          image: sruja/api-gateway:latest
          ports:
            - containerPort: 8080
          envFrom:
            - configMapRef:
                name: sruja-config
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
```

## Security Architecture

### Authentication & Authorization

```typescript
// JWT Token Structure
interface JWTPayload {
  sub: string; // User ID
  email: string;
  persona: "cxo" | "architect" | "developer" | "sre";
  permissions: Permission[];
  orgId: string;
  iat: number;
  exp: number;
}

// Permission Model
enum Permission {
  READ_ENTITY = "entity:read",
  WRITE_ENTITY = "entity:write",
  DELETE_ENTITY = "entity:delete",
  READ_RISKS = "risks:read",
  MANAGE_INTEGRATIONS = "integrations:manage",
  RUN_ANALYTICS = "analytics:run",
  ADMIN = "admin",
}

// Role-Based Access Control
const ROLE_PERMISSIONS = {
  cxo: [Permission.READ_ENTITY, Permission.READ_RISKS],
  architect: [
    Permission.READ_ENTITY,
    Permission.WRITE_ENTITY,
    Permission.READ_RISKS,
    Permission.RUN_ANALYTICS,
  ],
  developer: [Permission.READ_ENTITY, Permission.WRITE_ENTITY, Permission.READ_RISKS],
  sre: [Permission.READ_ENTITY, Permission.READ_RISKS, Permission.MANAGE_INTEGRATIONS],
};
```

### Data Security

```typescript
// Encryption at Rest
interface EncryptionConfig {
  algorithm: "AES-256-GCM";
  keyRotation: "90d";
  fields: ["slos", "metadata.sensitive"];
}

// Data Masking
interface DataMasking {
  pii: {
    strategy: "hash";
    algorithm: "SHA-256";
  };
  confidential: {
    strategy: "mask";
    pattern: "***-****-****";
  };
}

// Audit Logging
interface AuditLog {
  timestamp: Date;
  userId: string;
  action: string;
  resource: string;
  result: "success" | "failure";
  metadata: Record<string, any>;
}
```

## Monitoring & Observability

### Application Metrics

```typescript
// Business Metrics
interface BusinessMetrics {
  userCount: number;
  activeIntegrations: number;
  entityCount: number;
  riskCount: number;
  scenarioAnalysesRun: number;
}

// Technical Metrics
interface TechnicalMetrics {
  requestLatency: Histogram;
  errorRate: Counter;
  throughput: Gauge;
  resourceUtilization: Gauge;
}
```

### Health Checks

```typescript
// Health Check Endpoints
GET / health / live; // Liveness probe
GET / health / ready; // Readiness probe
GET / health / detailed; // Detailed health status

// Health Check Response
interface HealthStatus {
  status: "healthy" | "unhealthy" | "degraded";
  services: {
    database: ServiceHealth;
    cache: ServiceHealth;
    messageQueue: ServiceHealth;
    integrations: ServiceHealth;
  };
  metrics: HealthMetrics;
}
```

## Development Workflow

### Local Development Setup

```bash
# Development environment
make dev-setup
make dev-start
make dev-test
make dev-stop

# Database migrations
make db-migrate
make db-seed
make db-reset

# Integration testing
make test-integration
make test-e2e
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Go
        uses: actions/setup-go@v3
        with:
          go-version: "1.21"
      - name: Run tests
        run: make test
      - name: Run integration tests
        run: make test-integration

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker images
        run: make docker-build
      - name: Push to registry
        run: make docker-push

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to staging
        run: make deploy-staging
      - name: Run E2E tests
        run: make test-e2e
      - name: Deploy to production
        run: make deploy-production
```

This technical architecture provides the foundation for building the Integrated SDLC Intelligence Platform while maintaining compatibility with the existing Sruja codebase and enabling incremental development.
