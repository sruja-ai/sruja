// apps/website/src/shared/__tests__/fixtures/sample-dsl.ts

export const SIMPLE_DSL = `
specification {
  element person
  element system
  element container
  element database
}

model {
  user = person "User"
  
  app = system "My App" {
    web = container "Web Server"
    db = database "Database"
  }
  
  user -> app.web "uses"
  app.web -> app.db "stores data in"
}
`;

export const COMPLEX_DSL = `
specification {
  element person
  element system
  element container
  element database
}

model {
  customer = person "Customer"
  admin = person "Administrator"
  
  shop = system "Online Shop" {
    webApp = container "Web Application"
    api = container "API Server"
    catalog = database "Product Catalog"
    orders = database "Order Database"
  }
  
  payment = system "Payment Gateway" {
    paymentAPI = container "Payment API"
  }

  customer -> shop.webApp "browses"
  shop.webApp -> shop.api "calls"
  shop.api -> shop.catalog "reads from"
  shop.api -> shop.orders "writes to"
  shop.api -> payment.paymentAPI "processes payments via"
  admin -> shop.api "manages"
}
`;
