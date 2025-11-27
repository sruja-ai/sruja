export const examples = [
  {
    name: 'Simple Example',
    code: `model {
  system User "End User"
  
  system API "API Service" {
    container WebApp "Web Application" {
      technology "React"
    }
    
    container Database "PostgreSQL Database" {
      technology "PostgreSQL 14"
    }
  }
  
  User -> WebApp "Uses"
  WebApp -> Database "Reads/Writes"
}

requirements {
  R1: functional "Must handle 10k concurrent users"
  R2: constraint "Must use PostgreSQL"
}

adrs {
  ADR001: "Use microservices architecture for scalability"
}`,
  },
  {
    name: 'Microservices',
    code: `model {
  system User "End User"
  
  system ECommerce "E-Commerce Platform" {
    container WebApp "Web Application" {
      technology "Next.js"
    }
    
    container AuthService "Authentication Service" {
      technology "Node.js"
    }
    
    container OrderService "Order Service" {
      technology "Go"
    }
    
    container PaymentService "Payment Service" {
      technology "Python"
    }
    
    container Database "PostgreSQL" {
      technology "PostgreSQL"
    }
  }
  
  User -> WebApp "Uses"
  WebApp -> AuthService "Authenticates"
  WebApp -> OrderService "Creates Orders"
  WebApp -> PaymentService "Processes Payments"
  OrderService -> Database "Stores Data"
  PaymentService -> Database "Stores Data"
}`,
  },
  {
    name: 'Event-Driven',
    code: `model {
  system User "End User"
  
  system Platform "Event Platform" {
    container API "API Gateway" {
      technology "Kong"
    }
    
    container ServiceA "Service A" {
      technology "Go"
    }
    
    container ServiceB "Service B" {
      technology "Node.js"
    }
    
    queue EventQueue "Event Queue" {
      technology "RabbitMQ"
    }
    
    container Database "Database" {
      technology "PostgreSQL"
    }
  }
  
  User -> API "Sends Events"
  API -> ServiceA "Routes"
  ServiceA -> EventQueue "Publishes"
  EventQueue -> ServiceB "Consumes"
  ServiceB -> Database "Stores"
}`,
  },
]

