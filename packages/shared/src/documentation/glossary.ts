// packages/shared/src/documentation/glossary.ts
// Glossary of Sruja concepts - single source of truth for concept summaries
// This file is used across studio and learn apps

export interface ConceptGlossaryEntry {
  id: string;
  title: string;
  summary: string;
  description: string;
  url: string;
  examples?: string[];
  keyPoints?: string[];
}

export const CONCEPT_GLOSSARY: ConceptGlossaryEntry[] = [
  {
    id: 'person',
    title: 'Person',
    summary: 'A Person represents a human user of your software system.',
    description: 'A **Person** represents a human user of your software system (e.g., "Customer", "Admin", "Employee"). Persons are typically shown at the top level of your architecture and represent end users, administrators, external stakeholders, or any human actor that interacts with your architecture.',
    url: '/docs/concepts/person',
    examples: [
      'person Customer "Bank Customer"',
      'person Admin "System Administrator"',
    ],
    keyPoints: [
      'Represents human users or stakeholders',
      'Top-level element in architecture diagrams',
      'Can have relationships with systems',
    ],
  },
  {
    id: 'system',
    title: 'System',
    summary: 'A System represents a software system, the highest level of abstraction in the C4 model.',
    description: 'A **System** represents a software system, which is the highest level of abstraction in the C4 model. A system delivers value to its users, whether they are human or other systems. Systems represent complete software applications that deliver value to users, can contain containers (Level 2), interact with other systems, and have clear boundaries.',
    url: '/docs/concepts/system',
    examples: [
      'system BankingSystem "Internet Banking System"',
      'system WebApp "Web Application"',
    ],
    keyPoints: [
      'Highest level of abstraction (C4 Level 1)',
      'Delivers value to users',
      'Can contain containers',
      'Interacts with other systems',
    ],
  },
  {
    id: 'container',
    title: 'Container',
    summary: 'A Container represents an application or a data store.',
    description: 'A **Container** represents an application or a data store. It is something that needs to be running in order for the overall software system to work. Containers are deployable units such as web applications, mobile applications, desktop applications, microservices, databases, and file systems.',
    url: '/docs/concepts/container',
    examples: [
      'container API "REST API"',
      'container WebApp "Web Application"',
    ],
    keyPoints: [
      'Deployable unit (C4 Level 2)',
      'Part of a system',
      'Can contain components',
      'Interacts with other containers',
    ],
  },
  {
    id: 'datastore',
    title: 'Datastore',
    summary: 'A Datastore represents a database or data storage mechanism.',
    description: 'A **Datastore** represents a database or data storage mechanism within a system. Datastores store and retrieve data for containers, including relational databases (PostgreSQL, MySQL), NoSQL databases (MongoDB, DynamoDB), file systems, and object storage.',
    url: '/docs/concepts/datastore',
    examples: [
      'datastore DB "PostgreSQL Database"',
      'datastore Cache "Redis Cache"',
    ],
    keyPoints: [
      'Stores and retrieves data',
      'Used by containers',
      'Part of a system',
    ],
  },
  {
    id: 'queue',
    title: 'Queue',
    summary: 'A Queue is an asynchronous messaging mechanism.',
    description: 'A **Queue** is an asynchronous messaging mechanism that allows containers to communicate asynchronously. Queues enable asynchronous communication, decoupling of services, event-driven architectures, and message buffering.',
    url: '/docs/concepts/queue',
    examples: [
      'queue EventQueue "Kafka Message Queue"',
      'queue TaskQueue "Background Task Queue"',
    ],
    keyPoints: [
      'Asynchronous messaging',
      'Decouples services',
      'Enables event-driven architecture',
    ],
  },
  {
    id: 'component',
    title: 'Component',
    summary: 'A Component is a logical grouping of functionality within a container.',
    description: 'A **Component** is a logical grouping of functionality within a container (C4 Level 3). Components represent modules within an application, services within a microservice, logical groupings of code, and reusable functionality.',
    url: '/docs/concepts/component',
    examples: [
      'component AuthService "Authentication Service"',
      'component PaymentProcessor "Payment Processing Module"',
    ],
    keyPoints: [
      'Logical grouping (C4 Level 3)',
      'Part of containers',
      'Can interact with other components',
    ],
  },
  {
    id: 'requirement',
    title: 'Requirement',
    summary: 'A Requirement represents a functional or non-functional requirement.',
    description: 'A **Requirement** represents a functional or non-functional requirement that the architecture must satisfy. Requirements document functional requirements, non-functional requirements (performance, security, etc.), constraints, and quality attributes.',
    url: '/docs/concepts/relations',
    examples: [
      'requirement R1 "High Availability"',
      'requirement R2 "Data Encryption"',
    ],
    keyPoints: [
      'Functional and non-functional requirements',
      'Constraints and quality attributes',
      'Can be satisfied by systems or containers',
    ],
  },
  {
    id: 'adr',
    title: 'ADR (Architecture Decision Record)',
    summary: 'An ADR documents an important architectural decision.',
    description: 'An **ADR** (Architecture Decision Record) documents an important architectural decision made along with its context and consequences. ADRs capture the decision made, context and constraints, consequences (positive and negative), and alternatives considered.',
    url: '/docs/concepts/adr',
    examples: [
      'adr ADR001 "Use Microservices Architecture"',
      'adr ADR002 "Database Selection"',
    ],
    keyPoints: [
      'Documents architectural decisions',
      'Captures context and consequences',
      'Can be related to systems or containers',
    ],
  },
  {
    id: 'deployment',
    title: 'Deployment Node',
    summary: 'A Deployment Node represents infrastructure where containers are deployed.',
    description: 'A **Deployment Node** represents the infrastructure where containers are deployed. Deployment nodes represent physical servers, virtual machines, cloud regions, and container orchestration platforms (Kubernetes).',
    url: '/docs/concepts/deployment',
    examples: [
      'deployment Production "AWS us-east-1"',
      'deployment Staging "Kubernetes Cluster"',
    ],
    keyPoints: [
      'Represents infrastructure',
      'Hosts containers',
      'Can be part of deployment diagrams',
    ],
  },
];

// Helper functions
export function getConceptById(id: string): ConceptGlossaryEntry | undefined {
  return CONCEPT_GLOSSARY.find(entry => entry.id === id);
}

export function getAllConcepts(): ConceptGlossaryEntry[] {
  return CONCEPT_GLOSSARY;
}

export function getConceptUrl(id: string): string {
  const concept = getConceptById(id);
  return concept?.url || '/docs/concepts';
}

