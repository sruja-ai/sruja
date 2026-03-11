#!/bin/bash

# Auto-generate architecture.sruja using enhanced heuristics
# This is a helper script for demonstration purposes

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

if [ -z "$1" ]; then
  echo "Usage: $0 <repository_path> [output_file]"
  exit 1
fi

REPO_PATH="$1"
OUTPUT_FILE="${2:-architecture_auto.sruja}"

echo -e "${CYAN}${BOLD}Auto-Generating Architecture${NC}"
echo -e "${BLUE}Repository: $REPO_PATH${NC}"
echo ""

# Analyze repository
echo -e "${YELLOW}[1/5] Analyzing repository size...${NC}"
TOTAL_FILES=$(find "$REPO_PATH" -type f \( -name "*.js" -o -name "*.ts" -o -name "*.go" -o -name "*.py" -o -name "*.java" -o -name "*.rs" \) 2>/dev/null | wc -l | tr -d ' ')
echo "  Total source files: $TOTAL_FILES"

# Determine abstraction level
if [ "$TOTAL_FILES" -gt 1000 ]; then
  ABSTRACTION="HIGH"
  TARGET_COMPONENTS=20
  echo -e "  ${GREEN}Abstraction level: HIGH (>1000 files)${NC}"
elif [ "$TOTAL_FILES" -gt 100 ]; then
  ABSTRACTION="MEDIUM"
  TARGET_COMPONENTS=30
  echo -e "  ${GREEN}Abstraction level: MEDIUM (100-1000 files)${NC}"
else
  ABSTRACTION="LOW"
  TARGET_COMPONENTS=40
  echo -e "  ${GREEN}Abstraction level: LOW (<100 files)${NC}"
fi

# Detect technologies
echo ""
echo -e "${YELLOW}[2/5] Detecting technologies...${NC}"

LANGUAGES=""
FRAMEWORKS=""
DATABASES=""

# Node.js
if [ -f "$REPO_PATH/package.json" ]; then
  LANGUAGES="Node.js/JavaScript"
  echo "  ✓ Node.js detected"
  
  # Check for frameworks
  if grep -q "express" "$REPO_PATH/package.json" 2>/dev/null; then
    FRAMEWORKS="Express.js"
    echo "    • Express.js framework"
  fi
  if grep -q "next" "$REPO_PATH/package.json" 2>/dev/null; then
    FRAMEWORKS="Next.js"
    echo "    • Next.js framework"
  fi
  if grep -q "react" "$REPO_PATH/package.json" 2>/dev/null; then
    FRAMEWORKS="$FRAMEWORKS React"
    echo "    • React"
  fi
fi

# Python
if [ -f "$REPO_PATH/requirements.txt" ] || [ -f "$REPO_PATH/pyproject.toml" ]; then
  LANGUAGES="$LANGUAGES Python"
  echo "  ✓ Python detected"
  
  if [ -f "$REPO_PATH/requirements.txt" ]; then
    if grep -q "django" "$REPO_PATH/requirements.txt" 2>/dev/null; then
      FRAMEWORKS="$FRAMEWORKS Django"
      echo "    • Django framework"
    fi
    if grep -q "fastapi" "$REPO_PATH/requirements.txt" 2>/dev/null; then
      FRAMEWORKS="$FRAMEWORKS FastAPI"
      echo "    • FastAPI framework"
    fi
  fi
fi

# Go
if [ -f "$REPO_PATH/go.mod" ]; then
  LANGUAGES="$LANGUAGES Go"
  echo "  ✓ Go detected"
  
  if grep -q "gorilla" "$REPO_PATH/go.mod" 2>/dev/null; then
    FRAMEWORKS="$FRAMEWORKS Gorilla"
    echo "    • Gorilla framework"
  fi
fi

# Java
if [ -f "$REPO_PATH/pom.xml" ] || [ -f "$REPO_PATH/build.gradle" ]; then
  LANGUAGES="$LANGUAGES Java"
  echo "  ✓ Java detected"
  
  if [ -f "$REPO_PATH/pom.xml" ]; then
    if grep -q "spring-boot" "$REPO_PATH/pom.xml" 2>/dev/null; then
      FRAMEWORKS="$FRAMEWORKS Spring Boot"
      echo "    • Spring Boot framework"
    fi
  fi
