# Sruja Architecture Agent – Detailed Process

Full process, file patterns, DSL templates, detection guides, and examples. Load this when executing architecture discovery.

**Selective extraction and concise summary:** When the user asks to **extract** or **focus on** a specific area (e.g. one subpath, "just services/auth"), follow **SKILL.md** § "Interactive and selective capture" and "Concise extraction summary": run `sruja discover --context -r .` (or `-r <subpath>`), show suggested areas, let the user pick, then output only the concise summary (Area, Entry points, Main components, Outbound, Tech, Open questions). No full DSL unless requested; no long prose or full scan JSON.

### Step 2: Collect Information

Use your tools to gather information. **Follow the discovery playbook order** so entry points and dependency context drive accuracy (research: ArchAgent, static-analysis combination; see [ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md](../../docs/ARCHITECTURE_DISCOVERY_RESEARCH_AND_PRACTICES.md)).

#### 2.0 Discovery playbook (ordered phases)

Do these in order; do not read the entire codebase.

| Phase | What to find | Where to look | Map to DSL |
|-------|--------------|---------------|------------|
| **1. Deployables / runtime** | How many runnable units? Technologies? | Dockerfile(s), docker-compose*.yml, compose.yaml, kubernetes/**/*.yaml, Procfile, fly.toml, vercel.json, .github/workflows | Each image/service/pod → system or container; set `technology` from base image or manifest. |
| **2. Entry points** | Main process entry files | package.json scripts, pyproject.toml, Cargo.toml, go.mod, pom.xml; then index.js, main.py, main.go, *Application.java, lib/express.js | One runnable entry = one container; internal modules/classes = components inside that container. |
| **3. Data stores & queues** | DBs, caches, message queues | config/**/*, .env.example, connection strings, ORM imports (pg, mongoose, redis, kafkajs, amqplib) | database or queue container; relationships "SQL", "Redis protocol", "AMQP - publishes/consumes". |
| **4. Service-to-service & externals** | HTTP/gRPC clients, SDKs, env URLs | Imports (axios, fetch, grpc), env vars (SERVICE_X_URL), docker-compose service names | Relationships with labels: "REST - auth", "gRPC - orders", "HTTPS - payment". |
| **5. UI / frontend** | SPA or server-rendered app | Next.js/Vue/React config, pages/, app/, frontend/ | Container with technology "React", "Next.js", etc.; relationship to API/BFF. |

After phase 1 you know how many containers/systems to create. After 2 you assign entry points to those containers and add internal components. After 3–4 you add databases and relationships. After 5 you add the frontend container and its link to the API.

#### 2.1 Clone Repositories

```bash
# Clone the repository to a temporary location
git clone <repo-url> /tmp/architecture-analysis

# If multiple repos, clone each one
git clone <repo-url-1> /tmp/architecture-analysis/service-a
git clone <repo-url-2> /tmp/architecture-analysis/service-b
```

#### 2.2 What to read first (read order)

**Do not read the entire codebase.** Infer structure from entry points and dependencies. Priority:

1. **README + package/manifest** – Stack, scripts, project overview.
2. **Entry point(s)** – e.g. `index.js`, `main.py`, `Application.java`.
3. **One level of imports or route registration** – Enough to see boundaries.
4. **Config for DB/queues/external APIs** – Connection strings, env vars.

#### 2.3 Find Key Files

**Look for these file patterns:**

**Package/Dependency Files:**
- `package.json` - Node.js dependencies and scripts
- `requirements.txt`, `pyproject.toml` - Python dependencies
- `go.mod` - Go modules
- `Cargo.toml` - Rust dependencies
- `pom.xml`, `build.gradle` - Java dependencies
- `Gemfile` - Ruby dependencies

**Infrastructure Files:**
- `docker-compose.yml` - Service definitions
- `Dockerfile` - Container configuration
- `kubernetes/**/*.yaml` - K8s manifests
- `terraform/**/*.tf` - Infrastructure as code
- `.github/workflows/*.yml` - CI/CD pipelines

**Documentation Files:**
- `README.md` - Project overview
- `docs/architecture.md` - Architecture documentation
- `docs/deployment.md` - Deployment guides
- `docs/api.md` - API documentation
- `ADRs/**/*.md` - Architecture Decision Records

**Configuration Files:**
- `.env.example` - Environment variables
- `config/**/*` - Configuration files
- `src/config/**/*` - Application config

**Source Code:**
- Entry points: `index.js`, `main.py`, `main.go`, `app.js`
- Routes/APIs: `routes/**/*`, `controllers/**/*`, `api/**/*`
- Models: `models/**/*`, `entities/**/*`
- Services: `services/**/*`, `workers/**/*`

#### 2.4 Analyze Code Structure

**Read key files to understand:**

1. **Technology Stack:**
   - Languages and versions
   - Frameworks (Express, Django, FastAPI, etc.)
   - Libraries and their purpose

2. **Databases & Storage:**
   - Database connections (PostgreSQL, MongoDB, Redis, etc.)
   - Connection strings in config
   - ORM usage (Sequelize, TypeORM, SQLAlchemy, etc.)

3. **External Dependencies:**
   - API clients (Stripe, AWS SDK, Twilio, etc.)
   - Environment variables pointing to external services
   - HTTP client calls to external APIs

