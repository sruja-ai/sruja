// apps/designer/src/data/featureTemplates.ts

/**
 * Feature templates define common product features and their architectural requirements.
 * Each template maps a feature to required components, optional components, and related requirements.
 */
export interface FeatureTemplate {
  id: string;
  name: string;
  description: string;
  category: "ecommerce" | "saas" | "platform" | "api" | "data" | "auth" | "other";
  requiredComponents: ComponentRequirement[];
  optionalComponents: ComponentRequirement[];
  requirements: string[]; // Requirement IDs that should be covered
  icon?: string;
}

export interface ComponentRequirement {
  type: "system" | "container" | "component" | "datastore" | "queue" | "service";
  name: string;
  description: string;
  technology?: string; // Suggested technology
}

/**
 * Pre-defined feature templates for common product features.
 */
export const FEATURE_TEMPLATES: FeatureTemplate[] = [
  {
    id: "product-search",
    name: "Product Search",
    description: "Full-text search functionality for products with filters and sorting",
    category: "ecommerce",
    requiredComponents: [
      {
        type: "component",
        name: "SearchService",
        description: "Handles search queries and results",
        technology: "Elasticsearch, Algolia, or similar",
      },
      {
        type: "datastore",
        name: "SearchIndex",
        description: "Search index for products",
        technology: "Elasticsearch",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "SearchAnalytics",
        description: "Tracks search queries and results",
      },
    ],
    requirements: ["R1", "R2"], // Example requirement IDs
  },
  {
    id: "shopping-cart",
    name: "Shopping Cart",
    description: "Shopping cart functionality for managing items before checkout",
    category: "ecommerce",
    requiredComponents: [
      {
        type: "component",
        name: "CartService",
        description: "Manages cart operations (add, remove, update)",
      },
      {
        type: "datastore",
        name: "CartStore",
        description: "Stores cart data",
        technology: "Redis or Database",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "CartSync",
        description: "Syncs cart across devices",
      },
    ],
    requirements: [],
  },
  {
    id: "checkout",
    name: "Checkout",
    description: "Payment processing and order creation",
    category: "ecommerce",
    requiredComponents: [
      {
        type: "component",
        name: "PaymentService",
        description: "Processes payments",
        technology: "Stripe, PayPal, or similar",
      },
      {
        type: "component",
        name: "OrderService",
        description: "Creates and manages orders",
      },
      {
        type: "datastore",
        name: "OrderDB",
        description: "Stores order data",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "PaymentGateway",
        description: "Third-party payment gateway integration",
      },
    ],
    requirements: [],
  },
  {
    id: "user-authentication",
    name: "User Authentication",
    description: "User login, registration, and session management",
    category: "auth",
    requiredComponents: [
      {
        type: "component",
        name: "AuthService",
        description: "Handles authentication logic",
        technology: "OAuth2, JWT, or similar",
      },
      {
        type: "datastore",
        name: "UserDB",
        description: "Stores user credentials and profiles",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "SSOProvider",
        description: "Single Sign-On provider integration",
      },
    ],
    requirements: [],
  },
  {
    id: "api-gateway",
    name: "API Gateway",
    description: "Centralized API entry point with routing, rate limiting, and authentication",
    category: "api",
    requiredComponents: [
      {
        type: "container",
        name: "APIGateway",
        description: "API gateway service",
        technology: "Kong, AWS API Gateway, or similar",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "RateLimiter",
        description: "Rate limiting service",
      },
      {
        type: "component",
        name: "APIKeyManager",
        description: "Manages API keys and authentication",
      },
    ],
    requirements: [],
  },
  {
    id: "data-analytics",
    name: "Data Analytics",
    description: "Collect, process, and analyze user behavior and system metrics",
    category: "data",
    requiredComponents: [
      {
        type: "component",
        name: "AnalyticsService",
        description: "Processes analytics events",
      },
      {
        type: "datastore",
        name: "AnalyticsDB",
        description: "Stores analytics data",
        technology: "Data warehouse or time-series DB",
      },
    ],
    optionalComponents: [
      {
        type: "component",
        name: "ETLService",
        description: "Extract, Transform, Load service",
      },
    ],
    requirements: [],
  },
];

/**
 * Get feature templates by category.
 */
export function getFeatureTemplatesByCategory(
  category?: FeatureTemplate["category"]
): FeatureTemplate[] {
  if (!category) return FEATURE_TEMPLATES;
  return FEATURE_TEMPLATES.filter((t) => t.category === category);
}

/**
 * Get a feature template by ID.
 */
export function getFeatureTemplate(id: string): FeatureTemplate | undefined {
  return FEATURE_TEMPLATES.find((t) => t.id === id);
}
