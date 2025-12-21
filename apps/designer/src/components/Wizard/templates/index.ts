/**
 * Starter Templates
 * Curated architecture templates for quick-start
 */

import type { SrujaModelDump } from "@sruja/shared";

// Helper to create metadata
const createMetadata = (name: string) => ({
  name,
  version: "1.0.0",
  generated: new Date().toISOString(),
  srujaVersion: "2.0.0"
});

// Helper to create a complete project structure
const createProject = (name: string) => ({
  id: name.toLowerCase().replace(/\s+/g, "-"),
  name,
});

export interface Template {
  id: string;
  name: string;
  category: "basic" | "intermediate" | "advanced";
  description: string;
  icon: string;
  architecture: SrujaModelDump; // Changed from ArchitectureJSON
}

/**
 * Simple 3-tier web application
 */
const simpleWebApp: Template = {
  id: "simple-web-app",
  name: "Simple Web App",
  category: "basic",
  description: "Classic 3-tier: User → Frontend → API → Database",
  icon: "🌐",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Simple Web Application"),
    projectId: "simple-web-application",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      "User": { id: "User" as any, kind: "person", title: "End User", tags: [], links: [], style: {} as any },
      "WebApp": { id: "WebApp" as any, kind: "system", title: "Web Application", tags: [], links: [], style: {} as any },
      "WebApp.Frontend": { id: "WebApp.Frontend" as any, kind: "container", title: "Frontend", technology: "React", tags: [], links: [], style: {} as any },
      "WebApp.API": { id: "WebApp.API" as any, kind: "container", title: "API Server", technology: "Node.js", tags: [], links: [], style: {} as any },
      "WebApp.Database": { id: "WebApp.Database" as any, kind: "datastore", title: "Database", tags: [], links: [], style: {} as any }
    },
    relations: [
      { id: "rel1" as any, source: { model: "User" }, target: { model: "WebApp.Frontend" }, title: "Uses" },
      { id: "rel2" as any, source: { model: "WebApp.Frontend" }, target: { model: "WebApp.API" }, title: "calls" },
      { id: "rel3" as any, source: { model: "WebApp.API" }, target: { model: "WebApp.Database" }, title: "reads/writes" }
    ],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: {
      requirements: [],
      flows: [],
      scenarios: [],
      adrs: [],
    },
    _metadata: createMetadata("Simple Web Application"),
  },
};

/**
 * Microservices with API Gateway
 */
const microservices: Template = {
  id: "microservices",
  name: "Microservices",
  category: "intermediate",
  description: "API Gateway with 3 services, message queue, and databases",
  icon: "🔗",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Microservices Architecture"),
    projectId: "microservices",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      "Customer": { id: "Customer" as any, kind: "person", title: "Customer", tags: [], links: [], style: {} as any },
      "Platform": { id: "Platform" as any, kind: "system", title: "Platform", tags: [], links: [], style: {} as any },
      "Platform.Gateway": { id: "Platform.Gateway" as any, kind: "container", title: "API Gateway", technology: "Kong", tags: [], links: [], style: {} as any },
      "Platform.UserService": { id: "Platform.UserService" as any, kind: "container", title: "User Service", technology: "Go", tags: [], links: [], style: {} as any },
      "Platform.OrderService": { id: "Platform.OrderService" as any, kind: "container", title: "Order Service", technology: "Java", tags: [], links: [], style: {} as any },
      "Platform.PaymentService": { id: "Platform.PaymentService" as any, kind: "container", title: "Payment Service", technology: "Go", tags: [], links: [], style: {} as any },
      "Platform.UserDB": { id: "Platform.UserDB" as any, kind: "datastore", title: "User DB", tags: [], links: [], style: {} as any },
      "Platform.OrderDB": { id: "Platform.OrderDB" as any, kind: "datastore", title: "Order DB", tags: [], links: [], style: {} as any },
      "Platform.EventBus": { id: "Platform.EventBus" as any, kind: "queue", title: "Event Bus", tags: [], links: [], style: {} as any },
    },
    relations: [
      { id: "rel4" as any, source: { model: "Customer" }, target: { model: "Platform.Gateway" }, title: "Uses" },
      { id: "rel5" as any, source: { model: "Platform.Gateway" }, target: { model: "Platform.UserService" }, title: "routes" },
      { id: "rel6" as any, source: { model: "Platform.Gateway" }, target: { model: "Platform.OrderService" }, title: "routes" },
      { id: "rel7" as any, source: { model: "Platform.Gateway" }, target: { model: "Platform.PaymentService" }, title: "routes" },
      { id: "rel8" as any, source: { model: "Platform.UserService" }, target: { model: "Platform.UserDB" }, title: "reads/writes" },
      { id: "rel9" as any, source: { model: "Platform.OrderService" }, target: { model: "Platform.OrderDB" }, title: "reads/writes" },
      { id: "rel10" as any, source: { model: "Platform.OrderService" }, target: { model: "Platform.EventBus" }, title: "publishes" },
      { id: "rel11" as any, source: { model: "Platform.PaymentService" }, target: { model: "Platform.EventBus" }, title: "subscribes" },
    ],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: {
      requirements: [
        { id: "R1", type: "performance", title: "API response <200ms" },
        { id: "R2", type: "availability", title: "99.9% uptime" },
      ],
      flows: [],
      scenarios: [],
      adrs: []
    },
    _metadata: createMetadata("Microservices Architecture"),
  },
};