4. **Service Communication:**
   - REST API routes
   - GraphQL schemas
   - gRPC definitions
   - Message queue usage (RabbitMQ, Kafka, etc.)

5. **Architecture Patterns:**
   - Monolith vs microservices
   - Event-driven patterns
   - API gateway pattern
   - CQRS, event sourcing

6. **Internal Abstractions (for DEEP scope):**
   
   **Wrapper/Decorator/Interceptor Patterns:**
   - Look for classes/functions that wrap or intercept other components
   - Generic searches:
     - `class.*Wrapper`, `class.*Decorator`, `class.*Proxy`, `class.*Interceptor`
     - `class.*Layer`, `class.*Chain`, `class.*Pipeline`
     - `decorate`, `wrap`, `intercept`, `middleware`
   - Language examples:
     - Node.js: middleware functions with `(req, res, next)` signature
     - Python: decorators (`@wrapper`), middleware classes
     - Java: Filter, Interceptor, AOP classes
     - Go: middleware functions wrapping `http.Handler`
   
   **Error Handling Paths:**
   - Trace how errors flow from entry point to handlers
   - Generic searches:
     - `catch`, `except`, `try`, `throw`, `raise`
     - `on('error')`, `onError`, `handleError`, `errorHandler`
     - Files: `error.*`, `exception.*`, `handler.*`
   - Language examples:
     - Node.js: error middleware `(err, req, res, next)`, `finalhandler`
     - Python: `try/except`, error handlers in Flask/FastAPI
     - Java: `@ExceptionHandler`, `try/catch`, `ErrorHandler`
     - Go: error returns, recovery middleware
   
   **Composition/Plugin/Mount Patterns:**
   - Find where sub-systems attach to parent systems
   - Generic searches:
     - `register`, `attach`, `mount`, `plugin`, `extend`
     - `parent`, `child`, `sub.*`, `compose`
     - `use`, `add`, `install`, `load`
   - Language examples:
     - Node.js: `app.use()`, `app.mount()`, plugins
     - Python: `app.include_router()`, Flask blueprints
     - Java: `@Import`, `@ComponentScan`, plugin systems
     - Go: middleware chaining, sub-routers
   
   **Environment-Specific/Conditional Behavior:**
   - Look for conditional logic based on environment
   - Generic searches:
     - `ENV`, `environment`, `env`, `config`
     - `production`, `staging`, `development`, `test`
     - `debug`, `verbose`, `logging.*level`
     - Feature flags: `feature.*flag`, `enable.*`, `disable.*`
   - Common patterns:
     - Caching enabled in production
     - Debug logging in development
     - Mock services in test
     - Feature toggles
   
   **Generic search commands:**
   ```bash
   # Wrapper/decorator patterns (adjust for language)
   grep -riE "class.*(Wrapper|Decorator|Layer|Interceptor|Proxy)" src/
   grep -riE "(decorate|wrap|middleware|intercept)" src/
   
   # Error handling (language-agnostic)
   grep -riE "(catch|except|error|exception|handle.*error)" src/
   
   # Composition/mount patterns
   grep -riE "(mount|attach|register|plugin|parent|compose)" src/
   
   # Environment-specific behavior
   grep -riE "(ENV|environment|production|staging|debug)" src/
   grep -riE "(cache.*enable|feature.*flag)" src/
   ```

### Step 3: Generate Sruja DSL

**Canonical form (required for parser):** Use **flat** top-level declarations only. Do **not** wrap content in `architecture "Name" { ... }` — the parser does not support that block. Declare kinds at the top (e.g. `person = kind "Person"`, `system = kind "System"`, `container = kind "Container"`, `database = kind "Database"`) or use `import { * } from 'sruja.ai/stdlib'`. Use assignment `Id = kind "Label" { ... }`, `database` for data stores, relationships `SourceId -> TargetId "label"`. Every element needs `description`; every container needs `technology`.

**C4 levels (map correctly):** **System** = software system or deployable service boundary. **Container** = runnable/deployable unit (process, web app, API server, worker, database, queue)—something that runs. **Component** = logical grouping *inside* a container (module, controller, service class, middleware, repository)—not a separate process. Do not use container for in-process modules or component for an entire API server.

### Framework vs application repos: scope presets

Use different defaults for **framework/libraries** vs **full applications** so depth is consistent:

| Repo type | Examples | Recommended scope | Containers | Components | Externals | Notes |
|----------|----------|-------------------|-----------|------------|----------|-------|
| **Framework / library** | express, fastapi, django, next.js | Standard | 1–2 (runtime + CLI/build) | 10–20 | 2–7 | Model the framework as one system; consumers as external systems (Node.js app, Python app, etc.). Components = main modules/classes (Application, Router, Request, Response, etc.). |
| **Full application / product** | saleor, gitea, ever-gauzy | Standard/Deep | 3–8 (API, workers, frontend, DBs, queues, etc.) | 15–30 (standard), 30–50 (deep) | 3–10 | Model user-facing systems (web, API, workers) plus data stores and key externals (payments, auth, messaging). Components capture internal layers only where they clarify flows. |

When the repo is clearly a **framework**, default to the first row; when it is a **product/app**, default to the second. If ambiguous, ask a scope/boundaries question before generating.

#### 3.1 Minimal valid template

Use **flat** syntax with **kinds declared** (parser requirement). See `book/valid-examples/getting-started.sruja`.