fi

# Detect databases
echo ""
echo -e "${YELLOW}[3/5] Detecting data stores...${NC}"

if grep -rq "postgres\|postgresql" "$REPO_PATH" --include="*.json" --include="*.yml" --include="*.yaml" --include="*.env*" 2>/dev/null; then
  DATABASES="$DATABASES PostgreSQL"
  echo "  ✓ PostgreSQL detected"
fi

if grep -rq "mongodb\|mongo" "$REPO_PATH" --include="*.json" --include="*.yml" --include="*.yaml" 2>/dev/null; then
  DATABASES="$DATABASES MongoDB"
  echo "  ✓ MongoDB detected"
fi

if grep -rq "redis" "$REPO_PATH" --include="*.json" --include="*.yml" --include="*.yaml" 2>/dev/null; then
  DATABASES="$DATABASES Redis"
  echo "  ✓ Redis detected"
fi

if grep -rq "mysql" "$REPO_PATH" --include="*.json" --include="*.yml" --include="*.yaml" 2>/dev/null; then
  DATABASES="$DATABASES MySQL"
  echo "  ✓ MySQL detected"
fi

# Detect architectural pattern
echo ""
echo -e "${YELLOW}[4/5] Detecting architectural pattern...${NC}"

DOCKERFILE_COUNT=$(find "$REPO_PATH" -name "Dockerfile" -type f 2>/dev/null | wc -l | tr -d ' ')
HAS_DOCKER_COMPOSE=0
if [ -f "$REPO_PATH/docker-compose.yml" ] || [ -f "$REPO_PATH/docker-compose.yaml" ]; then
  HAS_DOCKER_COMPOSE=1
fi

PATTERN="Unknown"
if [ "$DOCKERFILE_COUNT" -gt 1 ] || [ "$HAS_DOCKER_COMPOSE" -eq 1 ]; then
  PATTERN="Microservices"
  echo "  ✓ Microservices pattern detected ($DOCKERFILE_COUNT Dockerfiles)"
elif [ -d "$REPO_PATH/api" ] && [ -d "$REPO_PATH/service" ]; then
  PATTERN="Layered"
  echo "  ✓ Layered architecture detected"
elif grep -rq "kafka\|rabbitmq\|event" "$REPO_PATH" --include="*.json" --include="*.yml" 2>/dev/null; then
  PATTERN="Event-Driven"
  echo "  ✓ Event-driven architecture detected"
else
  PATTERN="Monolith"
  echo "  ✓ Monolithic architecture detected"
fi

# Generate architecture.sruja
echo ""
echo -e "${YELLOW}[5/5] Generating architecture.sruja...${NC}"

PROJECT_NAME=$(basename "$REPO_PATH")
PATTERN_LOWER=$(echo "$PATTERN" | tr '[:upper:]' '[:lower:]')

cat > "$OUTPUT_FILE" << EOF
# Auto-generated architecture for $PROJECT_NAME
# Generated by enhanced Sruja template
# Abstraction level: $ABSTRACTION
# Pattern: $PATTERN

system "$PROJECT_NAME" {
  description "Auto-generated architecture for $PROJECT_NAME - $PATTERN_LOWER pattern"
  
EOF

# Generate based on pattern
case "$PATTERN" in
  "Microservices")
    cat >> "$OUTPUT_FILE" << 'EOF'
  container "API Gateway" {
    technology "Node.js/Express"
    description "Main entry point for external requests"
    
    component "Router"
    component "Auth Middleware"
    component "Rate Limiter"
  }
  
EOF
    
    # Add generic services
    if [ "$DOCKERFILE_COUNT" -gt 1 ]; then
      cat >> "$OUTPUT_FILE" << 'EOF'
  container "Core Service" {
    technology "Detected from codebase"
    description "Main business logic service"
    
    component "API Layer"
    component "Business Logic"
    component "Data Access"
  }
  
  container "Background Worker" {
    technology "Message Queue Consumer"
    description "Processes background jobs"
  }
  