/**
 * Event-Driven Architecture
 */
const eventDriven: Template = {
  id: "event-driven",
  name: "Event-Driven",
  category: "intermediate",
  description: "Producers, consumers, and event store with async messaging",
  icon: "⚡",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Event-Driven Architecture"),
    projectId: "event-driven",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      "User": { id: "User" as any, kind: "person", title: "User", tags: [], links: [], style: {} as any },
      "EventSystem": { id: "EventSystem" as any, kind: "system", title: "Event System", tags: [], links: [], style: {} as any },
      "EventSystem.Producer": { id: "EventSystem.Producer" as any, kind: "container", title: "Event Producer", technology: "Python", tags: [], links: [], style: {} as any },
      "EventSystem.Consumer": { id: "EventSystem.Consumer" as any, kind: "container", title: "Event Consumer", technology: "Python", tags: [], links: [], style: {} as any },
      "EventSystem.Processor": { id: "EventSystem.Processor" as any, kind: "container", title: "Stream Processor", technology: "Flink", tags: [], links: [], style: {} as any },
      "EventSystem.EventBus": { id: "EventSystem.EventBus" as any, kind: "queue", title: "Event Bus", tags: [], links: [], style: {} as any },
      "EventSystem.EventStore": { id: "EventSystem.EventStore" as any, kind: "datastore", title: "Event Store", tags: [], links: [], style: {} as any },
    },
    relations: [
      { id: "rel12" as any, source: { model: "User" }, target: { model: "EventSystem.Producer" }, title: "Triggers" },
      { id: "rel13" as any, source: { model: "EventSystem.Producer" }, target: { model: "EventSystem.EventBus" }, title: "publishes" },
      { id: "rel14" as any, source: { model: "EventSystem.Consumer" }, target: { model: "EventSystem.EventBus" }, title: "subscribes" },
      { id: "rel15" as any, source: { model: "EventSystem.Processor" }, target: { model: "EventSystem.EventBus" }, title: "consumes" },
      { id: "rel16" as any, source: { model: "EventSystem.Processor" }, target: { model: "EventSystem.EventStore" }, title: "writes" },
    ],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: {
      adrs: [
        {
          id: "ADR001",
          title: "Use Event Sourcing",
          status: "accepted",
          context: "Need event replay capability",
          decision: "Store all events in immutable event store",
        },
      ],
      requirements: [],
      flows: [],
      scenarios: []
    },
    _metadata: createMetadata("Event-Driven Architecture"),
  },
};

/**
 * Monolith with Modules
 */
const modularMonolith: Template = {
  id: "modular-monolith",
  name: "Modular Monolith",
  category: "basic",
  description: "Single deployable with well-separated modules",
  icon: "📦",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Modular Monolith"),
    projectId: "modular-monolith",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      "User": { id: "User" as any, kind: "person", title: "User", tags: [], links: [], style: {} as any },
      "App": { id: "App" as any, kind: "system", title: "Application", tags: [], links: [], style: {} as any },
      "App.Monolith": { id: "App.Monolith" as any, kind: "container", title: "Monolith", technology: "Java, Spring Boot", tags: [], links: [], style: {} as any },
      "App.Monolith.UserModule": { id: "App.Monolith.UserModule" as any, kind: "component", title: "User Module", tags: [], links: [], style: {} as any },
      "App.Monolith.OrderModule": { id: "App.Monolith.OrderModule" as any, kind: "component", title: "Order Module", tags: [], links: [], style: {} as any },
      "App.Monolith.PaymentModule": { id: "App.Monolith.PaymentModule" as any, kind: "component", title: "Payment Module", tags: [], links: [], style: {} as any },
      "App.MainDB": { id: "App.MainDB" as any, kind: "datastore", title: "PostgreSQL", tags: [], links: [], style: {} as any },
    },
    relations: [
      { id: "rel17" as any, source: { model: "User" }, target: { model: "App.Monolith" }, title: "Uses" },
      { id: "rel18" as any, source: { model: "App.Monolith.OrderModule" }, target: { model: "App.Monolith.UserModule" }, title: "calls" },
      { id: "rel19" as any, source: { model: "App.Monolith.PaymentModule" }, target: { model: "App.Monolith.OrderModule" }, title: "calls" },
      { id: "rel20" as any, source: { model: "App.Monolith" }, target: { model: "App.MainDB" }, title: "reads/writes" },
    ],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: { requirements: [], flows: [], scenarios: [], adrs: [] },
    _metadata: createMetadata("Modular Monolith"),
  },
};