```sruja
// Smallest valid architecture (passes sruja lint). Flat form; no architecture "Name" { } wrapper.
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" { description "End user" }
App = system "My App" {
  description "Main application"
  Web = container "Web" { technology "React"; description "UI" }
  Api = container "API" { technology "Node.js"; description "REST API" }
  Web -> Api "HTTPS"
}
User -> App "uses"
```

#### 3.2 System definition (canonical)

```sruja
MySystem = system "Service Name" {
  description "What this service does - be specific"
  Api = container "API" {
    technology "Node.js"
    description "What this component does"
  }
  DB = database "Database Name" {
    technology "PostgreSQL"
    description "What data it stores"
  }
  Api -> DB "SQL - reads and writes"
}
```

#### 3.3 External systems

Model external services as **systems** with description:

```sruja
Stripe = system "Stripe" {
  description "External payment gateway"
  PaymentApi = container "Payment API" {
    technology "REST"
    description "Charges and refunds"
  }
}
MyApi -> Stripe.PaymentApi "HTTPS - payment processing"
```

#### 3.4 End users

```sruja
EndUser = person "End User" {
  description "Customer using the platform"
}
EndUser -> MySystem.Api "HTTPS - browse, login"
```

#### 3.5 Relationship labels

- **Good:** `"HTTPS - auth"`, `"gRPC - order validation"`, `"reads from"`, `"writes to"`, `"publishes events to"`, `"invokes"`.
- **Bad:** `"uses"`, `"calls"` (too vague unless combined with protocol).

#### 3.6 Multi-service (single file)

For multiple systems in one file, define each with assignment form and use `SystemId.ContainerId` in relationships:

```sruja
UserSvc = system "User Service" {
  description "User management and authentication"
  Api = container "REST API" { technology "Node.js"; description "User API" }
  DB = database "User Database" { technology "PostgreSQL"; description "User data" }
  Api -> DB "SQL"
}
OrderSvc = system "Order Service" {
  description "Order processing"
  Api = container "Order API" { technology "Python"; description "Order API" }
  DB = database "Order DB" { technology "MongoDB"; description "Orders" }
  Api -> DB "MongoDB protocol"
}
OrderSvc.Api -> UserSvc.Api "REST - validate user"
```

#### 3.7 Modeling Internal Patterns (for DEEP scope)

When using **deep scope**, capture internal abstractions as components. These patterns apply across languages/frameworks:

**Wrapper/Interceptor/Chain Pattern:**

Generic structure - applies to middleware, filters, interceptors, decorators:
```sruja
pipeline = container "Request Pipeline" {
  technology "<framework>"
  description "Sequential processing chain"
  
  interceptor = component "Interceptor" {
    description "Wraps each handler with pre/post processing"
  }
  
  chain = component "Chain" {
    description "Ordered list of interceptors"
  }
  
  interceptor -> chain "registered in"
}
```

Examples by framework:
- Express.js: `middleware` container with `layer` and `stack` components
- Django: `middleware` container with `MiddlewareMixin` classes
- Spring: `filterChain` container with `Filter` components
- FastAPI: `middleware` container with `Middleware` components

**Error Handling Pattern:**

Generic error flow:
```sruja
handler = container "Request Handler" {
  technology "<framework>"
  
  errorHandler = component "Error Handler" {
    description "Catches unhandled errors and returns appropriate response"
  }
}

app -> handler.errorHandler "passes unhandled errors to"
handler.errorHandler -> logger "logs errors via"
```

Examples by framework:
- Express.js: `finalhandler` package, error middleware `(err, req, res, next)`
- Django: `MIDDLEWARE` with exception handling, custom error views
- Spring: `@ExceptionHandler`, `@ControllerAdvice`, `ErrorController`
- FastAPI: `@app.exception_handler()`, exception classes

**Composition/Module/Plugin Pattern:**

Generic modular structure:
```sruja
app = system "Application" {
  description "Main application"
  
  mainModule = container "Main Module" {
    technology "<framework>"
    description "Primary application entry point"
  }
  
  subModule = container "Sub-Module" {
    technology "<framework>"
    description "Mounted/registered submodule"
  }
  
  mainModule -> subModule "registers at path"
  subModule -> app "inherits configuration from"
}
```

Examples by framework:
- Express.js: `app.use(subapp)` for mounting sub-applications
- Django: `include()` for URL patterns, apps in `INSTALLED_APPS`
- Flask: Blueprints with `app.register_blueprint()`
- Spring: `@Import`, `@ComponentScan`, modules
- FastAPI: `app.include_router()` for sub-routers

**Environment-Specific/Conditional Behavior:**

Generic conditional configuration:
```sruja
service = container "Service" {
  technology "<framework>"
  
  cache = component "Cache" {
    description "Caches data. Enabled when env=production"
  }
  
  debugLogger = component "Debug Logger" {
    description "Verbose logging. Enabled when env=development"
  }
}

cache -> config "reads environment from"
```

Examples:
- Node.js: `NODE_ENV` environment variable
- Python: `ENVIRONMENT`, `DEBUG` settings in Django/Flask
- Java: Spring profiles (`@Profile("production")`)
- Go: Environment variables via `os.Getenv()`

**ADR for internal patterns:**
```sruja
ADR_Pattern = adr "Pattern name" {
  status "Accepted"
  context "Why this pattern is needed"
  decision "What pattern was chosen and why"
  consequences "Trade-offs and implications"
}
```

