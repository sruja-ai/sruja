// apps/website/src/shared/utils/errors.ts

/**
 * Formats parser errors into a user-friendly array of messages
 */
export function formatParseError(message: string, dsl: string): string[] {
  const details: string[] = []
  const m = message || 'Parse error'
  details.push(m)
  const lines = dsl.split('\n')
  let lineNum: number | null = null
  let colNum: number | null = null
  const r1 = m.match(/line\s+(\d+)/i)
  const r2 = m.match(/:(\d+):(\d+)/)
  if (r1 && r1[1]) lineNum = parseInt(r1[1], 10)
  if (r2 && r2[1] && r2[2]) { lineNum = parseInt(r2[1], 10); colNum = parseInt(r2[2], 10) }
  if (lineNum && lineNum >= 1 && lineNum <= lines.length) {
    const start = Math.max(1, lineNum - 1)
    const end = Math.min(lines.length, lineNum + 1)
    for (let i = start; i <= end; i++) {
      const prefix = i === lineNum ? '>' : ' '
      const line = lines[i - 1]
      details.push(prefix + ' Line ' + i + ': ' + line)
    }
    if (colNum && colNum > 0 && lineNum >= 1 && lineNum <= lines.length) {
      const pointer = ' '.repeat(Math.max(0, ('> Line ' + lineNum + ': ').length + colNum - 1)) + '^'
      details.push(pointer)
    }
  }
  const lower = m.toLowerCase()
  if (lower.includes('unterminated string') || lower.includes('missing quote') || lower.includes('expected string')) {
    details.push('Hint: Wrap labels and descriptions in quotes')
  }
  if (lower.includes('unexpected eof') || lower.includes('missing closing brace') || lower.includes('expected }')) {
    details.push('Hint: Add a closing } for the last opened block')
  }
  if (lower.includes('unknown identifier') || lower.includes('undefined reference')) {
    details.push('Hint: Verify names are declared before use')
  }
  if (!details.some(d => d.startsWith('Hint:'))) {
    const hasBareRelation = lines.some(l => /->\s+[^\s]+\s+[^"\n]+$/.test(l))
    if (hasBareRelation) details.push('Hint: Relation labels should be quoted, e.g., Web -> API "Calls"')
  }
  return details
}
