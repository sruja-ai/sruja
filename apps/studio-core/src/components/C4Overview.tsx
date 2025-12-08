import React from 'react';

export function C4Overview() {
  return (
    <div className="pb-6 border-b border-[var(--color-border)] last:border-b-0">
      <div className="flex items-start justify-between mb-2">
        <h4 className="m-0 text-base font-semibold text-[var(--color-text-primary)]">C4 Model Overview</h4>
        <a
          href="https://c4model.com/"
          target="_blank"
          rel="noopener noreferrer"
          className="text-[var(--color-info-500)] hover:underline text-xs"
        >
          Official Docs
        </a>
      </div>
      <div className="text-sm leading-relaxed text-[var(--color-text-primary)]">
        <p className="mb-2">C4 is a hierarchical way to describe software architecture using three practical levels:</p>
        <p className="mb-1"><strong>L1 — System Context</strong>: define people (actors), systems, and high‑level relationships.</p>
        <ul className="list-disc list-inside text-xs text-[var(--color-text-secondary)] mb-2">
          <li>Add persons and systems</li>
          <li>Capture external dependencies</li>
          <li>Describe key flows</li>
        </ul>
        <p className="mb-1"><strong>L2 — Containers</strong>: decompose systems into deployable units (containers), datastores, and queues.</p>
        <ul className="list-disc list-inside text-xs text-[var(--color-text-secondary)] mb-2">
          <li>Identify services, UIs, APIs</li>
          <li>Model datastores and brokers</li>
          <li>Define container interactions</li>
        </ul>
        <p className="mb-1"><strong>L3 — Components</strong>: detail significant components inside containers and their collaborations.</p>
        <ul className="list-disc list-inside text-xs text-[var(--color-text-secondary)]">
          <li>Identify modules and responsibilities</li>
          <li>Show internal dependencies</li>
          <li>Keep components cohesive</li>
        </ul>
        <p className="mt-2 text-xs text-[var(--color-text-secondary)]">Use L1/L2/L3 controls in the toolbar and the Guide tab to track progress.</p>
      </div>
    </div>
  );
}