### Step 4: Validate (mandatory) — fix until lint passes

**Loop: run lint → if errors, apply fixes → re-run lint. Repeat until pass. Do not present until pass.**

1. Run `sruja lint architecture.sruja`.
2. If there are errors, apply the fix from the table below.
3. Re-run `sruja lint`.
4. Repeat until lint passes. Do not present a file that fails lint.

```bash
sruja lint architecture.sruja
```

#### Lint error → fix

Use `sruja lint --format json` to get machine-readable diagnostics with `code`; map code to fix below.

| Code | Symptom | Fix |
|------|---------|-----|
| E101, E102, E103, E104 | Parse/syntax error | Fix the indicated line: check braces, strings, tokens. Ensure flat syntax; no `architecture "Name" { }` wrapper. |
| E201 | Duplicate identifier | Use a unique ID for each element; rename or remove the duplicate. |
| E202 | Undefined reference | Define the referenced ID before use, or fix typo in relationship (source or target). |
| E203 | Invalid relationship | Fix relationship endpoints or labels per DSL rules. |
| E204 | Circular dependency | Break the cycle: remove one relationship in the cycle (e.g. remove `A -> B` or `B -> A`). Re-run lint. |
| E205 | Orphan component | Add at least one relationship `X -> Orphan "..."` or `Orphan -> Y "..."`, or remove the element. |
| E206 | Layer violation | Fix dependency direction so higher layers do not depend on lower (e.g. service must not depend on web). Remove or reverse the violating edge. |
| E301 | Invalid property | Correct the property value (e.g. valid enum, number range). |
| E302 | Missing field | Add the required field (e.g. description, technology on container). |
| E303 | Validation rule failed | Follow the rule message; common: add description, technology, or fix structure. |
| E401 | Policy violation | Satisfy the policy (e.g. governance, scenario) or adjust the declaration. |
| W001 | Best practice | Improve documentation or structure as suggested (optional but recommended). |
| (no code) | Missing description | Add `description "..."` to the element. |
| (no code) | Missing technology (container) | Add `technology "..."` to the container. |

#### Example: fixing a circular dependency

If lint reports: `E204 circular dependency between [NodeHTTPServer, Application]`:

- Open the `.sruja` file and remove one edge in that cycle. For example, delete the line `NodeHTTPServer -> Application "request"` (or the reverse, depending on which direction is redundant).
- Save and run `sruja lint` again. Repeat until pass.

### Step 5: Present and Iterate

Present the architecture to the user:

```
I've analyzed your codebase and generated the architecture:

**Services detected:**
- User Service (Node.js, PostgreSQL)
- Order Service (Python, MongoDB)
- Payment Service (Go, Redis)

**External dependencies:**
- Stripe (payment processing)
- AWS S3 (file storage)

**End users:**
- E-commerce shoppers

**End-to-end flow:**
User → User Service → Order Service → Payment Service → Stripe

[Show generated .sruja file]

✓ Architecture validated successfully

**Questions for refinement:**
1. Did I identify all services correctly?
2. Are there any external services I missed?
3. Would you like to add deployment patterns?
4. Any specific flows you want documented?
```

## Scope ladder

| Scope | Systems | Components | Use when |
|-------|---------|------------|----------|
| **Minimal** | 1 | 3–7 containers | Quick sketch, entry points and main deps only. |
| **Standard (recommended)** | 1–2 | 10–30 | All key relationships and technologies. |
| **Deep** | Multiple | 30–50 | Internal components, key external systems. |

Default to **Standard** unless the user asks for minimal or deep.

## Per-language / framework hints

Use these to infer **entry points**, **routes/services/data access**, and **technology** strings accurately.

| Stack | Entry points | Routes / services / data | Technology string |
|-------|--------------|---------------------------|--------------------|
| **Express** | `index.js`, `app.js`, `lib/express.js` | `routes/*.js`, `app.get/post`, `services/` | "Node.js", "Express" |
| **FastAPI** | `main.py`, `app.py` | `@app.get/post`, `routers/`, `services/` | "Python", "FastAPI" |
| **Django** | `manage.py`, `*/wsgi.py`, `urls.py` | `views.py`, `urls.py`, `models.py` | "Python", "Django" |
| **Spring Boot** | `*Application.java`, `src/main/java` | `@RestController`, `*Controller.java`, `*Repository` | "Java", "Spring Boot" |
| **Next.js** | `pages/`, `app/`, `next.config.*` | `pages/api/`, `app/api/` | "Node.js", "Next.js" |
| **Go Gin** | `main.go`, `cmd/*/main.go` | `router.GET/POST()`, `handlers/`, `*Handler` | "Go", "Gin" |
| **NestJS** | `main.ts`, `src/main.ts` | `@Controller()`, `@Injectable()`, `modules/` | "Node.js", "NestJS" |
| **Rails** | `config.ru`, `bin/rails`, `app/` | `routes.rb`, `controllers/`, `models/` | "Ruby", "Rails" |
| **Flask** | `app.py`, `wsgi.py` | `@app.route`, `blueprints/` | "Python", "Flask" |
| **Rust (Actix/Axum)** | `main.rs`, `src/main.rs` | route macros, handlers | "Rust", "Actix" / "Axum" |

Do not read the entire codebase; use entry points and one level of imports to infer structure.

