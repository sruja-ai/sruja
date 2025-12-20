/**
 * C4 Node Classifier
 *
 * Classifies nodes by their semantic role in C4 diagrams:
 * - Person: User actors at the top of diagrams
 * - ExternalSystem/ExternalService: External dependencies (sides/bottom)
 * - DataStore/Database/Queue/Topic/Cache: Data layer (bottom)
 * - WebUI/API/Gateway: Presentation layer (top of container/component)
 * - Service/BFF: Business logic layer (middle)
 * - Repository: Data access layer (bottom of container/component)
 * - Component: Generic component (middle)
 */

export type C4SemanticRole =
  | "Person"
  | "ExternalSystem"
  | "ExternalService"
  | "ExternalContainer"
  | "ExternalComponent"
  | "DataStore"
  | "Database"
  | "Queue"
  | "Topic"
  | "Cache"
  | "FileSystem"
  | "WebUI"
  | "API"
  | "Gateway"
  | "BFF"
  | "Service"
  | "Repository"
  | "Controller"
  | "Component"
  | "Container"
  | "System"
  | "Unknown";

export type C4SemanticTier = "presentation" | "logic" | "data" | "external" | "actor" | "unknown";

export interface C4ClassificationResult {
  role: C4SemanticRole;
  tier: C4SemanticTier;
  rank: number;
  isExternal: boolean;
  isDataLayer: boolean;
  isPresentationLayer: boolean;
}

interface ClassifiableNode {
  id: string;
  original?: {
    kind?: string;
    type?: string;
    label?: string;
    tag?: string;
    tags?: string[];
    isExternal?: boolean;
    technology?: string;
  };
  type?: string;
}

const PERSON_PATTERNS = /^(person|user|customer|admin|actor|client|operator|developer|analyst)/i;
const DATABASE_PATTERNS =
  /\b(db|database|datastore|sql|mysql|postgres|postgresql|mongo|mongodb|redis|dynamo|dynamodb|cassandra|elastic|elasticsearch|mariadb|aurora|rds|bigquery|snowflake|cockroach|timescale|influx)\b/i;
const QUEUE_PATTERNS =
  /\b(queue|kafka|rabbit|rabbitmq|sqs|sns|pubsub|eventbridge|kinesis|nats|pulsar|activemq|zeromq)\b/i;
const TOPIC_PATTERNS = /\b(topic|event|stream|channel)\b/i;
const CACHE_PATTERNS = /\b(cache|redis|memcache|memcached|cdn|cloudfront|fastly|varnish)\b/i;
const FILESYSTEM_PATTERNS = /\b(filesystem|file|storage|s3|blob|gcs|bucket|disk|nas|nfs)\b/i;
const WEBUI_PATTERNS =
  /\b(web|webapp|webapplication|frontend|ui|spa|react|angular|vue|svelte|nextjs|remix|mobile|ios|android|app)\b/i;
const API_PATTERNS =
  /\b(api|rest|graphql|grpc|endpoint|restapi|apigateway|apigw|openapi|swagger)\b/i;
const GATEWAY_PATTERNS = /\b(gateway|ingress|loadbalancer|lb|proxy|nginx|envoy|kong|traefik)\b/i;
const BFF_PATTERNS = /\b(bff|backend.?for.?frontend)\b/i;
const SERVICE_PATTERNS =
  /\b(service|svc|worker|processor|handler|manager|coordinator|orchestrator|engine)\b/i;
const REPOSITORY_PATTERNS = /\b(repository|repo|dao|dal|dataaccess|store|persistence)\b/i;
const CONTROLLER_PATTERNS = /\b(controller|ctrl|handler|endpoint|route|router)\b/i;