/**
 * SaaS Multi-Tenant
 */
const saasMultiTenant: Template = {
  id: "saas-multi-tenant",
  name: "SaaS Multi-Tenant",
  category: "advanced",
  description: "Multi-tenant SaaS with tenant isolation and shared services",
  icon: "☁️",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("SaaS Multi-Tenant"),
    projectId: "saas-multi-tenant",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      "Tenant": { id: "Tenant" as any, kind: "person", title: "Tenant User", tags: [], links: [], style: {} as any },
      "Admin": { id: "Admin" as any, kind: "person", title: "Platform Admin", tags: [], links: [], style: {} as any },
      "SaaS": { id: "SaaS" as any, kind: "system", title: "SaaS Platform", tags: [], links: [], style: {} as any },
      "SaaS.Portal": { id: "SaaS.Portal" as any, kind: "container", title: "Tenant Portal", technology: "Next.js", tags: [], links: [], style: {} as any },
      "SaaS.AdminPanel": { id: "SaaS.AdminPanel" as any, kind: "container", title: "Admin Panel", technology: "React", tags: [], links: [], style: {} as any },
      "SaaS.TenantAPI": { id: "SaaS.TenantAPI" as any, kind: "container", title: "Tenant API", technology: "Node.js", tags: [], links: [], style: {} as any },
      "SaaS.IdentityService": { id: "SaaS.IdentityService" as any, kind: "container", title: "Identity Service", technology: "Go", tags: [], links: [], style: {} as any },
      "SaaS.TenantDB": { id: "SaaS.TenantDB" as any, kind: "datastore", title: "Tenant Database", tags: [], links: [], style: {} as any },
      "SaaS.ConfigDB": { id: "SaaS.ConfigDB" as any, kind: "datastore", title: "Config Store", tags: [], links: [], style: {} as any },
    },
    relations: [
      { id: "rel21" as any, source: { model: "Tenant" }, target: { model: "SaaS.Portal" }, title: "Uses" },
      { id: "rel22" as any, source: { model: "Admin" }, target: { model: "SaaS.AdminPanel" }, title: "Manages" },
      { id: "rel23" as any, source: { model: "SaaS.Portal" }, target: { model: "SaaS.TenantAPI" }, title: "calls" },
      { id: "rel24" as any, source: { model: "SaaS.TenantAPI" }, target: { model: "SaaS.IdentityService" }, title: "authenticates" },
      { id: "rel25" as any, source: { model: "SaaS.TenantAPI" }, target: { model: "SaaS.TenantDB" }, title: "reads/writes" },
      { id: "rel26" as any, source: { model: "SaaS.IdentityService" }, target: { model: "SaaS.ConfigDB" }, title: "reads" },
    ],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: {
      requirements: [
        { id: "R1", type: "security", title: "Tenant data isolation" },
        { id: "R2", type: "compliance", title: "GDPR compliance" },
      ],
      flows: [],
      scenarios: [],
      adrs: []
    },
    _metadata: createMetadata("SaaS Multi-Tenant"),
  },
};

/**
 * Empty Starter
 */
const emptyStarter: Template = {
  id: "empty",
  name: "Empty",
  category: "basic",
  description: "Start from scratch with a blank architecture",
  icon: "📝",
  architecture: {
    _stage: "parsed" as const,
    specification: { tags: {}, elements: {}, deployments: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("New Architecture"),
    projectId: "new-architecture",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {},
    relations: [],
    views: {
      index: {
        id: "index" as any,
        title: "Overview",
      } as any,
    },
    sruja: {},
    _metadata: createMetadata("New Architecture"),
  },
};

/**
 * All available templates
 */
export const templates: Template[] = [
  emptyStarter,
  simpleWebApp,
  modularMonolith,
  microservices,
  eventDriven,
  saasMultiTenant,
];

/**
 * Get template by ID
 */
export function getTemplateById(id: string): Template | undefined {
  return templates.find((t) => t.id === id);
}

/**
 * Get templates by category
 */
export function getTemplatesByCategory(category: Template["category"]): Template[] {
  return templates.filter((t) => t.category === category);
}
