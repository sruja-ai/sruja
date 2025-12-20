// packages/shared/src/builder/example-usage.ts
// Complete examples of using SrujaBuilder (Option 1 - Recommended)

import { SrujaBuilder } from "./likec4-adapter";

/**
 * Example 1: Compositional style with helpers (LikeC4 pattern)
 */
export function example1Compositional() {
  const {
    builder,
    model: { model, system, component, relTo },
    views: { views, view, $include },
  } = SrujaBuilder.forSpecification({
    elements: {
      system: {},
      container: {},
      component: {},
    },
    tags: ["frontend", "backend"],
  });

  const result = builder
    .with(
      model(
        system("cloud", "Cloud System").with(
          component("api", "API Server"),
          component("frontend", "Web Frontend").with(
            relTo("cloud.api", "Makes API calls")
          )
        )
      )
    )
    .with(
      views(
        view("index", "System Overview").with($include("*"))
      )
    )
    .build();

  return result;
}

/**
 * Example 2: Chainable style
 */
export function example2Chainable() {
  // Use object spec format for proper TypeScript inference
  const result = SrujaBuilder.specification({
    elements: {
      system: {},
      container: {},
      component: {},
    },
  } as any)
    .model((helpers: any, _: any) => {
      const { system, container, component, relTo } = helpers;
      return _(system("ecommerce", "E-commerce Platform").with(
        container("catalog", "Product Catalog"),
        container("cart", "Shopping Cart"),
        component("api", "REST API").with(
          relTo("ecommerce.catalog", "Queries"),
          relTo("ecommerce.cart", "Updates")
        )
      ));
    })
    .views(({ view, viewOf, $include }, _) =>
      _(
        view("overview", "System Overview").with($include("*")),
        viewOf("catalog" as any, "ecommerce.catalog" as any, "Catalog Details").with(
          $include("*")
        )
      )
    )
    .build();

  return result;
}

/**
 * Example 3: With deployment model
 */
export function example3WithDeployment() {
  const {
    builder,
    model: { model, system, component },
    deployment: { deployment, env, node, instanceOf },
  } = SrujaBuilder.forSpecification({
    elements: {
      system: {},
      component: {},
    },
    deployments: {
      env: {},
      node: {},
    },
  });

  const result = builder
    .with(
      model(
        system("app", "Application").with(
          component("api", "API Server")
        )
      )
    )
    .with(
      deployment(
        env("production", "Production").with(
          node("eu-west", "EU West").with(
            instanceOf("app.api", "API Instance")
          )
        )
      )
    )
    .build();

  return result;
}

/**
 * Example 4: Simple usage
 */
export function example4Simple() {
  // Use object spec format for proper TypeScript inference
  const result = SrujaBuilder.specification({
    elements: {
      system: {},
      container: {},
    },
  } as any)
    .model((helpers: any, _: any) => {
      const { system, container } = helpers;
      return _(system("mysystem", "My System").with(
        container("api", "API")
      ));
    })
    .build();

  // result is SrujaModelDump - ready to use with Sruja's architecture store
  return result;
}