export function classifyC4Node(node: ClassifiableNode): C4ClassificationResult {
  const original = node.original || {};
  const kind = original.kind || original.type || node.type || "";
  const label = original.label || node.id || "";
  const tag = original.tag || "";
  const tags = original.tags || [];
  const technology = original.technology || "";
  const isExternal = original.isExternal === true;

  const searchText = `${kind} ${label} ${tag} ${tags.join(" ")} ${technology}`.toLowerCase();

  if (kind === "Person" || PERSON_PATTERNS.test(kind) || PERSON_PATTERNS.test(label)) {
    return {
      role: "Person",
      tier: "actor",
      rank: 0,
      isExternal: false,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "ExternalSystem" || (kind === "SoftwareSystem" && isExternal)) {
    return {
      role: "ExternalSystem",
      tier: "external",
      rank: 60,
      isExternal: true,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "ExternalContainer" || (kind === "Container" && isExternal)) {
    return {
      role: "ExternalContainer",
      tier: "external",
      rank: 60,
      isExternal: true,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "ExternalComponent" || (kind === "Component" && isExternal)) {
    return {
      role: "ExternalComponent",
      tier: "external",
      rank: 60,
      isExternal: true,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (
    kind === "DataStore" ||
    kind === "Database" ||
    DATABASE_PATTERNS.test(searchText) ||
    tag === "datastore" ||
    tag === "database"
  ) {
    return {
      role: "DataStore",
      tier: "data",
      rank: 100,
      isExternal: isExternal,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (kind === "Queue" || QUEUE_PATTERNS.test(searchText) || tag === "queue") {
    return {
      role: "Queue",
      tier: "data",
      rank: 95,
      isExternal: isExternal,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (kind === "Topic" || TOPIC_PATTERNS.test(searchText) || tag === "topic") {
    return {
      role: "Topic",
      tier: "data",
      rank: 95,
      isExternal: isExternal,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (kind === "Cache" || CACHE_PATTERNS.test(searchText) || tag === "cache") {
    return {
      role: "Cache",
      tier: "data",
      rank: 90,
      isExternal: isExternal,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (kind === "FileSystem" || FILESYSTEM_PATTERNS.test(searchText) || tag === "filesystem") {
    return {
      role: "FileSystem",
      tier: "data",
      rank: 90,
      isExternal: isExternal,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (REPOSITORY_PATTERNS.test(label) || REPOSITORY_PATTERNS.test(tag)) {
    return {
      role: "Repository",
      tier: "data",
      rank: 90, // L3: Repositories at rank 90 (bottom lane)
      isExternal: false,
      isDataLayer: true,
      isPresentationLayer: false,
    };
  }

  if (CONTROLLER_PATTERNS.test(label) || CONTROLLER_PATTERNS.test(tag)) {
    return {
      role: "Controller",
      tier: "presentation",
      rank: 0, // L3: Controllers at rank 0 (top lane)
      isExternal: false,
      isDataLayer: false,
      isPresentationLayer: true,
    };
  }

  if (GATEWAY_PATTERNS.test(searchText)) {
    return {
      role: "Gateway",
      tier: "presentation",
      rank: 5,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: true,
    };
  }

  if (BFF_PATTERNS.test(searchText)) {
    return {
      role: "BFF",
      tier: "presentation",
      rank: 15,
      isExternal: false,
      isDataLayer: false,
      isPresentationLayer: true,
    };
  }

  if (API_PATTERNS.test(searchText)) {
    return {
      role: "API",
      tier: "presentation",
      rank: 10,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: true,
    };
  }

  if (WEBUI_PATTERNS.test(searchText)) {
    return {
      role: "WebUI",
      tier: "presentation",
      rank: 5,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: true,
    };
  }

  if (SERVICE_PATTERNS.test(label) || SERVICE_PATTERNS.test(tag)) {
    return {
      role: "Service",
      tier: "logic",
      rank: 40,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "SoftwareSystem" || kind === "System") {
    return {
      role: "System",
      tier: "logic",
      rank: 30,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "Container") {
    if (DATABASE_PATTERNS.test(searchText) || QUEUE_PATTERNS.test(searchText)) {
      return {
        role: "DataStore",
        tier: "data",
        rank: 100,
        isExternal: isExternal,
        isDataLayer: true,
        isPresentationLayer: false,
      };
    }
    if (WEBUI_PATTERNS.test(searchText) || API_PATTERNS.test(searchText)) {
      return {
        role: "Container",
        tier: "presentation",
        rank: 10,
        isExternal: isExternal,
        isDataLayer: false,
        isPresentationLayer: true,
      };
    }
    return {
      role: "Container",
      tier: "logic",
      rank: 40,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  if (kind === "Component") {
    return {
      role: "Component",
      tier: "logic",
      rank: 40,
      isExternal: isExternal,
      isDataLayer: false,
      isPresentationLayer: false,
    };
  }

  return {
    role: "Unknown",
    tier: "unknown",
    rank: 50,
    isExternal: isExternal,
    isDataLayer: false,
    isPresentationLayer: false,
  };
}

export function getSemanticRank(node: ClassifiableNode): number {
  return classifyC4Node(node).rank;
}

export function getSemanticTier(node: ClassifiableNode): C4SemanticTier {
  return classifyC4Node(node).tier;
}

export function isDataLayerNode(node: ClassifiableNode): boolean {
  return classifyC4Node(node).isDataLayer;
}

export function isPresentationLayerNode(node: ClassifiableNode): boolean {
  return classifyC4Node(node).isPresentationLayer;
}

export function isExternalNode(node: ClassifiableNode): boolean {
  return classifyC4Node(node).isExternal;
}

export function buildSemanticRankMap<T extends ClassifiableNode>(
  nodes: T[]
): Map<string, number> {
  const rankMap = new Map<string, number>();
  for (const node of nodes) {
    rankMap.set(node.id, getSemanticRank(node));
  }
  return rankMap;
}

export function groupNodesByTier<T extends ClassifiableNode>(
  nodes: T[]
): Map<C4SemanticTier, T[]> {
  const groups = new Map<C4SemanticTier, T[]>();
  for (const node of nodes) {
    const tier = getSemanticTier(node);
    const group = groups.get(tier) || [];
    group.push(node);
    groups.set(tier, group);
  }
  return groups;
}

export function getSameRankConstraints<T extends ClassifiableNode>(
  nodes: T[]
): string[][] {
  const tierGroups = groupNodesByTier(nodes);
  const sameRankGroups: string[][] = [];

  for (const [tier, group] of tierGroups) {
    if (tier !== "unknown" && group.length > 1) {
      sameRankGroups.push(group.map((n) => n.id));
    }
  }

  return sameRankGroups;
}
