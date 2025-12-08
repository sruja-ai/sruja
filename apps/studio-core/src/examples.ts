// Example DSL architectures for the Studio

export const EXAMPLES = {
    'Simple Web App': `architecture "Simple Web Application" {
  system WebApp "Web Application" {
    container API "API Service"
    container DB "Database"
    API -> DB "Validate User"
  }
  person User "End User"
  User -> WebApp.API "Credentials"
}`,

    'E-Commerce Platform': `architecture "E-Commerce Platform" {
  system Frontend "Customer Frontend" {
    container WebUI "Web Interface"
    container MobileApp "Mobile App"
  }
  
  system Backend "Backend Services" {
    container APIGateway "API Gateway"
    container OrderService "Order Service"
    container PaymentService "Payment Service"
    container InventoryService "Inventory Service"
    
    datastore OrderDB "Order Database"
    datastore PaymentDB "Payment Database"
    datastore InventoryDB "Inventory Database"
    
    APIGateway -> OrderService "Route Orders"
    APIGateway -> PaymentService "Process Payments"
    APIGateway -> InventoryService "Check Stock"
    
    OrderService -> OrderDB "Store Orders"
    PaymentService -> PaymentDB "Record Transactions"
    InventoryService -> InventoryDB "Update Stock"
  }
  
  person Customer "Customer"
  
  Customer -> Frontend.WebUI "Browse & Shop"
  Frontend.WebUI -> Backend.APIGateway "API Requests"
  Frontend.MobileApp -> Backend.APIGateway "API Requests"
}`,

    'Microservices Architecture': `architecture "Microservices Platform" {
  system Services "Microservices" {
    container UserService "User Service"
    container AuthService "Auth Service"
    container NotificationService "Notification Service"
    
    datastore UserDB "User DB"
    datastore AuthDB "Auth DB"
    
    queue EventBus "Event Bus"
    
    UserService -> UserDB "CRUD Users"
    AuthService -> AuthDB "Store Tokens"
    UserService -> EventBus "Publish Events"
    NotificationService -> EventBus "Subscribe Events"
  }
  
  person Admin "Administrator"
  person EndUser "End User"
  
  EndUser -> Services.AuthService "Login"
  Admin -> Services.UserService "Manage Users"
}`,

    'Event-Driven System': `architecture "Event-Driven Architecture" {
  system EventProcessing "Event Processing System" {
    container Producer "Event Producer"
    container Consumer1 "Analytics Consumer"
    container Consumer2 "Notification Consumer"
    
    queue EventStream "Event Stream"
    
    datastore AnalyticsDB "Analytics Database"
    datastore NotificationDB "Notification Log"
    
    Producer -> EventStream "Publish Events"
    EventStream -> Consumer1 "Stream Events"
    EventStream -> Consumer2 "Stream Events"
    Consumer1 -> AnalyticsDB "Store Analytics"
    Consumer2 -> NotificationDB "Log Notifications"
  }
  
  person DataAnalyst "Data Analyst"
  person User "System User"
  
  User -> EventProcessing.Producer "Trigger Events"
  DataAnalyst -> EventProcessing.AnalyticsDB "Query Data"
}`,

    'Three-Tier Application': `architecture "Three-Tier Application" {
  system PresentationTier "Presentation Layer" {
    container WebServer "Web Server"
  }
  
  system BusinessTier "Business Logic Layer" {
    container AppServer "Application Server"
    container CacheServer "Cache Server"
  }
  
  system DataTier "Data Layer" {
    datastore PrimaryDB "Primary Database"
    datastore SecondaryDB "Read Replica"
  }
  
  person User "Application User"
  
  User -> PresentationTier.WebServer "HTTP Requests"
  PresentationTier.WebServer -> BusinessTier.AppServer "API Calls"
  BusinessTier.AppServer -> BusinessTier.CacheServer "Cache Lookup"
  BusinessTier.AppServer -> DataTier.PrimaryDB "Write Data"
  BusinessTier.AppServer -> DataTier.SecondaryDB "Read Data"
}`
};

export type ExampleKey = keyof typeof EXAMPLES;
