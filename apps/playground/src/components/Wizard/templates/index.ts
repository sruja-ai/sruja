/**
 * Starter Templates
 * Curated architecture templates for quick-start
 */

import type { ArchitectureJSON } from "../../types";

export interface Template {
  id: string;
  name: string;
  category: "basic" | "intermediate" | "advanced";
  description: string;
  icon: string;
  architecture: ArchitectureJSON;
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
    metadata: {
      name: "Simple Web Application",
      version: "1.0.0",
      generated: new Date().toISOString(),
    },
    architecture: {
      persons: [{ id: "User", label: "End User" }],
      systems: [
        {
          id: "WebApp",
          label: "Web Application",
          containers: [
            { id: "Frontend", label: "Frontend", technology: "React" },
            { id: "API", label: "API Server", technology: "Node.js" },
          ],
          datastores: [{ id: "Database", label: "Database" }],
        },
      ],
      relations: [
        { from: "User", to: "WebApp.Frontend", label: "Uses" },
        { from: "WebApp.Frontend", to: "WebApp.API", verb: "calls" },
        { from: "WebApp.API", to: "WebApp.Database", verb: "reads/writes" },
      ],
    },
    navigation: { levels: ["L1", "L2"] },
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
    metadata: {
      name: "Microservices Architecture",
      version: "1.0.0",
      generated: new Date().toISOString(),
    },
    architecture: {
      persons: [{ id: "Customer", label: "Customer" }],
      systems: [
        {
          id: "Platform",
          label: "Platform",
          containers: [
            { id: "Gateway", label: "API Gateway", technology: "Kong" },
            { id: "UserService", label: "User Service", technology: "Go" },
            { id: "OrderService", label: "Order Service", technology: "Java" },
            { id: "PaymentService", label: "Payment Service", technology: "Go" },
          ],
          datastores: [
            { id: "UserDB", label: "User DB" },
            { id: "OrderDB", label: "Order DB" },
          ],
          queues: [{ id: "EventBus", label: "Event Bus" }],
        },
      ],
      relations: [
        { from: "Customer", to: "Platform.Gateway", label: "Uses" },
        { from: "Platform.Gateway", to: "Platform.UserService", verb: "routes" },
        { from: "Platform.Gateway", to: "Platform.OrderService", verb: "routes" },
        { from: "Platform.Gateway", to: "Platform.PaymentService", verb: "routes" },
        { from: "Platform.UserService", to: "Platform.UserDB", verb: "reads/writes" },
        { from: "Platform.OrderService", to: "Platform.OrderDB", verb: "reads/writes" },
        { from: "Platform.OrderService", to: "Platform.EventBus", verb: "publishes" },
        { from: "Platform.PaymentService", to: "Platform.EventBus", verb: "subscribes" },
      ],
      requirements: [
        { id: "R1", type: "performance", title: "API response <200ms", tags: ["Platform.Gateway"] },
        { id: "R2", type: "availability", title: "99.9% uptime", tags: ["Platform"] },
      ],
    },
    navigation: { levels: ["L1", "L2"] },
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
    metadata: {
      name: "Event-Driven Architecture",
      version: "1.0.0",
      generated: new Date().toISOString(),
    },
    architecture: {
      persons: [{ id: "User", label: "User" }],
      systems: [
        {
          id: "EventSystem",
          label: "Event System",
          containers: [
            { id: "Producer", label: "Event Producer", technology: "Python" },
            { id: "Consumer", label: "Event Consumer", technology: "Python" },
            { id: "Processor", label: "Stream Processor", technology: "Flink" },
          ],
          queues: [{ id: "EventBus", label: "Event Bus" }],
          datastores: [{ id: "EventStore", label: "Event Store" }],
        },
      ],
      relations: [
        { from: "User", to: "EventSystem.Producer", label: "Triggers" },
        {
          from: "EventSystem.Producer",
          to: "EventSystem.EventBus",
          verb: "publishes",
          interaction: "async",
        },
        {
          from: "EventSystem.Consumer",
          to: "EventSystem.EventBus",
          verb: "subscribes",
          interaction: "async",
        },
        {
          from: "EventSystem.Processor",
          to: "EventSystem.EventBus",
          verb: "consumes",
          interaction: "async",
        },
        { from: "EventSystem.Processor", to: "EventSystem.EventStore", verb: "writes" },
      ],
      adrs: [
        {
          id: "ADR001",
          title: "Use Event Sourcing",
          status: "accepted",
          context: "Need event replay capability",
          decision: "Store all events in immutable event store",
          tags: ["EventSystem.EventStore"],
        },
      ],
    },
    navigation: { levels: ["L1", "L2"] },
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
    metadata: { name: "Modular Monolith", version: "1.0.0", generated: new Date().toISOString() },
    architecture: {
      persons: [{ id: "User", label: "User" }],
      systems: [
        {
          id: "App",
          label: "Application",
          containers: [
            {
              id: "Monolith",
              label: "Monolith",
              technology: "Java, Spring Boot",
              components: [
                { id: "UserModule", label: "User Module" },
                { id: "OrderModule", label: "Order Module" },
                { id: "PaymentModule", label: "Payment Module" },
              ],
            },
          ],
          datastores: [{ id: "MainDB", label: "PostgreSQL" }],
        },
      ],
      relations: [
        { from: "User", to: "App.Monolith", label: "Uses" },
        { from: "App.Monolith.OrderModule", to: "App.Monolith.UserModule", verb: "calls" },
        { from: "App.Monolith.PaymentModule", to: "App.Monolith.OrderModule", verb: "calls" },
        { from: "App.Monolith", to: "App.MainDB", verb: "reads/writes" },
      ],
    },
    navigation: { levels: ["L1", "L2", "L3"] },
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
    metadata: { name: "SaaS Multi-Tenant", version: "1.0.0", generated: new Date().toISOString() },
    architecture: {
      persons: [
        { id: "Tenant", label: "Tenant User" },
        { id: "Admin", label: "Platform Admin" },
      ],
      systems: [
        {
          id: "SaaS",
          label: "SaaS Platform",
          containers: [
            { id: "Portal", label: "Tenant Portal", technology: "Next.js" },
            { id: "AdminPanel", label: "Admin Panel", technology: "React" },
            { id: "TenantAPI", label: "Tenant API", technology: "Node.js" },
            { id: "IdentityService", label: "Identity Service", technology: "Go" },
          ],
          datastores: [
            { id: "TenantDB", label: "Tenant Database" },
            { id: "ConfigDB", label: "Config Store" },
          ],
        },
      ],
      relations: [
        { from: "Tenant", to: "SaaS.Portal", label: "Uses" },
        { from: "Admin", to: "SaaS.AdminPanel", label: "Manages" },
        { from: "SaaS.Portal", to: "SaaS.TenantAPI", verb: "calls" },
        { from: "SaaS.TenantAPI", to: "SaaS.IdentityService", verb: "authenticates" },
        { from: "SaaS.TenantAPI", to: "SaaS.TenantDB", verb: "reads/writes" },
        { from: "SaaS.IdentityService", to: "SaaS.ConfigDB", verb: "reads" },
      ],
      requirements: [
        { id: "R1", type: "security", title: "Tenant data isolation", tags: ["SaaS.TenantDB"] },
        { id: "R2", type: "compliance", title: "GDPR compliance", tags: ["SaaS"] },
      ],
    },
    navigation: { levels: ["L1", "L2"] },
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
    metadata: { name: "New Architecture", version: "1.0.0", generated: new Date().toISOString() },
    architecture: {
      persons: [],
      systems: [],
      relations: [],
    },
    navigation: { levels: ["L1"] },
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
