# Self-Hosted Studio

## Overview

**Self-hosted Studio** is a standalone web application that teams can deploy on their own infrastructure. It's designed for:
1. **Easy preview sharing** - Share architecture previews with team members (before PR, during PR, anytime)
2. **Architecture design** - Design and visualize architectures without Git connection
3. **DSL export** - Export designs as DSL files (single or multiple files, zipped)

**Key difference from CLI Studio Server**:
- **CLI Studio Server** (`sruja studio`): Local server, connected to Git repo, reads/writes `.sruja` files directly
- **Self-hosted Studio**: Standalone web app, **no Git connection**, for sharing and designing, exports DSL

## Use Cases

### 1. Share Previews Anytime

**Before PR**:
- Designer creates architecture in Studio
- Shares preview link with team: `https://studio.company.com/preview/abc123`
- Team reviews and provides feedback
- Designer updates and re-shares

**During PR**:
- PR includes preview link: `https://studio.company.com/preview/pr-123`
- Reviewers click link to view architecture changes
- No need to clone repo or run CLI

**Ad-hoc Sharing**:
- Share architecture designs with stakeholders
- Share architecture proposals for discussion
- Share architecture documentation

### 2. Design Architectures

**Visual Design**:
- Create architectures from scratch in Studio
- No need to write DSL manually
- Visual drag-and-drop interface
- Real-time preview

**Iterate Quickly**:
- Make changes visually
- See results immediately
- Export when ready

### 3. Export DSL Files

**Single File Export**:
- Export complete architecture as single `.sruja` file
- Download directly from Studio

**Multiple Files Export**:
- Export as multiple `.sruja` files (systems, containers, components, ADRs, etc.)
- Download as ZIP archive
- Ready to commit to Git repo

## Architecture

```
┌─────────────────────────────────┐
│   Self-Hosted Studio (Web App) │
│   (No Git Connection)           │
└──────────────┬──────────────────┘
               │
               ├──────────────────┐
               │                  │
               ▼                  ▼
    ┌──────────────────┐  ┌──────────────┐
    │  Design/Edit     │  │  Preview     │
    │  Architecture    │  │  Sharing     │
    └────────┬─────────┘  └──────┬───────┘
             │                   │
             │                   │
             ▼                   ▼
    ┌──────────────────┐  ┌──────────────┐
    │  Export DSL     │  │  Share Link  │
    │  (Single/ZIP)   │  │  (URL)       │
    └──────────────────┘  └──────────────┘
```

## Features

### Core Features

1. **Visual Architecture Editor**
   - Drag-and-drop elements
   - Property editing
   - Real-time preview
   - Same UI as CLI Studio

2. **Preview Sharing**
   - Generate shareable preview links
   - View-only mode for shared links
   - No authentication required (or optional)
   - Persistent previews (stored in database or file system)

3. **DSL Export**
   - Export single `.sruja` file
   - Export multiple `.sruja` files (ZIP)
   - Download directly from browser

4. **Import DSL** (Core Feature)
   - Upload `.sruja` files (single or multiple)
   - Load from URL (e.g., GitHub Pages, customer domain, GitHub raw)
   - Paste DSL text directly
   - Import existing architecture from Git repo
   - Start editing immediately after import

## Design Philosophy: Simple but Functional

**Self-hosted Studio is intentionally simple**:
- ✅ Core features: Import, Edit, Export, Share
- ✅ No authentication required
- ✅ No complex collaboration features
- ✅ Easy to deploy and maintain
- ✅ Focus on preview sharing and DSL editing

**Advanced features are Cloud Studio only**:
- ❌ User authentication (Cloud Studio only)
- ❌ Preview management UI (Cloud Studio only)
- ❌ Comments and collaboration (Cloud Studio only)
- ❌ Version history (Cloud Studio only)
- ❌ Approval workflows (Cloud Studio only)

**Why keep it simple?**
- Easier to deploy and maintain
- Lower infrastructure requirements
- Faster to set up
- Focus on core value: sharing previews and editing DSL
- Advanced features available via Cloud Studio when needed

## Deployment Options

Self-hosted Studio supports multiple deployment models. Choose based on your infrastructure, scale, and requirements.

### Option 1: Docker Container (Recommended for Simple Deployments)

**Best for**: Small teams, quick setup, local development

#### Simple Docker Run

