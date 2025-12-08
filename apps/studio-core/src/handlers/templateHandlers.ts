// apps/studio-core/src/handlers/templateHandlers.ts

/**
 * Template DSL definitions
 */
const TEMPLATES: Record<string, string> = {
  'web-app': `architecture WebApp {
  person Customer "Customer"
  system WebApp "Web Application" {
    container WebFrontend "Web Frontend" "React"
    container WebBackend "Web Backend" "Node.js"
    container Database "Database" "PostgreSQL"
  }
  relation Customer -> WebFrontend "Uses"
  relation WebFrontend -> WebBackend "API calls"
  relation WebBackend -> Database "Stores data"
}`,
  'ecommerce': `architecture ECommerce {
  person Customer "Customer"
  person Admin "Admin"
  system ECommerceSystem "E-Commerce System" {
    container WebApp "Web Application" "React"
    container API "API Server" "Node.js"
    container PaymentService "Payment Service" "Go"
    container InventoryService "Inventory Service" "Java"
    datastore ProductDB "Product Database" "PostgreSQL"
    datastore OrderDB "Order Database" "MongoDB"
  }
  relation Customer -> WebApp "Shops"
  relation Admin -> WebApp "Manages"
  relation WebApp -> API "Calls"
  relation API -> PaymentService "Processes payments"
  relation API -> InventoryService "Checks stock"
  relation API -> ProductDB "Queries"
  relation API -> OrderDB "Stores orders"
}`,
  'microservices': `architecture Microservices {
  system MicroservicesSystem "Microservices System" {
    container APIGateway "API Gateway" "Kong"
    container UserService "User Service" "Go"
    container OrderService "Order Service" "Java"
    container NotificationService "Notification Service" "Node.js"
    datastore UserDB "User Database" "PostgreSQL"
    datastore OrderDB "Order Database" "MongoDB"
  }
  relation APIGateway -> UserService "Routes"
  relation APIGateway -> OrderService "Routes"
  relation APIGateway -> NotificationService "Routes"
  relation UserService -> UserDB "Stores"
  relation OrderService -> OrderDB "Stores"
}`,
  'api-backend': `architecture APIBackend {
  person Developer "Developer"
  system APISystem "API System" {
    container APIServer "API Server" "Express.js"
    container AuthService "Auth Service" "Node.js"
    container DataService "Data Service" "Python"
    datastore AuthDB "Auth Database" "Redis"
    datastore DataDB "Data Database" "PostgreSQL"
  }
  relation Developer -> APIServer "Uses API"
  relation APIServer -> AuthService "Authenticates"
  relation APIServer -> DataService "Queries data"
  relation AuthService -> AuthDB "Stores tokens"
  relation DataService -> DataDB "Stores data"
}`,
};

/**
 * Handle applying a template
 */
export function createHandleApplyTemplate({
  updateDsl,
  setToast,
}: {
  updateDsl: (dsl: string) => Promise<void>;
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void;
}) {
  return async (templateId: string) => {
    const templateDsl = TEMPLATES[templateId];
    if (templateDsl) {
      await updateDsl(templateDsl);
      setToast({ message: `Applied ${templateId} template`, type: 'success' });
    }
  };
}