EOF
    fi
    
    # Add databases
    if [ -n "$DATABASES" ]; then
      cat >> "$OUTPUT_FILE" << 'EOF'
  datastore "Primary Database" {
    technology "PostgreSQL"
    description "Main data store"
  }
  
  datastore "Cache" {
    technology "Redis"
    description "Session and data cache"
  }
  
EOF
    fi
    
    # Add relationships
    cat >> "$OUTPUT_FILE" << 'EOF'
  container "API Gateway" -> container "Core Service" "routes to"
  container "Core Service" -> datastore "Primary Database" "queries"
  container "Core Service" -> datastore "Cache" "caches in"
EOF
    ;;
    
  "Layered")
    cat >> "$OUTPUT_FILE" << 'EOF'
  container "Presentation Layer" {
    technology "Frontend Framework"
    description "User interface layer"
    
    component "Components"
    component "Views"
  }
  
  container "API Layer" {
    technology "REST API"
    description "API endpoints and routing"
    
    component "Controllers"
    component "Routes"
    component "Middleware"
  }
  
  container "Business Layer" {
    technology "Service Layer"
    description "Core business logic"
    
    component "Services"
    component "Domain Models"
    component "Validators"
  }
  
  container "Data Layer" {
    technology "Data Access"
    description "Database interactions"
    
    component "Repositories"
    component "ORM Models"
  }
  
  datastore "Database" {
    technology "PostgreSQL"
    description "Primary data store"
  }
  
  container "Presentation Layer" -> container "API Layer" "calls"
  container "API Layer" -> container "Business Layer" "delegates to"
  container "Business Layer" -> container "Data Layer" "uses"
  container "Data Layer" -> datastore "Database" "queries"
EOF
    ;;
    
  "Event-Driven")
    cat >> "$OUTPUT_FILE" << 'EOF'
  container "Command API" {
    technology "Node.js/Express"
    description "Handles write operations"
    
    component "Command Handlers"
    component "Validators"
  }
  
  container "Query API" {
    technology "Node.js/Express"
    description "Handles read operations"
    
    component "Query Handlers"
    component "Projections"
  }
  
  container "Event Store" {
    technology "EventStore"
    description "Stores event stream"
  }
  
  container "Read Model" {
    technology "MongoDB"
    description "Materialized views"
  }
  
  messagebus "Event Bus" {
    technology "Kafka"
    description "Event streaming"
  }
  
  container "Command API" -> container "Event Store" "appends to"
  container "Event Store" -> messagebus "Event Bus" "publishes to"
  messagebus "Event Bus" -> container "Query API" "delivers to"
  container "Query API" -> container "Read Model" "updates"
EOF
    ;;
    
  *)  # Monolith
    cat >> "$OUTPUT_FILE" << 'EOF'
  container "Web Application" {
    technology "Full-Stack Framework"
    description "Monolithic web application"
    
    component "API Endpoints"
    component "Business Logic"
    component "Data Access"
  }
  
  datastore "Database" {
    technology "PostgreSQL"
    description "Primary data store"
  }
  
  container "Web Application" -> datastore "Database" "queries"
EOF
    ;;
esac

cat >> "$OUTPUT_FILE" << EOF

}

# Generated by Sruja Enhanced Template
# Component count: ~$TARGET_COMPONENTS (target)
# Languages: ${LANGUAGES:-Auto-detected}
# Frameworks: ${FRAMEWORKS:-Auto-detected}
# Pattern: $PATTERN
EOF

echo -e "${GREEN}✓ Generated: $OUTPUT_FILE${NC}"
echo ""
echo "Next steps:"
echo "  1. Review and edit $OUTPUT_FILE"
echo "  2. Validate: sruja lint $OUTPUT_FILE"
echo "  3. Check drift: sruja drift -r $REPO_PATH -a $OUTPUT_FILE"
echo ""