```bash
docker run -d \
  --name sruja-studio \
  -p 8080:8080 \
  -v ./previews:/app/previews \
  sruja/studio:latest
```

#### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  studio:
    image: sruja/studio:latest
    ports:
      - "8080:8080"
    environment:
      - STUDIO_PORT=8080
      - STUDIO_STORAGE_PATH=/app/previews
      - STUDIO_MAX_FILE_SIZE=10485760  # 10MB
    volumes:
      - ./previews:/app/previews
    restart: unless-stopped
```

**Deploy**:
```bash
docker-compose up -d
```

**Access**: `http://localhost:8080`

**Pros**:
- ✅ Simple setup
- ✅ Easy to update
- ✅ Works on any Docker host

**Cons**:
- ❌ Single instance (no scaling)
- ❌ Local storage only
- ❌ No high availability

### Option 2: Kubernetes

**Best for**: Production deployments, high availability, scaling

#### Basic Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sruja-studio
  labels:
    app: sruja-studio
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sruja-studio
  template:
    metadata:
      labels:
        app: sruja-studio
    spec:
      containers:
      - name: studio
        image: sruja/studio:latest
        ports:
        - containerPort: 8080
        env:
        - name: STUDIO_PORT
          value: "8080"
        - name: STUDIO_STORAGE_TYPE
          value: "s3"  # or "gcs", "azure", "local"
        - name: STUDIO_STORAGE_PATH
          value: "s3://sruja-previews"  # S3 bucket
        - name: AWS_REGION
          value: "us-east-1"
        - name: AWS_ACCESS_KEY_ID
          valueFrom:
            secretKeyRef:
              name: sruja-studio-secrets
              key: aws-access-key-id
        - name: AWS_SECRET_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: sruja-studio-secrets
              key: aws-secret-access-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: sruja-studio
spec:
  selector:
    app: sruja-studio
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sruja-studio-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sruja-studio
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

#### Secrets

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: sruja-studio-secrets
type: Opaque
stringData:
  aws-access-key-id: "AKIA..."
  aws-secret-access-key: "secret..."
```

**Deploy**:
```bash
kubectl apply -f k8s/secrets.yaml
kubectl apply -f k8s/deployment.yaml
```

**Pros**:
- ✅ High availability
- ✅ Auto-scaling
- ✅ Rolling updates
- ✅ Health checks

**Cons**:
- ❌ Complex setup
- ❌ Requires Kubernetes cluster

### Option 3: Serverless (AWS Lambda)

**Best for**: Low traffic, cost-effective, event-driven

#### Lambda Function Handler

```go
// cmd/studio-server/lambda/main.go
package main

import (
    "context"
    "github.com/aws/aws-lambda-go/events"
    "github.com/aws/aws-lambda-go/lambda"
    "github.com/awslabs/aws-lambda-go-api-proxy/gorillamux"
    "github.com/gorilla/mux"
)

var muxAdapter *gorillamux.GorillaMuxAdapter

func init() {
    r := mux.NewRouter()
    // Setup routes (same as regular server)
    setupRoutes(r)
    muxAdapter = gorillamux.New(r)
}

func handler(ctx context.Context, req events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
    return muxAdapter.ProxyWithContext(ctx, req)
}

func main() {
    lambda.Start(handler)
}
```

#### Serverless Framework Configuration

```yaml
# serverless.yml
service: sruja-studio

