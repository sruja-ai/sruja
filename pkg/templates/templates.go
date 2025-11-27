// pkg/templates/templates.go
package templates

import (
	"fmt"
	"os"
	"path/filepath"
)

// Template represents a project template.
type Template struct {
	Name        string
	Description string
	Files       map[string]string
}

// GetTemplate returns a template by name.
func GetTemplate(name string) (*Template, error) {
	switch name {
	case "basic":
		return basicTemplate(), nil
	case "microservices":
		return microservicesTemplate(), nil
	case "event-driven":
		return eventDrivenTemplate(), nil
	case "monolith":
		return monolithTemplate(), nil
	case "api-gateway":
		return apiGatewayTemplate(), nil
	case "service-mesh":
		return serviceMeshTemplate(), nil
	default:
		return nil, fmt.Errorf("unknown template: %s", name)
	}
}

// ListTemplates returns all available templates.
func ListTemplates() []string {
	return []string{"basic", "microservices", "event-driven", "monolith", "api-gateway", "service-mesh"}
}

// Generate generates the template files in the target directory.
func (t *Template) Generate(targetDir string) error {
	// Create directory if it doesn't exist
	if err := os.MkdirAll(targetDir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Write files
	for filename, content := range t.Files {
		filePath := filepath.Join(targetDir, filename)
		
		// Create directory for file if needed
		dir := filepath.Dir(filePath)
		if err := os.MkdirAll(dir, 0755); err != nil {
			return fmt.Errorf("failed to create directory for %s: %w", filename, err)
		}

		// Write file
		if err := os.WriteFile(filePath, []byte(content), 0644); err != nil {
			return fmt.Errorf("failed to write %s: %w", filename, err)
		}
	}

	return nil
}

// basicTemplate returns a basic starter template.
func basicTemplate() *Template {
	return &Template{
		Name:        "basic",
		Description: "Basic architecture starter",
		Files: map[string]string{
			"architecture.sruja": `architecture "My System" {
  system API "API Service" {
    description "Main API service that handles requests"
    
    container WebApp "Web Application" {
      technology "React"
      
      component Dashboard "Dashboard Component" {
        technology "React"
      }
    }
    
    datastore DB "PostgreSQL Database" {
      technology "PostgreSQL"
    }
  }
  
  person User "End User"
  
  User -> API "uses"
  API.WebApp -> API.DB "reads from and writes to"
}
`,
            "README.md": "# My Architecture\n\nThis project uses Sruja DSL to define the architecture.\n\n## Getting Started\n\n1. View the architecture: architecture.sruja\n2. Generate diagrams: sruja compile architecture.sruja\n3. Validate: sruja lint architecture.sruja\n4. Explain elements: sruja explain API\n\n## Commands\n\n- sruja compile architecture.sruja - Generate diagrams\n- sruja lint architecture.sruja - Validate architecture\n- sruja explain <element-id> - Get element explanation\n- sruja list systems - List all systems\n\n## Learn More\n\n- https://sruja.dev/docs\n- https://sruja.dev/docs/quickstart\n",
			".vscode/settings.json": `{
  "files.associations": {
    "*.sruja": "sruja"
  },
  "[sruja]": {
    "editor.defaultFormatter": "sruja.formatter"
  }
}
`,
		},
	}
}

// microservicesTemplate returns a microservices architecture template.
func microservicesTemplate() *Template {
	return &Template{
		Name:        "microservices",
		Description: "Microservices architecture pattern",
		Files: map[string]string{
			"architecture.sruja": `architecture "E-Commerce Platform" {
  system Frontend "Frontend Application" {
    container WebApp "Web Application" {
      technology "React"
      
      component ProductCatalog "Product Catalog"
      component ShoppingCart "Shopping Cart"
    }
  }
  
  system ProductService "Product Service" {
    container API "Product API" {
      technology "Node.js"
      
      component ProductController "Product Controller"
      component ProductService "Product Service Logic"
    }
    
    datastore ProductDB "Product Database" {
      technology "MongoDB"
    }
  }
  
  system OrderService "Order Service" {
    container API "Order API" {
      technology "Go"
      
      component OrderController "Order Controller"
      component PaymentProcessor "Payment Processor"
    }
    
    datastore OrderDB "Order Database" {
      technology "PostgreSQL"
    }
  }
  
  system UserService "User Service" {
    container API "User API" {
      technology "Java"
      
      component UserController "User Controller"
      component AuthService "Authentication Service"
    }
    
    datastore UserDB "User Database" {
      technology "PostgreSQL"
    }
  }
  
  queue OrderQueue "Order Queue" {
    technology "RabbitMQ"
  }
  
  person Customer "Customer"
  
  Customer -> Frontend "browses and purchases"
  Frontend.WebApp -> ProductService.API "fetches products"
  Frontend.WebApp -> OrderService.API "places orders"
  Frontend.WebApp -> UserService.API "authenticates"
  ProductService.API -> ProductService.ProductDB "stores products"
  OrderService.API -> OrderService.OrderDB "stores orders"
  OrderService.API -> OrderQueue "publishes order events"
  UserService.API -> UserService.UserDB "stores users"
}
`,
            "README.md": "# E-Commerce Microservices Architecture\n\nThis is a microservices-based e-commerce platform using Sruja DSL.\n\n## Architecture Overview\n\n- Frontend Service: React-based web application\n- Product Service: Manages product catalog\n- Order Service: Handles order processing and payments\n- User Service: Manages users and authentication\n\n## Getting Started\n\nGenerate diagrams: sruja compile architecture.sruja --output diagram.mermaid\nValidate architecture: sruja lint architecture.sruja\nExplain a service: sruja explain ProductService\n\n## Learn More\n\n- https://sruja.dev/docs/patterns/microservices\n",
		},
	}
}

// eventDrivenTemplate returns an event-driven architecture template.
func eventDrivenTemplate() *Template {
	return &Template{
		Name:        "event-driven",
		Description: "Event-driven architecture pattern",
		Files: map[string]string{
			"architecture.sruja": `architecture "Event-Driven System" {
  system EventProducer "Event Producer Service" {
    container API "Event API" {
      component EventEmitter "Event Emitter"
    }
  }
  
  system EventBus "Event Bus" {
    queue EventQueue "Event Queue" {
      technology "Kafka"
    }
  }
  
  system EventConsumer1 "Order Consumer Service" {
    container Service "Order Service" {
      component OrderProcessor "Order Processor"
      component EventHandler "Event Handler"
    }
    
    datastore OrderDB "Order Database"
  }
  
  system EventConsumer2 "Notification Consumer Service" {
    container Service "Notification Service" {
      component EmailSender "Email Sender"
      component EventHandler "Event Handler"
    }
  }
  
  EventProducer.API -> EventBus.EventQueue "publishes events"
  EventBus.EventQueue -> EventConsumer1.Service "consumes order events"
  EventBus.EventQueue -> EventConsumer2.Service "consumes notification events"
  EventConsumer1.Service -> EventConsumer1.OrderDB "stores orders"
}
`,
            "README.md": "# Event-Driven Architecture\n\nThis architecture uses an event-driven pattern with message queues.\n\n## Components\n\n- Event Producer: Publishes events to the event bus\n- Event Bus: Message queue (Kafka) for event distribution\n- Event Consumers: Services that process events\n\n## Learn More\n\n- https://sruja.dev/docs/patterns/event-driven\n",
		},
	}
}

// monolithTemplate returns a monolithic architecture template.
func monolithTemplate() *Template {
	return &Template{
		Name:        "monolith",
		Description: "Monolithic architecture pattern",
		Files: map[string]string{
			"architecture.sruja": `architecture "Monolithic Application" {
  system App "Monolithic Application" {
    container WebServer "Web Server" {
      technology "Express.js"
      
      component UserModule "User Module"
      component ProductModule "Product Module"
      component OrderModule "Order Module"
      component PaymentModule "Payment Module"
    }
    
    datastore DB "Application Database" {
      technology "PostgreSQL"
    }
    
    queue Cache "Cache" {
      technology "Redis"
    }
  }
  
  person User "End User"
  
  User -> App.WebServer "uses"
  App.WebServer -> App.DB "reads from and writes to"
  App.WebServer -> App.Cache "caches data"
}
`,
            "README.md": "# Monolithic Architecture\n\nThis is a monolithic application architecture.\n\n## Components\n\n- Web Server: Handles all HTTP requests\n- Database: Single database for all data\n- Cache: Redis cache for performance\n\n## Learn More\n\n- https://sruja.dev/docs/patterns/monolith\n",
		},
	}
}

// apiGatewayTemplate returns an API Gateway architecture template.
func apiGatewayTemplate() *Template {
	return &Template{
		Name:        "api-gateway",
		Description: "API Gateway pattern",
		Files: map[string]string{
			"architecture.sruja": `architecture "API Gateway Architecture" {
  system APIGateway "API Gateway" {
    container Gateway "API Gateway Service" {
      technology "Kong"
      
      component Router "Request Router"
      component Auth "Authentication"
      component RateLimiter "Rate Limiter"
      component LoadBalancer "Load Balancer"
    }
  }
  
  system Service1 "User Service" {
    container API "User API" {
      technology "Node.js"
    }
    
    datastore UserDB "User Database"
  }
  
  system Service2 "Product Service" {
    container API "Product API" {
      technology "Go"
    }
    
    datastore ProductDB "Product Database"
  }
  
  person Client "API Client"
  
  Client -> APIGateway.Gateway "sends requests"
  APIGateway.Gateway -> Service1.API "routes to user service"
  APIGateway.Gateway -> Service2.API "routes to product service"
  Service1.API -> Service1.UserDB "stores users"
  Service2.API -> Service2.ProductDB "stores products"
}
`,
            "README.md": "# API Gateway Architecture\n\nThis architecture uses an API Gateway pattern for routing and management.\n\n## Components\n\n- API Gateway: Single entry point for all API requests\n- Backend Services: Multiple microservices behind the gateway\n\n## Learn More\n\n- https://sruja.dev/docs/patterns/api-gateway\n",
		},
	}
}

// serviceMeshTemplate returns a service mesh architecture template.
func serviceMeshTemplate() *Template {
	return &Template{
		Name:        "service-mesh",
		Description: "Service mesh architecture pattern",
		Files: map[string]string{
			"architecture.sruja": `architecture "Service Mesh Architecture" {
  system ServiceMesh "Service Mesh" {
    container ControlPlane "Control Plane" {
      technology "Istio"
      
      component Pilot "Service Discovery"
      component Citadel "Security"
      component Galley "Configuration"
    }
    
    container DataPlane "Data Plane" {
      technology "Envoy"
      
      component Proxy "Sidecar Proxy"
    }
  }
  
  system Frontend "Frontend Service" {
    container App "Frontend Application" {
      technology "React"
    }
  }
  
  system Backend1 "Backend Service 1" {
    container API "Backend API" {
      technology "Go"
    }
  }
  
  system Backend2 "Backend Service 2" {
    container API "Backend API" {
      technology "Node.js"
    }
  }
  
  person User "End User"
  
  User -> Frontend.App "uses"
  Frontend.App -> ServiceMesh.DataPlane "routes through mesh"
  ServiceMesh.DataPlane -> Backend1.API "routes to service 1"
  ServiceMesh.DataPlane -> Backend2.API "routes to service 2"
  ServiceMesh.ControlPlane -> ServiceMesh.DataPlane "manages"
}
`,
            "README.md": "# Service Mesh Architecture\n\nThis architecture uses a service mesh for service-to-service communication.\n\n## Components\n\n- Service Mesh: Handles inter-service communication, security, and observability\n- Services: Individual microservices with sidecar proxies\n\n## Learn More\n\n- https://sruja.dev/docs/patterns/service-mesh\n",
		},
	}
}
