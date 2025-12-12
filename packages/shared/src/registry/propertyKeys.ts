export const propertyKeys: string[] = [
  'capacity.instanceType',
  'capacity.instanceProvider',
  'capacity.readReplicas',
  'capacity.cache',
  'obs.metrics.application',
  'obs.alerting.critical',
  'obs.logging.application',
  'obs.tracing.tool',
  'obs.tracing.sampleRate',
  'compliance.pci.level',
  'compliance.pci.status',
  'compliance.soc2.type',
  'compliance.gdpr.status',
  'cost.monthly.total',
  'cost.monthly.compute',
  'cost.perTransaction.average',
]

export function isInsideBlock(text: string, positionLine: number, block: 'properties' | 'metadata'): boolean {
  const lines = text.split(/\r?\n/)
  for (let i = positionLine - 1; i >= 0 && i >= positionLine - 50; i--) {
    const line = (lines[i] || '').trim()
    if (line.includes('{')) {
      const prev = (lines[i - 1] || '').toLowerCase()
      if (prev.includes(block)) return true
      // Also check same line pattern: properties { or metadata {
      if (line.toLowerCase().includes(`${block} {`)) return true
      return false
    }
  }
  return false
}
