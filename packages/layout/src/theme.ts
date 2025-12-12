import type { C4Kind, C4Level } from './c4-model'

export interface C4Theme {
  fontFamily: string
  fontSize: Partial<Record<C4Kind | C4Level, number>>
  fontWeight: Partial<Record<C4Kind | C4Level, string>>
  padding: Partial<Record<C4Kind | C4Level, number>>
  margin: Partial<Record<C4Kind | C4Level, number>>
  textColor: string
  backgroundColor: string
}

export const DefaultTheme: C4Theme = {
  fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  fontSize: { Person: 14, SoftwareSystem: 16, Container: 14, Component: 12, Database: 14, landscape: 18, context: 16, container: 14, component: 12, deployment: 14 },
  fontWeight: { Person: 'bold', SoftwareSystem: 'bold', Container: '600', Component: 'normal', Database: '600' },
  padding: { Person: 16, SoftwareSystem: 24, Container: 20, Component: 12, Database: 16 },
  margin: { Person: 20, SoftwareSystem: 30, Container: 20, Component: 15, Database: 20 },
  textColor: '#1a1a1a',
  backgroundColor: '#ffffff'
}
