/**
 * Initial DSL for the homepage live editor. Shared so Astro can render static fallback and client can hydrate the demo.
 */
export const HERO_INITIAL_DSL = [
  "import { * } from 'sruja.ai/stdlib'",
  "",
  'User = person "Customer"',
  'App = system "E-commerce Platform" {',
  '  Web = container "React App"',
  '  API = container "Node.js API"',
  '  DB = database "PostgreSQL"',
  "}",
  "",
  'User -> App.Web "Visits"',
  'App.Web -> App.API "Calls"',
  'App.API -> App.DB "Stores Data"',
  "",
  "view index {",
  "  include *",
  "}",
].join("\n");