provider:
  name: aws
  runtime: provided.al2
  region: us-east-1
  memorySize: 512
  timeout: 30
  environment:
    STUDIO_STORAGE_TYPE: s3
    STUDIO_STORAGE_PATH: s3://sruja-previews
    AWS_REGION: us-east-1
  iamRoleStatements:
    - Effect: Allow
      Action:
        - s3:GetObject
        - s3:PutObject
        - s3:DeleteObject
      Resource: arn:aws:s3:::sruja-previews/*

functions:
  api:
    handler: bootstrap
    events:
      - http:
          path: /{proxy+}
          method: ANY
          cors: true
      - http:
          path: /
          method: ANY
          cors: true

plugins:
  - serverless-offline
```

#### Build and Deploy

```bash
# Build Go binary for Lambda
GOOS=linux GOARCH=amd64 go build -o bootstrap ./cmd/studio-server/lambda

# Package for Lambda
zip function.zip bootstrap

# Deploy with Serverless Framework
serverless deploy
```

**Pros**:
- ✅ Pay per request
- ✅ Auto-scaling
- ✅ No server management
- ✅ Cost-effective for low traffic

**Cons**:
- ❌ Cold starts
- ❌ 15-minute timeout limit
- ❌ Stateless only (use S3 for storage)

### Option 4: Serverless (GCP Cloud Functions)

**Best for**: GCP users, simple deployments

#### Cloud Function

```go
// cmd/studio-server/cloudfunction/main.go
package main

import (
    "net/http"
    "github.com/gorilla/mux"
)

var router *mux.Router

func init() {
    router = mux.NewRouter()
    setupRoutes(router)
}

func StudioHTTP(w http.ResponseWriter, r *http.Request) {
    router.ServeHTTP(w, r)
}
```

#### Deployment

```bash
# Deploy to Cloud Functions
gcloud functions deploy sruja-studio \
  --runtime go121 \
  --trigger-http \
  --allow-unauthenticated \
  --entry-point StudioHTTP \
  --set-env-vars STUDIO_STORAGE_TYPE=gcs,STUDIO_STORAGE_PATH=gs://sruja-previews \
  --memory 512MB \
  --timeout 540s \
  --max-instances 10
```

**Pros**:
- ✅ Simple deployment
- ✅ Auto-scaling
- ✅ Integrated with GCP

**Cons**:
- ❌ GCP-specific
- ❌ Cold starts

### Option 5: Serverless (Vercel)

**Best for**: Frontend-focused, edge deployment

#### Vercel Configuration

```json
// vercel.json
{
  "version": 2,
  "builds": [
    {
      "src": "cmd/studio-server/vercel/main.go",
      "use": "@vercel/go"
    }
  ],
  "routes": [
    {
      "src": "/(.*)",
      "dest": "cmd/studio-server/vercel/main.go"
    }
  ],
  "env": {
    "STUDIO_STORAGE_TYPE": "s3",
    "STUDIO_STORAGE_PATH": "s3://sruja-previews"
  }
}
```

#### Vercel Function

```go
// cmd/studio-server/vercel/main.go
package main

import (
    "net/http"
    "github.com/gorilla/mux"
)

var router *mux.Router

func init() {
    router = mux.NewRouter()
    setupRoutes(router)
}

func Handler(w http.ResponseWriter, r *http.Request) {
    router.ServeHTTP(w, r)
}
```

**Deploy**:
```bash
vercel --prod
```

**Pros**:
- ✅ Edge deployment
- ✅ Simple setup
- ✅ Great for static + API

**Cons**:
- ❌ Function timeout limits
- ❌ Stateless only

### Option 6: AWS ECS/Fargate

**Best for**: AWS users, container-based, managed

#### Task Definition

```json
{
  "family": "sruja-studio",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "containerDefinitions": [
    {
      "name": "studio",
      "image": "sruja/studio:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "STUDIO_PORT",
          "value": "8080"
        },
        {
          "name": "STUDIO_STORAGE_TYPE",
          "value": "s3"
        },
        {
          "name": "STUDIO_STORAGE_PATH",
          "value": "s3://sruja-previews"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/sruja-studio",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### Deploy

```bash
# Register task definition
aws ecs register-task-definition --cli-input-json file://task-definition.json

# Create service
aws ecs create-service \
  --cluster sruja-cluster \
  --service-name sruja-studio \
  --task-definition sruja-studio \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-123],securityGroups=[sg-123],assignPublicIp=ENABLED}"
```

**Pros**:
- ✅ Managed containers
- ✅ Auto-scaling
- ✅ Integrated with AWS services

**Cons**:
- ❌ AWS-specific
- ❌ More expensive than Lambda

### Option 7: GCP Cloud Run

**Best for**: GCP users, serverless containers

#### Cloud Run Deployment

```bash
# Build and push container
gcloud builds submit --tag gcr.io/PROJECT_ID/sruja-studio

# Deploy to Cloud Run
gcloud run deploy sruja-studio \
  --image gcr.io/PROJECT_ID/sruja-studio \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars STUDIO_STORAGE_TYPE=gcs,STUDIO_STORAGE_PATH=gs://sruja-previews \
  --memory 512Mi \
  --cpu 1 \
  --min-instances 1 \
  --max-instances 10
```

**Pros**:
- ✅ Serverless containers
- ✅ Pay per request
- ✅ Auto-scaling

**Cons**:
- ❌ GCP-specific
- ❌ Cold starts

### Option 8: Traditional Server (VM/EC2)

**Best for**: Full control, custom configurations

#### Systemd Service

```ini
# /etc/systemd/system/sruja-studio.service
[Unit]
Description=Sruja Studio Server
After=network.target

[Service]
Type=simple
User=sruja
WorkingDirectory=/opt/sruja-studio
ExecStart=/opt/sruja-studio/sruja-studio
Environment="STUDIO_PORT=8080"
Environment="STUDIO_STORAGE_PATH=/var/lib/sruja-studio/previews"
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### Nginx Reverse Proxy

```nginx
# /etc/nginx/sites-available/sruja-studio
server {
    listen 80;
    server_name studio.example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

**Deploy**:
```bash
# Install service
sudo systemctl enable sruja-studio
sudo systemctl start sruja-studio

# Setup Nginx
sudo ln -s /etc/nginx/sites-available/sruja-studio /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

**Pros**:
- ✅ Full control
- ✅ Custom configurations
- ✅ Persistent storage

**Cons**:
- ❌ Manual management
- ❌ No auto-scaling
- ❌ Requires maintenance

### Option 9: Azure Container Instances

**Best for**: Azure users, simple container deployment

#### Azure Deployment

```bash
# Create resource group
az group create --name sruja-studio-rg --location eastus

# Create container instance
az container create \
  --resource-group sruja-studio-rg \
  --name sruja-studio \
  --image sruja/studio:latest \
  --dns-name-label sruja-studio \
  --ports 8080 \
  --environment-variables \
    STUDIO_PORT=8080 \
    STUDIO_STORAGE_TYPE=azure \
    STUDIO_STORAGE_PATH=previews \
  --azure-file-volume-share-name previews \
  --azure-file-volume-account-name storageaccount \
  --azure-file-volume-account-key <key> \
  --azure-file-volume-mount-path /app/previews
```

**Pros**:
- ✅ Simple deployment
- ✅ Azure integration

**Cons**:
- ❌ Azure-specific
- ❌ Limited scaling

### Storage Backends

All deployment options support multiple storage backends:

#### Local Storage (Docker, VM)

```bash
STUDIO_STORAGE_TYPE=local
STUDIO_STORAGE_PATH=/app/previews
```

#### S3 (AWS)

```bash
STUDIO_STORAGE_TYPE=s3
STUDIO_STORAGE_PATH=s3://bucket-name/previews
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
```

#### Google Cloud Storage

```bash
STUDIO_STORAGE_TYPE=gcs
STUDIO_STORAGE_PATH=gs://bucket-name/previews
GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json
```

#### Azure Blob Storage

```bash
STUDIO_STORAGE_TYPE=azure
STUDIO_STORAGE_PATH=previews
AZURE_STORAGE_ACCOUNT=...
AZURE_STORAGE_KEY=...
```

### Comparison Matrix

| Deployment | Best For | Scaling | Cost | Complexity |
|------------|----------|---------|------|------------|
| **Docker** | Small teams, dev | Manual | Low | Low |
| **Kubernetes** | Production, HA | Auto | Medium | High |
| **AWS Lambda** | Low traffic, events | Auto | Very Low | Medium |
| **GCP Functions** | GCP users | Auto | Low | Low |
| **Vercel** | Frontend-focused | Auto | Low | Low |
| **ECS/Fargate** | AWS containers | Auto | Medium | Medium |
| **Cloud Run** | GCP containers | Auto | Low | Low |
| **VM/EC2** | Full control | Manual | Medium | High |
| **Azure ACI** | Azure users | Manual | Medium | Low |

### Recommended Deployment by Use Case

**Development/Testing**:
- Docker Compose

**Small Team (< 10 users)**:
- Docker on single server
- Vercel (if frontend-focused)

**Medium Team (10-100 users)**:
- Kubernetes (2-3 replicas)
- AWS ECS/Fargate
- GCP Cloud Run

**Large Team (100+ users)**:
- Kubernetes (auto-scaling)
- AWS ECS/Fargate (auto-scaling)
- GCP Cloud Run (auto-scaling)

**Cost-Optimized (Low Traffic)**:
- AWS Lambda
- GCP Cloud Functions
- Vercel

**High Availability Required**:
- Kubernetes (multi-zone)
- AWS ECS/Fargate (multi-AZ)
- GCP Cloud Run (multi-region)

## Implementation

### Backend (Go)

**Storage abstraction** for multiple backends:

```go
// pkg/studio/storage/storage.go
package storage

type Storage interface {
    Save(id string, data []byte) error
    Load(id string) ([]byte, error)
    Delete(id string) error
    List() ([]string, error)
}

// Local storage
type LocalStorage struct {
    basePath string
}

// S3 storage
type S3Storage struct {
    bucket   string
    region   string
    s3Client *s3.Client
}

// GCS storage
type GCSStorage struct {
    bucket   string
    gcsClient *storage.Client
}

// Azure storage
type AzureStorage struct {
    accountName string
    accountKey  string
    container  string
    client     *azblob.ContainerClient
}
```

**Minimal API server**:
```go
// cmd/studio-server/main.go
package main

import (
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "os"
    "path/filepath"
    "strings"
    "time"
    
    "github.com/gorilla/mux"
)

type StudioServer struct {
    storage     storage.Storage
    port        string
    storageType string
}

func main() {
    storageType := os.Getenv("STUDIO_STORAGE_TYPE")
    if storageType == "" {
        storageType = "local"
    }
    
    storagePath := os.Getenv("STUDIO_STORAGE_PATH")
    if storagePath == "" {
        storagePath = "./previews"
    }
    
    port := os.Getenv("STUDIO_PORT")
    if port == "" {
        port = "8080"
    }
    
    // Initialize storage backend
    var storageBackend storage.Storage
    switch storageType {
    case "s3":
        storageBackend = storage.NewS3Storage(storagePath, os.Getenv("AWS_REGION"))
    case "gcs":
        storageBackend = storage.NewGCSStorage(storagePath)
    case "azure":
        storageBackend = storage.NewAzureStorage(storagePath, os.Getenv("AZURE_STORAGE_ACCOUNT"), os.Getenv("AZURE_STORAGE_KEY"))
    default:
        // Local storage
        os.MkdirAll(storagePath, 0755)
        storageBackend = storage.NewLocalStorage(storagePath)
    }
    
    server := &StudioServer{
        storage:     storageBackend,
        port:        port,
        storageType: storageType,
    }
    
    // Serve static files (React app)
    r := mux.NewRouter()
    r.PathPrefix("/").Handler(http.FileServer(http.Dir("./studio-dist")))
    
    // API endpoints
    api := r.PathPrefix("/api").Subrouter()
    api.HandleFunc("/preview", server.createPreview).Methods("POST")
    api.HandleFunc("/preview/{id}", server.getPreview).Methods("GET")
    api.HandleFunc("/preview/{id}", server.updatePreview).Methods("PUT")
    api.HandleFunc("/preview/{id}", server.deletePreview).Methods("DELETE")
    api.HandleFunc("/import", server.importDSL).Methods("POST") // Import DSL
    api.HandleFunc("/import/url", server.importDSLFromURL).Methods("POST") // Import from URL
    api.HandleFunc("/export/{id}", server.exportDSL).Methods("GET")
    api.HandleFunc("/export/{id}/zip", server.exportDSLZip).Methods("GET")
    
    fmt.Printf("Studio server running on :%s\n", port)
    http.ListenAndServe(":"+port, r)
}

// Create preview
func (s *StudioServer) createPreview(w http.ResponseWriter, r *http.Request) {
    var req struct {
        Architecture ArchitectureJSON `json:"architecture"`
        Name         string           `json:"name,omitempty"`
    }
    
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }
    
    // Generate preview ID
    previewID := generatePreviewID()
    
    // Save preview
    data, err := json.Marshal(req.Architecture)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    if err := s.storage.Save(previewID+".json", data); err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Return preview URL
    response := map[string]string{
        "id":  previewID,
        "url": fmt.Sprintf("/preview/%s", previewID),
    }
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(response)
}

// Get preview
func (s *StudioServer) getPreview(w http.ResponseWriter, r *http.Request) {
    vars := mux.Vars(r)
    previewID := vars["id"]
    
    // Load from storage
    data, err := s.storage.Load(previewID + ".json")
    if err != nil {
        http.Error(w, "Preview not found", http.StatusNotFound)
        return
    }
    
    w.Header().Set("Content-Type", "application/json")
    w.Write(data)
}

// Export DSL (single file)
func (s *StudioServer) exportDSL(w http.ResponseWriter, r *http.Request) {
    vars := mux.Vars(r)
    previewID := vars["id"]
    
    // Load architecture JSON from storage
    data, err := s.storage.Load(previewID + ".json")
    if err != nil {
        http.Error(w, "Preview not found", http.StatusNotFound)
        return
    }
    
    var arch ArchitectureJSON
    if err := json.Unmarshal(data, &arch); err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Convert JSON to DSL
    dsl, err := jsonToDSL(arch)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Return DSL file
    w.Header().Set("Content-Type", "text/plain")
    w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%s.sruja", previewID))
    w.Write([]byte(dsl))
}

// Export DSL (multiple files, ZIP)
func (s *StudioServer) exportDSLZip(w http.ResponseWriter, r *http.Request) {
    vars := mux.Vars(r)
    previewID := vars["id"]
    
    // Load architecture JSON from storage
    data, err := s.storage.Load(previewID + ".json")
    if err != nil {
        http.Error(w, "Preview not found", http.StatusNotFound)
        return
    }
    
    var arch ArchitectureJSON
    if err := json.Unmarshal(data, &arch); err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Convert to multiple DSL files
    files, err := jsonToDSLFiles(arch)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Create ZIP archive
    zipData, err := createZipArchive(files)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Return ZIP file
    w.Header().Set("Content-Type", "application/zip")
    w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%s.zip", previewID))
    w.Write(zipData)
}

// Import DSL (upload file or paste text)
func (s *StudioServer) importDSL(w http.ResponseWriter, r *http.Request) {
    // Check if file upload or text paste
    contentType := r.Header.Get("Content-Type")
    
    if strings.HasPrefix(contentType, "multipart/form-data") {
        // File upload
        err := r.ParseMultipartForm(10 << 20) // 10 MB max
        if err != nil {
            http.Error(w, err.Error(), http.StatusBadRequest)
            return
        }
        
        files := r.MultipartForm.File["files"]
        if len(files) == 0 {
            http.Error(w, "No files uploaded", http.StatusBadRequest)
            return
        }
        
        // Handle single or multiple files
        var architectures []ArchitectureJSON
        for _, fileHeader := range files {
            file, err := fileHeader.Open()
            if err != nil {
                http.Error(w, err.Error(), http.StatusBadRequest)
                return
            }
            defer file.Close()
            
            // Read DSL content
            dslBytes, err := io.ReadAll(file)
            if err != nil {
                http.Error(w, err.Error(), http.StatusBadRequest)
                return
            }
            
            // Parse DSL to JSON
            arch, err := dslToJSON(string(dslBytes))
            if err != nil {
                http.Error(w, fmt.Sprintf("Failed to parse DSL: %v", err), http.StatusBadRequest)
                return
            }
            
            architectures = append(architectures, arch)
        }
        
        // Merge multiple architectures if needed
        mergedArch := mergeArchitectures(architectures)
        
        // Generate preview ID and save
        previewID := generatePreviewID()
        data, err := json.Marshal(mergedArch)
        if err != nil {
            http.Error(w, err.Error(), http.StatusInternalServerError)
            return
        }
        
        if err := s.storage.Save(previewID+".json", data); err != nil {
            http.Error(w, err.Error(), http.StatusInternalServerError)
            return
        }
        
        // Return preview info
        response := map[string]interface{}{
            "id":           previewID,
            "url":          fmt.Sprintf("/preview/%s", previewID),
            "architecture": mergedArch,
        }
        
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(response)
        
    } else {
        // Text paste
        var req struct {
            DSL string `json:"dsl"`
        }
        
        if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
            http.Error(w, err.Error(), http.StatusBadRequest)
            return
        }
        
        // Parse DSL to JSON
        arch, err := dslToJSON(req.DSL)
        if err != nil {
            http.Error(w, fmt.Sprintf("Failed to parse DSL: %v", err), http.StatusBadRequest)
            return
        }
        
        // Generate preview ID and save
        previewID := generatePreviewID()
        data, err := json.Marshal(arch)
        if err != nil {
            http.Error(w, err.Error(), http.StatusInternalServerError)
            return
        }
        
        if err := s.storage.Save(previewID+".json", data); err != nil {
            http.Error(w, err.Error(), http.StatusInternalServerError)
            return
        }
        
        // Return preview info
        response := map[string]interface{}{
            "id":           previewID,
            "url":          fmt.Sprintf("/preview/%s", previewID),
            "architecture": arch,
        }
        
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(response)
    }
}

// Import DSL from URL
func (s *StudioServer) importDSLFromURL(w http.ResponseWriter, r *http.Request) {
    var req struct {
        URL string `json:"url"`
    }
    
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }
    
    // Fetch DSL from URL
    resp, err := http.Get(req.URL)
    if err != nil {
        http.Error(w, fmt.Sprintf("Failed to fetch URL: %v", err), http.StatusBadRequest)
        return
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != http.StatusOK {
        http.Error(w, fmt.Sprintf("Failed to fetch URL: %s", resp.Status), http.StatusBadRequest)
        return
    }
    
    // Read DSL content
    dslBytes, err := io.ReadAll(resp.Body)
    if err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }
    
    // Parse DSL to JSON
    arch, err := dslToJSON(string(dslBytes))
    if err != nil {
        http.Error(w, fmt.Sprintf("Failed to parse DSL: %v", err), http.StatusBadRequest)
        return
    }
    
    // Generate preview ID and save
    previewID := generatePreviewID()
    data, err := json.Marshal(arch)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    if err := s.storage.Save(previewID+".json", data); err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    // Return preview info
    response := map[string]interface{}{
        "id":           previewID,
        "url":          fmt.Sprintf("/preview/%s", previewID),
        "architecture": arch,
    }
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(response)
}

func generatePreviewID() string {
    return fmt.Sprintf("%d", time.Now().UnixNano())
}

// Helper functions (implement these)
func dslToJSON(dsl string) (ArchitectureJSON, error) {
    // Parse DSL using Sruja parser
    // Convert AST to JSON
    // Return ArchitectureJSON
    // Implementation depends on Sruja parser
}

func mergeArchitectures(archs []ArchitectureJSON) ArchitectureJSON {
    // Merge multiple architectures into one
    // Combine systems, containers, components, etc.
    // Return merged ArchitectureJSON
}
```

### Frontend (React)

**Studio component with export**:
```typescript
// Studio.tsx
export function Studio() {
  const [architecture, setArchitecture] = useState<ArchitectureJSON | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);
  
  // Load preview from URL
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const previewParam = urlParams.get('preview');
    
    if (previewParam) {
      // Load from self-hosted Studio
      loadPreview(previewParam);
    }
  }, []);
  
  const loadPreview = async (id: string) => {
    const response = await fetch(`/api/preview/${id}`);
    if (!response.ok) {
      throw new Error('Failed to load preview');
    }
    const json = await response.json();
    setArchitecture(json);
    setPreviewId(id);
  };
  
  const importDSL = async (dsl: string) => {
    const response = await fetch('/api/import', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ dsl }),
    });
    
    if (!response.ok) {
      throw new Error('Failed to import DSL');
    }
    
    const { id, architecture } = await response.json();
    setArchitecture(architecture);
    setPreviewId(id);
    
    // Update URL
    window.history.pushState({}, '', `/preview/${id}`);
  };
  
  const importDSLFromFile = async (files: FileList) => {
    const formData = new FormData();
    Array.from(files).forEach(file => {
      formData.append('files', file);
    });
    
    const response = await fetch('/api/import', {
      method: 'POST',
      body: formData,
    });
    
    if (!response.ok) {
      throw new Error('Failed to import DSL files');
    }
    
    const { id, architecture } = await response.json();
    setArchitecture(architecture);
    setPreviewId(id);
    
    // Update URL
    window.history.pushState({}, '', `/preview/${id}`);
  };
  
  const importDSLFromURL = async (url: string) => {
    const response = await fetch('/api/import/url', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url }),
    });
    
    if (!response.ok) {
      throw new Error('Failed to import DSL from URL');
    }
    
    const { id, architecture } = await response.json();
    setArchitecture(architecture);
    setPreviewId(id);
    
    // Update URL
    window.history.pushState({}, '', `/preview/${id}`);
  };
  
  const savePreview = async () => {
    if (!architecture) return;
    
    const response = await fetch('/api/preview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ architecture }),
    });
    
    const { id, url } = await response.json();
    setPreviewId(id);
    
    // Update URL
    window.history.pushState({}, '', `/preview/${id}`);
    
    // Show share link
    alert(`Preview saved! Share link: ${window.location.origin}/preview/${id}`);
  };
  
  const exportDSL = async (format: 'single' | 'zip') => {
    if (!previewId) return;
    
    const endpoint = format === 'single' 
      ? `/api/export/${previewId}`
      : `/api/export/${previewId}/zip`;
    
    const response = await fetch(endpoint);
    const blob = await response.blob();
    
    // Download file
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${previewId}.${format === 'single' ? 'sruja' : 'zip'}`;
    a.click();
    window.URL.revokeObjectURL(url);
  };
  
  return (
    <div>
      <StudioToolbar
        onSave={savePreview}
        onImportFile={(files) => importDSLFromFile(files)}
        onImportURL={(url) => importDSLFromURL(url)}
        onImportPaste={(dsl) => importDSL(dsl)}
        onExportSingle={() => exportDSL('single')}
        onExportZip={() => exportDSL('zip')}
      />
      <StudioCanvas architecture={architecture} />
    </div>
  );
}
```

## Usage Examples

### 1. Share Preview Before PR

**Designer**:
1. Opens Studio: `https://studio.company.com`
2. Designs architecture visually
3. Clicks "Save Preview"
4. Gets share link: `https://studio.company.com/preview/abc123`
5. Shares link in Slack/email

**Reviewer**:
1. Clicks link
2. Views architecture in Studio (read-only)
3. Provides feedback

### 2. Share Preview During PR

**PR Comment**:
```markdown
## 🏗️ Architecture Preview

**View Preview:**
- 🎨 [Open in Studio](https://studio.company.com/preview/pr-123)
- 📦 [Download DSL](./architecture.sruja)
```

**Reviewer**:
1. Clicks Studio link
2. Views architecture changes
3. Reviews visually

### 3. Import, Edit, and Commit Workflow

**Complete workflow**:
1. **Import**: Upload existing `.sruja` file from Git repo
2. **Edit**: Make changes visually in Studio
3. **Preview**: Save preview and share with team
4. **Export**: Export as single file or ZIP
5. **Commit**: Download and commit to Git repo

**Example**:
```bash
# 1. Clone repo
git clone https://github.com/org/repo.git
cd repo

# 2. Open Studio and import
# Upload: architecture.sruja

# 3. Make changes in Studio
# Add new container, update relations, etc.

# 4. Export DSL
# Download: architecture.sruja

# 5. Replace file and commit
cp ~/Downloads/architecture.sruja ./architecture.sruja
git add architecture.sruja
git commit -m "Update architecture: add analytics container"
git push
```

### 4. Import from GitHub Raw URL

**Direct import from GitHub**:
1. Opens Studio: `https://studio.company.com`
2. Clicks "Import" → "From URL"
3. Pastes GitHub raw URL: `https://raw.githubusercontent.com/org/repo/main/architecture.sruja`
4. Studio fetches, parses, and loads
5. User edits and exports

## Comparison: CLI Studio vs Self-Hosted Studio

| Feature | CLI Studio Server | Self-Hosted Studio |
|---------|------------------|-------------------|
| **Git Connection** | ✅ Yes (reads/writes `.sruja` files) | ❌ No (standalone) |
| **File Operations** | ✅ Direct read/write | ❌ Export only |
| **Preview Sharing** | ❌ Local only | ✅ Shareable links |
| **Design from Scratch** | ✅ Yes | ✅ Yes |
| **Export DSL** | ✅ Yes | ✅ Yes (single/ZIP) |
| **Deployment** | Local (`sruja studio`) | Self-hosted (Docker/K8s) |
| **Use Case** | Local development | Team sharing |

## Benefits

✅ **Easy Sharing** - Share previews with team members instantly  
✅ **No Git Required** - Design and share without Git connection  
✅ **Persistent Previews** - Previews stored on server, accessible anytime  
✅ **Export Ready** - Export DSL files when ready to commit  
✅ **Team Collaboration** - Multiple team members can view same preview  
✅ **Before PR** - Share designs before creating PR  
✅ **During PR** - Share previews in PR comments  

## Future Enhancements

1. **User Authentication**
   - Team members can log in
   - Track who created what
   - Access control

2. **Preview Management**
   - List all previews
   - Delete old previews
   - Search previews

3. **Collaboration Features**
   - Comments on previews
   - Version history
   - Approval workflow

4. **Integration with CLI Studio**
   - Import from CLI Studio
   - Export to CLI Studio
   - Sync previews