### Deployable and runtime detection (playbook Phase 1)

Use these to map **runnable units** to systems/containers before drilling into code.

| Artifact | What it implies | Technology from |
|----------|------------------|-----------------|
| **Dockerfile** | One container per Dockerfile (or multi-stage single image) | Base image (node, python, openjdk, golang) + package manifest |
| **docker-compose.yml / compose.yaml** | One service = one container; names = candidate system/container labels | service image or build context + package.json/pyproject.toml |
| **kubernetes/** *.yaml | Deployments/StatefulSets = runnable units | container image; CronJobs = scheduled job container |
| **Procfile** | One process type = one container (web, worker, etc.) | Process command (node, python, bundle exec) |
| **fly.toml**, **vercel.json**, **netlify.toml** | Single deployable (serverless or PaaS app) | Config build command / runtime |
| **.github/workflows** | CI only; use to infer deployables from build/job names if no Docker/K8s | Build matrix or job names |

When multiple deployables exist (e.g. docker-compose with api, worker, frontend), create one container per deployable and set `technology` from the corresponding manifest or image.

## Detection Guide

### Technologies to Detect

**Languages:**
- JavaScript/TypeScript: `package.json`, `.js`, `.ts` files
- Python: `requirements.txt`, `.py` files, `setup.py`
- Go: `go.mod`, `.go` files
- Java: `pom.xml`, `build.gradle`, `.java` files
- Rust: `Cargo.toml`, `.rs` files
- Ruby: `Gemfile`, `.rb` files
- C#: `.csproj`, `.cs` files

**Frameworks:**
- Node.js: Express, NestJS, Fastify, Next.js
- Python: Django, FastAPI, Flask, Tornado
- Go: Gin, Echo, Fiber
- Java: Spring Boot, Quarkus
- Ruby: Rails, Sinatra

**Databases:**
- PostgreSQL: `pg`, `psycopg2`, connection strings with `postgres://`
- MongoDB: `mongoose`, `pymongo`, `mongodb://`
- MySQL: `mysql2`, `pymysql`, `mysql://`
- Redis: `redis`, `ioredis`, `redis://`
- Elasticsearch: `@elastic/elasticsearch`, `elasticsearch`

**Message Queues:**
- RabbitMQ: `amqplib`, `pika`, `amqp://`
- Kafka: `kafkajs`, `confluent-kafka`
- Redis Pub/Sub: redis client pub/sub methods
- AWS SQS/SNS: `aws-sdk` SQS/SNS usage

**External Services:**
- Stripe: `stripe` package, `api.stripe.com`
- AWS: `aws-sdk`, `boto3`
- Twilio: `twilio` package
- SendGrid: `@sendgrid/mail`
- Google Cloud: `@google-cloud/*` packages

### Analysis Patterns

**Finding REST APIs:**
```javascript
// Express.js
app.get('/users/:id', handler)
router.post('/users', handler)

// FastAPI (Python)
@app.get("/users/{user_id}")
@app.post("/users")

// Go Gin
router.GET("/users/:id", handler)
router.POST("/users", handler)
```

**Finding Databases:**
```javascript
// Connection strings
postgres://user:pass@host:5432/db
mongodb://user:pass@host:27017/db
redis://host:6379/0

// ORM usage
new Sequelize('postgres://...')
mongoose.connect('mongodb://...')
```

**Finding External Services:**
```javascript
// Stripe
const stripe = require('stripe')(key)
stripe.charges.create(...)

// AWS
const s3 = new AWS.S3()
s3.upload(...)

// HTTP clients
axios.post('https://api.example.com/...')
fetch('https://external-service.com/api')
```

**Finding Message Queues:**
```javascript
// RabbitMQ
channel.assertQueue('orders')
channel.consume('orders', handler)

// Kafka
producer.send({ topic: 'orders', messages })
consumer.subscribe({ topic: 'orders' })
```

### Documentation Analysis

**Extract from README.md:**
- Project description
- Technology stack
- Architecture overview
- Deployment information

**Extract from docs/deployment.md:**
- Deployment patterns (self-hosted, cloud, hybrid)
- Infrastructure requirements
- Configuration options

**Extract from docs/integration.md:**
- Client integration patterns
- API usage examples
- Webhook endpoints

**Extract from user personas:**
- End user segments
- User behaviors
- Access patterns

## Import from Specs

Model imported APIs as **systems** with assignment form. Use `database` for data stores.

### OpenAPI Import

```sruja
ImportedApi = system "Service Name" {
  description "Imported from OpenAPI spec"
  Api = container "API Name" {
    technology "REST"
    description "From info.description; key endpoints from paths"
  }
}
MyApi -> ImportedApi.Api "HTTPS - [purpose]"
```

### GraphQL Import

```sruja
GraphQLService = system "Service Name" {
  description "Imported from GraphQL schema"
  Gql = container "GraphQL API" {
    technology "GraphQL"
    description "Queries and mutations from schema"
  }
}
```

### AsyncAPI Import

```sruja
EventSystem = system "Event System" {
  description "Imported from AsyncAPI spec"
  Stream = container "Event Stream" {
    technology "Kafka"
    description "Topics and channels from spec"
  }
}
Producer -> EventSystem.Stream "publishes events to"
```

## Advanced Features

### End-to-End Flow Tracing

Trace complete flows from user to backend:

```sruja
// Generate a view showing complete flow
view "Order Processing Flow" {
  includes [
    "end_user",
    "web_app.frontend",
    "order_service.api",
    "payment_service.api",
    "stripe.payment_api"
  ]
  
  description "Complete flow from user placing order to payment processing"
}
```

### Gap Detection

Identify missing information:

```
**Architecture Completeness Analysis:**

✓ Complete:
- Service definitions
- Technology stack
- Database relationships

⚠ Partial:
- External services (API endpoints detected, but full specs missing)
- Deployment patterns (mentioned in README, but not detailed)

✗ Missing:
- End user segments
- Performance requirements
- SLA definitions

**Recommendations:**
1. Define end user personas
2. Document deployment patterns
3. Add external service SLAs
4. Specify performance constraints
```

### Architecture Improvements

Suggest improvements based on analysis:

```
**Architecture Observations:**

1. **Potential Bottleneck:** Order service is used by 5 other services
   - Consider: Caching, rate limiting, or decomposition

2. **Missing Redundancy:** Single database for user service
   - Consider: Read replicas or multi-region setup

3. **External Dependency:** Critical path depends on Stripe
   - Consider: Circuit breaker, fallback mechanisms

4. **Security:** No authentication layer detected
   - Consider: API gateway with auth, JWT tokens
```

## Examples

### Example 1: Simple Web Service

**User request:** "Analyze my Node.js API"

**Analysis:**
1. Clone repository
2. Read `package.json` - Express, PostgreSQL client
3. Read `docker-compose.yml` - PostgreSQL service
4. Read `src/index.js` - Express server on port 3000
5. Read `src/routes/users.js` - GET /users, POST /users

**Generated (canonical form):**
```sruja
UserApi = system "User API" {
  description "REST API for user management"
  Api = container "REST API" {
    technology "Node.js"
    description "Express.js REST API for user operations; GET/POST /users"
  }
  DB = database "User Database" {
    technology "PostgreSQL"
    description "Primary data store for user information"
  }
  Api -> DB "SQL - user queries and updates"
}
ApiUser = person "API User" {
  description "Client applications consuming the API"
}
ApiUser -> UserApi.Api "HTTPS - REST API calls"
```

### Example 2: Microservices Platform

**User request:** "Analyze my e-commerce platform with 3 services"

**Analysis:**
1. Clone user-service, order-service, payment-service
2. Analyze each service's tech stack
3. Detect cross-service communication
4. Identify shared external dependencies

**Generated:**

**user-service.sruja (canonical):**
```sruja
UserSvc = system "User Service" {
  description "User management and authentication"
  Api = container "User API" {
    technology "Node.js"
    description "REST API for user operations"
  }
  DB = database "User Database" {
    technology "PostgreSQL"
    description "User profiles and credentials"
  }
  Cache = database "Session Cache" {
    technology "Redis"
    description "Active user sessions"
  }
  Api -> DB "SQL"
  Api -> Cache "Redis protocol"
}
```

**order-service.sruja (canonical):**
```sruja
OrderSvc = system "Order Service" {
  description "Order processing and management"
  Api = container "Order API" {
    technology "Python"
    description "REST API for order operations"
  }
  Worker = container "Order Worker" {
    technology "Python"
    description "Background order processing"
  }
  DB = database "Order Database" {
    technology "MongoDB"
    description "Order documents and history"
  }
  Queue = database "Message Queue" {
    technology "RabbitMQ"
    description "Order event stream"
  }
  Api -> DB "MongoDB protocol"
  Worker -> DB "MongoDB protocol"
  Api -> Queue "AMQP - publishes order events"
  Queue -> Worker "AMQP - consumes order events"
}
```

**architecture.sruja (single-file multi-system, canonical):**
```sruja
UserSvc = system "User Service" { ... }
OrderSvc = system "Order Service" { ... }
PaymentSvc = system "Payment Service" { ... }

OrderSvc.Api -> UserSvc.Api "REST - validate user"
OrderSvc.Api -> PaymentSvc.Api "gRPC - process payment"

Stripe = system "Stripe" {
  description "External payment gateway"
  PaymentApi = container "Payment API" {
    technology "REST"
    description "Charge creation, refund processing"
  }
}
PaymentSvc.Api -> Stripe.PaymentApi "HTTPS - payment processing"

Shopper = person "Shopper" {
  description "E-commerce customer browsing and purchasing"
}
Shopper -> UserSvc.Api "HTTPS - account management"
Shopper -> OrderSvc.Api "HTTPS - place orders"
```

### Example 3: Documentation-Based Analysis

**User request:** "Analyze architecture from docs, no code access"

**Analysis:**
1. Read `README.md` - Project overview
2. Read `docs/architecture.md` - System design
3. Read `docs/deployment.md` - Infrastructure
4. Read `docs/api.md` - API documentation

**Generated (canonical):**
```sruja
ContentPlatform = system "Content Platform" {
  description "Content management and delivery platform (from docs)"
  Web = container "Web Application" {
    technology "React"
    description "User-facing web application (from docs)"
  }
  Api = container "API Server" {
    technology "Node.js"
    description "Backend API for content management (from docs)"
  }
  DB = database "Content Database" {
    technology "PostgreSQL"
    description "Content storage (from deployment docs)"
  }
  Cache = database "CDN Cache" {
    technology "CloudFront"
    description "Content delivery network (from deployment docs)"
  }
  Web -> Api "HTTPS - API calls"
  Api -> DB "SQL"
  Api -> Cache "Invalidates cache on content update"
}
Auth0 = system "Auth0" {
  description "External authentication service (from architecture docs)"
  AuthApi = container "Auth API" {
    technology "OAuth 2.0"
    description "User authentication"
  }
}
ContentPlatform.Web -> Auth0.AuthApi "OAuth - user authentication"
ContentEditor = person "Content Editor" {
  description "Content creators and editors (from user personas doc)"
}
ContentReader = person "Content Reader" {
  description "End users consuming content (from user personas doc)"
}
ContentEditor -> ContentPlatform.Web "HTTPS - content management"
ContentReader -> ContentPlatform.Web "HTTPS - read content"
ContentReader -> ContentPlatform.Cache "HTTPS - cached content delivery"
```

## Best Practices

### 1. Start Simple, Then Deepen

- Begin with high-level overview
- Ask if user wants more detail
- Don't overwhelm with too much information

### 2. Validate Assumptions

When uncertain:
- State your assumption
- Provide confidence level
- Ask for confirmation

Example:
```
"I detected PostgreSQL based on the pg package in dependencies. 
Is this the primary database, or do you also use MongoDB for analytics?"
```

### 3. Focus on What Matters

- External dependencies over internal utilities
- Service boundaries over individual functions
- Data flows over code structure
- End-to-end paths over isolated components

### 4. Use Confidence Levels

Label uncertain detections in the description:
```sruja
Analytics = system "Analytics Service" {
  description "Detected from code (confidence: medium) - HTTP client calls to analytics.example.com; needs confirmation"
  Api = container "Analytics API" {
    technology "REST"
    description "Inferred from client usage"
  }
}
```

### 4.1 Ask questions instead of guessing (preferred)

When evidence is insufficient, **ask questions** rather than inventing architecture.

- Prefer questions for: system boundaries, what is deployable, which external services matter, and which “areas” to model first.
- If you must proceed non-interactively, keep scope conservative and clearly mark uncertainty in descriptions (confidence: low/medium) and list “Open questions”.

### 5. Document Gaps

Always highlight what's missing:
```
**Note:** I couldn't detect:
- End user segments (no user documentation found)
- Deployment patterns (no deployment docs)
- SLA requirements (not documented)

Would you like to add these manually?
```

### 6. Iterate Collaboratively

Don't try to be perfect on first try:
1. Generate initial architecture
2. Ask for feedback
3. Refine based on input
4. Repeat until satisfied

## Discovery interview: ask questions to capture better

Use the LLM to **ask the user intelligent questions** before or during discovery. This yields better scope, names, and boundaries than code-only inference.

### Deriving the right questions from repo context

After gathering evidence (`sruja discover --context -r .` and the phased playbook), map **repo signals** to **question categories**. Ask only what is still ambiguous; do not ask when evidence is sufficient.

| Repo signal (what you observed) | What's ambiguous | Question category | Example question (adapt) |
|----------------------------------|------------------|-------------------|---------------------------|
| Multiple top-level dirs (e.g. `services/`, `apps/`, `packages/`) | Which area to capture first; one vs many systems | Scope / area | "Should we capture one area first (e.g. `services/auth`) or the whole repo? I can do one subpath at a time and stitch." |
| Monorepo or many deployables (docker-compose with 5+ services) | System boundaries; deployable vs library | Boundaries | "Is this one system or several? Which directories are separate deployables?" |
| Env vars or SDK usage (e.g. `STRIPE_KEY`, `SERVICE_X_URL`, `axios` to unknown host) | Identity of external systems; which to show | Externals | "I see SERVICE_X_URL / Stripe client. Which external systems must appear on the diagram?" |
| Multiple entry points (e.g. `main.go`, `cmd/worker/main.go`, `cmd/api/main.go`) | Main user-facing entry; key flows | Entry / flows | "What's the main user-facing entry (web app, API, CLI)? Any key flows (e.g. checkout) to make explicit?" |
| No Docker/K8s but many inferred services | What actually runs in production | Boundaries + Externals | "I don't see deployment manifests. Which of these (list) are real deployables? Any external services?" |
| README or docs mention requirements/ADRs | Whether to encode into DSL; corrections | Intent | "I found candidate requirements in README/docs; should I encode them into the DSL? Any corrections?" |

**Workflow:** (1) Run discovery and playbook. (2) From the table, pick 2–5 questions that match what you saw and what's still unclear. (3) Ask the user; use answers to set scope, subpath, names, externals, and intent. (4) Only then generate the full architecture. Do not guess missing answers.

### Choose two-step vs one-go from context

After running `sruja discover --context -r .`: if the repo is **small and obvious** (e.g. &lt;15 components, one main dir, single framework), **skip questions** and generate in one go; if **large or ambiguous** (many components, multiple areas like services/ and apps/), use **two-step** (derive questions, then generate). When unclear, prefer two-step.

### Divide analysis into multiple parts

When scope is **too big for one pass** (e.g. 50+ components, many services, or one very large area), **split the analysis**: analyze by subpath (`sruja discover --context -r services/auth`, then `services/orders`), by bounded context, or by depth (high-level first, then expand one big container). Produce one fragment or section per part; use external systems for cross-refs; stitch or document the split (see incremental capture docs).

### When to ask

- **Before** diving into code when the request is vague ("document our architecture") or the repo is large.
- **During** when you see multiple possible boundaries or external systems and need to choose.
- **After** the first draft as refinement questions (see Output Format below).

### Question bank (use 2–5 per session; adapt to context)

**Context / shape**
- "Is this a single service, a monolith with modules, or several microservices?"
- "Should we capture one area first or the whole repo?"

**Large repo**
- "The repo is big. Should we focus on a specific area (e.g. `services/auth`, `apps/web`) or the whole codebase? I can capture by subpath and we can stitch later."
- "Which directory or service should we start with?"

**Scope**
- "Do you want a minimal sketch (entry points + main deps), standard (10–30 components), or a deeper model (internal layers, error paths)?"

**Boundaries**
- "What are your main bounded contexts or team-owned areas?"
- "Any external systems (payments, auth, notifications) that must appear in the diagram?"

**Entry points and flows**
- "What's the main user-facing entry (web app, public API, CLI)?"
- "Any key flows (e.g. checkout, auth) I should make explicit?"

**Refinement (after first draft)**
- "Does this match how you think about the system? Any services or boundaries missing?"
- "Prefer different names for systems or containers?"

### How to use answers

- **Scope** → Choose minimal / standard / deep; set target component count.
- **Large repo / area** → Run quickstart with `-r <subpath>`; generate one fragment per area; mention stitch later.
- **Boundaries / externals** → Include those systems and relationships in the DSL; name them as the user said.
- **Entry points / flows** → Ensure those paths are visible (person → system → containers) and relationships are labeled.

## Extraction quality checklist (static + LLM)

Use this to improve both **developer experience** and **quality of architecture extraction** (static and LLM-assisted):

| Step | Command / action | Quality check |
|------|------------------|----------------|
| 1. Context | `sruja discover --context -r .` (or `--format json` for agents) | Repo context, suggested areas, framework, and component count inform scope and naming. |
| 2. Generate | Produce DSL in **canonical form** (flat, kinds declared; no `architecture "Name" { }`). | Matches parser; fewer parse errors. |
| 3. Lint | `sruja lint architecture.sruja` or `sruja lint --format json` | Get diagnostic codes (E201, E204, …). |
| 4. Fix | Use the [Lint error → fix](#lint-error--fix) table; re-run lint. | Fix until `ok: true` / no errors. |
| 5. Export | `sruja export markdown architecture.sruja` (optional) | Human-readable doc for review. |

**Static extraction:** Steps 1 and 3 use scan and lint only; no LLM required. **LLM extraction:** Steps 2 and 4 use the skill and the code→fix table so generated architecture passes validation. Doing all five steps raises both DX and extraction quality.

## Post-generate checklist (validation rules)

Before presenting architecture, self-check:

- [ ] Every `system`, `container`, `component`, `database`, `person` has `description`.
- [ ] Every `container` has `technology`.
- [ ] Every element appears in at least one relationship (no orphans).
- [ ] Relationship labels are specific (protocol and/or purpose).
- [ ] `sruja lint` passes (run it; do not present a file that fails).

## Output Format

Always present architecture in this format:

```markdown
## Architecture Analysis

### Summary
- **Services**: [number] services detected
- **Technologies**: [list main technologies]
- **External Dependencies**: [list external services]
- **End Users**: [list user types]

### Services

#### [Service Name]
- **Technology**: [language/framework]
- **Description**: [what it does]
- **Components**: [list containers/databases]

### External Dependencies
[List external services and their purpose]

### End Users
[List user types and their behaviors]

### End-to-End Flow
[Describe main user flow]

### Generated Architecture

```sruja
[Full Sruja DSL]
```

### Validation
✓ Architecture validated successfully
[Or list any validation errors]

### Questions for Refinement
Ask 1–3 from the discovery interview (e.g. "Does this match how you think about the system? Any services or boundaries missing? Prefer different names?")
```

## Remember

- **You are an expert** at understanding code and architecture
- **Use your tools** to gather information, don't guess
- **Ask questions** when uncertain
- **Validate** your output with `sruja lint`
- **Iterate** with the user to refine
- **Focus on what matters** - service boundaries, data flows, external dependencies
- **Document gaps** - it's okay to not know everything
- **Be collaborative** - architecture discovery is a dialogue

## Quick Reference

### Common Commands

```bash
# Clone repository
git clone <url> /tmp/analysis

# Read key files
read package.json
read docker-compose.yml
read README.md

# Validate architecture
sruja lint architecture.sruja

# Export to other formats
sruja export markdown architecture.sruja
sruja export mermaid architecture.sruja
```

### File priority (read order)

1. **README + package/manifest** – Stack, scripts, overview.
2. **Entry point(s)** – e.g. `index.js`, `main.py`, `Application.java`.
3. **One level of imports or route registration** – Boundaries only.
4. **Config for DB/queues/external APIs** – Env, connection strings.
Do not read entire codebase.

### Detection Confidence Levels

- **High (90%+)**: Explicitly declared in config/code
- **Medium (70-90%)**: Inferred from usage patterns
- **Low (50-70%)**: Guessed from limited information

Always state confidence level for uncertain detections.

---

*This skill empowers you to discover and document software architecture intelligently. Use your tools, ask questions, validate output, and iterate with users to create accurate and useful architecture documentation.*