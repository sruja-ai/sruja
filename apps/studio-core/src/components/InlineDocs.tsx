// apps/studio-core/src/components/InlineDocs.tsx
import React from 'react';
import { X, ExternalLink } from 'lucide-react';

interface InlineDocsProps {
  nodeType: string;
  nodeId?: string;
  nodeLabel?: string;
  isOpen: boolean;
  onClose: () => void;
  onOpenFullDocs: (url: string) => void;
}

const DOC_SUMMARIES: Record<string, { title: string; description: string; url: string }> = {
  person: {
    title: 'Person',
    description: 'A person represents a user or stakeholder in your architecture. People interact with systems through various means.',
    url: '/learn/docs/docs/concepts/person',
  },
  system: {
    title: 'System',
    description: 'A system is a software system that delivers value to its users. Systems can contain containers and interact with other systems.',
    url: '/learn/docs/docs/concepts/system',
  },
  container: {
    title: 'Container',
    description: 'A container is a deployable unit within a system. Examples include web applications, mobile apps, and microservices.',
    url: '/learn/docs/docs/concepts/container',
  },
  datastore: {
    title: 'Datastore',
    description: 'A datastore represents a database or data storage mechanism. It stores and retrieves data for containers.',
    url: '/learn/docs/docs/concepts/datastore',
  },
  queue: {
    title: 'Queue',
    description: 'A queue is an asynchronous messaging mechanism that allows containers to communicate asynchronously.',
    url: '/learn/docs/docs/concepts/queue',
  },
  requirement: {
    title: 'Requirement',
    description: 'A requirement represents a functional or non-functional requirement that the architecture must satisfy.',
    url: '/learn/docs/docs/concepts/relations',
  },
  adr: {
    title: 'ADR (Architecture Decision Record)',
    description: 'An ADR documents an important architectural decision made along with its context and consequences.',
    url: '/learn/docs/docs/concepts/adr',
  },
  deployment: {
    title: 'Deployment Node',
    description: 'A deployment node represents the infrastructure where containers are deployed, such as servers or cloud regions.',
    url: '/learn/docs/docs/concepts/deployment',
  },
  component: {
    title: 'Component',
    description: 'A component is a logical grouping of functionality within a container, typically representing a module or service.',
    url: '/learn/docs/docs/concepts/component',
  },
};

export const InlineDocs: React.FC<InlineDocsProps> = ({
  nodeType,
  nodeId,
  nodeLabel,
  isOpen,
  onClose,
  onOpenFullDocs,
}) => {
  if (!isOpen) return null;

  const doc = DOC_SUMMARIES[nodeType] || {
    title: nodeType,
    description: 'Documentation for this element type.',
    url: '/learn/docs/docs/concepts',
  };

  return (
    <div className="fixed bottom-4 right-4 w-96 max-w-[calc(100vw-2rem)] bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-xl z-50">
      <div className="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)] flex items-center justify-between">
        <h3 className="m-0 text-sm font-semibold text-[var(--color-text-primary)]">Documentation</h3>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-[var(--color-neutral-200)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors"
          title="Close"
        >
          <X size={16} />
        </button>
      </div>
      <div className="p-4">
        <div className="mb-3">
          <h4 className="m-0 text-base font-semibold text-[var(--color-text-primary)] mb-1">
            {doc.title}
          </h4>
          {nodeLabel && (
            <p className="m-0 text-xs text-[var(--color-text-secondary)] font-mono">
              {nodeId || nodeLabel}
            </p>
          )}
        </div>
        <p className="m-0 text-sm text-[var(--color-text-primary)] leading-relaxed mb-4">
          {doc.description}
        </p>
        <button
          onClick={() => onOpenFullDocs(doc.url)}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-[var(--color-primary)] text-[var(--color-background)] rounded-md text-sm font-medium hover:opacity-90 transition-opacity"
        >
          <ExternalLink size={14} />
          View Full Documentation
        </button>
      </div>
    </div>
  );
};

