import React from 'react';
import type { ArchitectureJSON } from '@sruja/viewer';

export function GuidePanel({ archData }: { archData: ArchitectureJSON | null }) {
  const arch = archData?.architecture;
  const persons = arch?.persons?.length || 0;
  const systems = arch?.systems?.length || 0;
  const containers = (arch?.systems || []).reduce((acc: number, s: any) => acc + (s.containers?.length || 0) + (s.datastores?.length || 0) + (s.queues?.length || 0), 0);
  const components = (arch?.systems || []).reduce((acc: number, s: any) => acc + (s.containers || []).reduce((a: number, c: any) => a + (c.components?.length || 0), 0), 0);

  const steps = [
    {
      title: 'Level 1: System Context',
      desc: 'Add actors (persons), systems, and relations between them.',
      ok: persons > 0 && systems > 0,
      metrics: [`Persons: ${persons}`, `Systems: ${systems}`],
    },
    {
      title: 'Level 2: Containers',
      desc: 'Decompose each system into containers, datastores, and queues.',
      ok: containers > 0,
      metrics: [`Containers/Stores/Queues: ${containers}`],
    },
    {
      title: 'Level 3: Components',
      desc: 'Decompose containers into components and refine internal interactions.',
      ok: components > 0,
      metrics: [`Components: ${components}`],
    },
  ];

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-4 min-h-0">
      <h4 className="m-0 text-sm font-semibold text-[var(--color-text-primary)]">Architecture Guide</h4>
      <p className="text-xs text-[var(--color-text-secondary)]">Progress through L1 → L2 → L3 to shape a robust architecture.</p>
      <div className="space-y-3">
        {steps.map((s, idx) => (
          <div key={idx} className="p-3 border border-[var(--color-border)] rounded-md bg-[var(--color-surface)]">
            <div className="flex items-start justify-between">
              <div>
                <div className="text-sm font-semibold text-[var(--color-text-primary)]">{s.title}</div>
                <div className="text-xs text-[var(--color-text-secondary)] mt-1">{s.desc}</div>
              </div>
              <div className={`text-xs font-semibold ${s.ok ? 'text-[var(--color-success-500)]' : 'text-[var(--color-warning-500)]'}`}>{s.ok ? 'Complete' : 'Pending'}</div>
            </div>
            <div className="mt-2 flex items-center gap-2">
              {s.metrics.map((m, i) => (
                <span key={i} className="text-xs px-2 py-1 rounded bg-[var(--color-primary-50)] text-[var(--color-text-secondary)]">{m}</span>
              ))}
            </div>
            <div className="mt-2 text-xs text-[var(--color-text-secondary)]">
              Use the L1/L2/L3 controls in the toolbar to focus each level.
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

