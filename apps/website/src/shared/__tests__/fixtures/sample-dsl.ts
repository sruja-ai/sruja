// apps/website/src/shared/__tests__/fixtures/sample-dsl.ts

export const SIMPLE_DSL = `
architecture "Test System" {
    system App "My App" {
        container Web "Web Server"
        datastore DB "Database"
    }
    person User "User"
    User -> Web "Uses"
    Web -> DB "Stores data in"
}
`;

export const COMPLEX_DSL = `
architecture "E-commerce Platform" {
    system Shop "Online Shop" {
        container WebApp "Web Application"
        container API "API Server"
        datastore Catalog "Product Catalog"
        datastore Orders "Order Database"
    }
    
    system Payment "Payment Gateway" {
        container PaymentAPI "Payment API"
    }
    
    person Customer "Customer"
    person Admin "Administrator"

    Customer -> WebApp "Browses"
    WebApp -> API "Calls"
    API -> Catalog "Reads from"
    API -> Orders "Writes to"
    API -> PaymentAPI "Processes payments via"
    Admin -> API "Manages"
}
`;
