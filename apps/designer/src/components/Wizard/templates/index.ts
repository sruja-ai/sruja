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
  srujaVersion: "2.0.0",
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Simple Web Application"),
    projectId: "simple-web-application",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      User: {
        id: "User",
        kind: "person",
        title: "End User",
        tags: [],
        links: [],
        style: {},
      },
      WebApp: {
        id: "WebApp",
        kind: "system",
        title: "Web Application",
        tags: [],
        links: [],
        style: {},
      },
      "WebApp.Frontend": {
        id: "WebApp.Frontend",
        kind: "container",
        title: "Frontend",
        technology: "React",
        tags: [],
        links: [],
        style: {},
      },
      "WebApp.API": {
        id: "WebApp.API",
        kind: "container",
        title: "API Server",
        technology: "Node.js",
        tags: [],
        links: [],
        style: {},
      },
      "WebApp.Database": {
        id: "WebApp.Database",
        kind: "datastore",
        title: "Database",
        tags: [],
        links: [],
        style: {},
      },
    },
    relations: [
      {
        id: "rel1",
        source: { model: "User" },
        target: { model: "WebApp.Frontend" },
        title: "Uses",
      },
      {
        id: "rel2",
        source: { model: "WebApp.Frontend" },
        target: { model: "WebApp.API" },
        title: "calls",
      },
      {
        id: "rel3",
        source: { model: "WebApp.API" },
        target: { model: "WebApp.Database" },
        title: "reads/writes",
      },
    ],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Microservices Architecture"),
    projectId: "microservices",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      Customer: {
        id: "Customer",
        kind: "person",
        title: "Customer",
        tags: [],
        links: [],
        style: {},
      },
      Platform: {
        id: "Platform",
        kind: "system",
        title: "Platform",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.Gateway": {
        id: "Platform.Gateway",
        kind: "container",
        title: "API Gateway",
        technology: "Kong",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.UserService": {
        id: "Platform.UserService",
        kind: "container",
        title: "User Service",
        technology: "Go",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.OrderService": {
        id: "Platform.OrderService",
        kind: "container",
        title: "Order Service",
        technology: "Java",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.PaymentService": {
        id: "Platform.PaymentService",
        kind: "container",
        title: "Payment Service",
        technology: "Go",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.UserDB": {
        id: "Platform.UserDB",
        kind: "datastore",
        title: "User DB",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.OrderDB": {
        id: "Platform.OrderDB",
        kind: "datastore",
        title: "Order DB",
        tags: [],
        links: [],
        style: {},
      },
      "Platform.EventBus": {
        id: "Platform.EventBus",
        kind: "queue",
        title: "Event Bus",
        tags: [],
        links: [],
        style: {},
      },
    },
    relations: [
      {
        id: "rel4",
        source: { model: "Customer" },
        target: { model: "Platform.Gateway" },
        title: "Uses",
      },
      {
        id: "rel5",
        source: { model: "Platform.Gateway" },
        target: { model: "Platform.UserService" },
        title: "routes",
      },
      {
        id: "rel6",
        source: { model: "Platform.Gateway" },
        target: { model: "Platform.OrderService" },
        title: "routes",
      },
      {
        id: "rel7",
        source: { model: "Platform.Gateway" },
        target: { model: "Platform.PaymentService" },
        title: "routes",
      },
      {
        id: "rel8",
        source: { model: "Platform.UserService" },
        target: { model: "Platform.UserDB" },
        title: "reads/writes",
      },
      {
        id: "rel9",
        source: { model: "Platform.OrderService" },
        target: { model: "Platform.OrderDB" },
        title: "reads/writes",
      },
      {
        id: "rel10",
        source: { model: "Platform.OrderService" },
        target: { model: "Platform.EventBus" },
        title: "publishes",
      },
      {
        id: "rel11",
        source: { model: "Platform.PaymentService" },
        target: { model: "Platform.EventBus" },
        title: "subscribes",
      },
    ],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
    },
    sruja: {
      requirements: [
        { id: "R1", type: "performance", title: "API response <200ms" },
        { id: "R2", type: "availability", title: "99.9% uptime" },
      ],
      flows: [],
      scenarios: [],
      adrs: [],
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Event-Driven Architecture"),
    projectId: "event-driven",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      User: {
        id: "User",
        kind: "person",
        title: "User",
        tags: [],
        links: [],
        style: {},
      },
      EventSystem: {
        id: "EventSystem",
        kind: "system",
        title: "Event System",
        tags: [],
        links: [],
        style: {},
      },
      "EventSystem.Producer": {
        id: "EventSystem.Producer",
        kind: "container",
        title: "Event Producer",
        technology: "Python",
        tags: [],
        links: [],
        style: {},
      },
      "EventSystem.Consumer": {
        id: "EventSystem.Consumer",
        kind: "container",
        title: "Event Consumer",
        technology: "Python",
        tags: [],
        links: [],
        style: {},
      },
      "EventSystem.Processor": {
        id: "EventSystem.Processor",
        kind: "container",
        title: "Stream Processor",
        technology: "Flink",
        tags: [],
        links: [],
        style: {},
      },
      "EventSystem.EventBus": {
        id: "EventSystem.EventBus",
        kind: "queue",
        title: "Event Bus",
        tags: [],
        links: [],
        style: {},
      },
      "EventSystem.EventStore": {
        id: "EventSystem.EventStore",
        kind: "datastore",
        title: "Event Store",
        tags: [],
        links: [],
        style: {},
      },
    },
    relations: [
      {
        id: "rel12",
        source: { model: "User" },
        target: { model: "EventSystem.Producer" },
        title: "Triggers",
      },
      {
        id: "rel13",
        source: { model: "EventSystem.Producer" },
        target: { model: "EventSystem.EventBus" },
        title: "publishes",
      },
      {
        id: "rel14",
        source: { model: "EventSystem.Consumer" },
        target: { model: "EventSystem.EventBus" },
        title: "subscribes",
      },
      {
        id: "rel15",
        source: { model: "EventSystem.Processor" },
        target: { model: "EventSystem.EventBus" },
        title: "consumes",
      },
      {
        id: "rel16",
        source: { model: "EventSystem.Processor" },
        target: { model: "EventSystem.EventStore" },
        title: "writes",
      },
    ],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
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
      scenarios: [],
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("Modular Monolith"),
    projectId: "modular-monolith",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      User: {
        id: "User",
        kind: "person",
        title: "User",
        tags: [],
        links: [],
        style: {},
      },
      App: {
        id: "App",
        kind: "system",
        title: "Application",
        tags: [],
        links: [],
        style: {},
      },
      "App.Monolith": {
        id: "App.Monolith",
        kind: "container",
        title: "Monolith",
        technology: "Java, Spring Boot",
        tags: [],
        links: [],
        style: {},
      },
      "App.Monolith.UserModule": {
        id: "App.Monolith.UserModule",
        kind: "component",
        title: "User Module",
        tags: [],
        links: [],
        style: {},
      },
      "App.Monolith.OrderModule": {
        id: "App.Monolith.OrderModule",
        kind: "component",
        title: "Order Module",
        tags: [],
        links: [],
        style: {},
      },
      "App.Monolith.PaymentModule": {
        id: "App.Monolith.PaymentModule",
        kind: "component",
        title: "Payment Module",
        tags: [],
        links: [],
        style: {},
      },
      "App.MainDB": {
        id: "App.MainDB",
        kind: "datastore",
        title: "PostgreSQL",
        tags: [],
        links: [],
        style: {},
      },
    },
    relations: [
      {
        id: "rel17",
        source: { model: "User" },
        target: { model: "App.Monolith" },
        title: "Uses",
      },
      {
        id: "rel18",
        source: { model: "App.Monolith.OrderModule" },
        target: { model: "App.Monolith.UserModule" },
        title: "calls",
      },
      {
        id: "rel19",
        source: { model: "App.Monolith.PaymentModule" },
        target: { model: "App.Monolith.OrderModule" },
        title: "calls",
      },
      {
        id: "rel20",
        source: { model: "App.Monolith" },
        target: { model: "App.MainDB" },
        title: "reads/writes",
      },
    ],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("SaaS Multi-Tenant"),
    projectId: "saas-multi-tenant",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {
      Tenant: {
        id: "Tenant",
        kind: "person",
        title: "Tenant User",
        tags: [],
        links: [],
        style: {},
      },
      Admin: {
        id: "Admin",
        kind: "person",
        title: "Platform Admin",
        tags: [],
        links: [],
        style: {},
      },
      SaaS: {
        id: "SaaS",
        kind: "system",
        title: "SaaS Platform",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.Portal": {
        id: "SaaS.Portal",
        kind: "container",
        title: "Tenant Portal",
        technology: "Next.js",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.AdminPanel": {
        id: "SaaS.AdminPanel",
        kind: "container",
        title: "Admin Panel",
        technology: "React",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.TenantAPI": {
        id: "SaaS.TenantAPI",
        kind: "container",
        title: "Tenant API",
        technology: "Node.js",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.IdentityService": {
        id: "SaaS.IdentityService",
        kind: "container",
        title: "Identity Service",
        technology: "Go",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.TenantDB": {
        id: "SaaS.TenantDB",
        kind: "datastore",
        title: "Tenant Database",
        tags: [],
        links: [],
        style: {},
      },
      "SaaS.ConfigDB": {
        id: "SaaS.ConfigDB",
        kind: "datastore",
        title: "Config Store",
        tags: [],
        links: [],
        style: {},
      },
    },
    relations: [
      {
        id: "rel21",
        source: { model: "Tenant" },
        target: { model: "SaaS.Portal" },
        title: "Uses",
      },
      {
        id: "rel22",
        source: { model: "Admin" },
        target: { model: "SaaS.AdminPanel" },
        title: "Manages",
      },
      {
        id: "rel23",
        source: { model: "SaaS.Portal" },
        target: { model: "SaaS.TenantAPI" },
        title: "calls",
      },
      {
        id: "rel24",
        source: { model: "SaaS.TenantAPI" },
        target: { model: "SaaS.IdentityService" },
        title: "authenticates",
      },
      {
        id: "rel25",
        source: { model: "SaaS.TenantAPI" },
        target: { model: "SaaS.TenantDB" },
        title: "reads/writes",
      },
      {
        id: "rel26",
        source: { model: "SaaS.IdentityService" },
        target: { model: "SaaS.ConfigDB" },
        title: "reads",
      },
    ],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
    },
    sruja: {
      requirements: [
        { id: "R1", type: "security", title: "Tenant data isolation" },
        { id: "R2", type: "compliance", title: "GDPR compliance" },
      ],
      flows: [],
      scenarios: [],
      adrs: [],
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
    specification: { tags: {}, elements: {}, relationships: {} },
    deployments: { elements: {}, relations: {} },
    project: createProject("New Architecture"),
    projectId: "new-architecture",
    globals: { predicates: {}, dynamicPredicates: {}, styles: {} },
    elements: {},
    relations: [],
    views: {
      index: {
        id: "index",
        title: "Overview",
      },
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
