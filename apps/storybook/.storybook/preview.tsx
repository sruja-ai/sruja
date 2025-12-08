import React from 'react'
import { ThemeProvider } from '../../../packages/ui/src/components/ThemeProvider'
import '../../../packages/ui/src/design-system/styles.css'
import { srujaLightTheme, srujaDarkTheme } from './theme'
import { INITIAL_VIEWPORTS, MINIMAL_VIEWPORTS } from 'storybook/viewport'

// Custom viewports for architecture diagrams
const architectureViewports = {
  smallMonitor: {
    name: 'Small Monitor',
    styles: { width: '1280px', height: '800px' },
    type: 'desktop' as const,
  },
  largeMonitor: {
    name: 'Large Monitor',
    styles: { width: '1920px', height: '1080px' },
    type: 'desktop' as const,
  },
  presentation: {
    name: 'Presentation (16:9)',
    styles: { width: '1600px', height: '900px' },
    type: 'desktop' as const,
  },
  tabletLandscape: {
    name: 'Tablet Landscape',
    styles: { width: '1024px', height: '768px' },
    type: 'tablet' as const,
  },
}

export const decorators = [
  (Story: any, context: any) => {
    const content = React.createElement(Story)
    const wrapped = (
      <div className="min-h-[50vh] px-6 py-6 bg-[var(--color-surface)] text-[var(--color-text-primary)]">
        <div className="max-w-4xl mx-auto">
          {content}
        </div>
      </div>
    )
    const isFullscreen = context?.parameters?.layout === 'fullscreen'
    return React.createElement(
      ThemeProvider,
      { defaultMode: 'system' },
      isFullscreen ? content : wrapped
    )
  },
]

export const parameters = {
  controls: {
    sort: 'alpha',
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
  options: {
    storySort: {
      order: ['Overview', 'Components', 'Patterns', 'Examples'],
    },
  },
  layout: 'padded',
  backgrounds: {
    default: 'Surface',
    values: [
      { name: 'Surface', value: '#f8fafc' },
      { name: 'Background', value: '#ffffff' },
      { name: 'Dark Surface', value: '#1e293b' },
      { name: 'Dark Background', value: '#0f172a' },
    ],
  },
  docs: {
    theme: srujaLightTheme,
  },
  // Viewport configuration with full device set + custom viewports
  viewport: {
    options: {
      ...INITIAL_VIEWPORTS,
      ...architectureViewports,
    },
  },
  // Accessibility configuration (axe-core via Storybook a11y addon)
  a11y: {
    config: {
      rules: [
        { id: 'region', enabled: false }
      ],
    },
    options: {
      runOnly: {
        type: 'tag',
        values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']
      }
    },
    test: 'todo'
  }
}

// Set initial default viewport key; users can change via toolbar
export const initialGlobals = {
  viewport: { value: 'desktop', isRotated: false },
}

export const tags = ['autodocs']
