import { Size } from './types'
import { C4Kind, C4Level } from './c4-model'
import { DefaultTheme, C4Theme } from './theme'
import { TextMeasurer } from './utils/text-measurer'
import { CanvasTextMeasurer } from './utils/canvas-text-measurer'
import { CachedTextMeasurer } from './utils/cached-text-measurer'

export interface C4LayoutOptions {
  direction: 'TB' | 'LR' | 'RL'
  alignment: 'start' | 'center' | 'end' | 'justify'
  spacing: {
    node: Partial<Record<C4Kind | C4Level, number>>
    rank: Partial<Record<C4Kind | C4Level | string, number>>
    padding: Partial<Record<C4Kind | C4Level, number>>
    port: number
  }
  minSize: Size
  maxSize: Size
  aspectRatioLimits: { min: number; max: number }
  edgeRouting: { algorithm: 'orthogonal' | 'polyline' | 'splines'; bendPenalty: number; crossingPenalty: number; segmentLength: number; avoidNodes: boolean; preferOrthogonal: boolean }
  overlapRemoval: { enabled: boolean; algorithm: 'force' | 'shift'; iterations: number; tolerance: number; padding: number }
  beautify: { alignNodes: boolean; straightenEdges: boolean; balanceTree: boolean; compactGroups: boolean; removeOverlaps: boolean }
  maxIterations: number
  tolerance: number
  useGPU: boolean
  measurer: TextMeasurer
  theme: C4Theme
}

export const PublicationPreset: C4LayoutOptions = {
  direction: 'TB',
  alignment: 'center',
  spacing: { node: { SoftwareSystem: 24 }, rank: { Container: 60 }, padding: { SoftwareSystem: 24, Container: 16 }, port: 10 },
  minSize: { width: 80, height: 40 },
  maxSize: { width: 800, height: 600 },
  aspectRatioLimits: { min: 1, max: 2 },
  edgeRouting: { algorithm: 'orthogonal', bendPenalty: 1, crossingPenalty: 10, segmentLength: 20, avoidNodes: true, preferOrthogonal: true },
  overlapRemoval: { enabled: true, algorithm: 'shift', iterations: 10, tolerance: 1, padding: 8 },
  beautify: { alignNodes: true, straightenEdges: true, balanceTree: true, compactGroups: true, removeOverlaps: true },
  maxIterations: 10,
  tolerance: 0.01,
  useGPU: false,
  measurer: new CachedTextMeasurer(new CanvasTextMeasurer()),
  theme: DefaultTheme
}

export const InteractivePreset: C4LayoutOptions = {
  ...PublicationPreset,
  spacing: { node: { SoftwareSystem: 30 }, rank: { Container: 80 }, padding: { SoftwareSystem: 28, Container: 20 }, port: 12 }
}

export const PresentationPreset: C4LayoutOptions = {
  ...PublicationPreset,
  spacing: { node: { SoftwareSystem: 40 }, rank: { Container: 100 }, padding: { SoftwareSystem: 32, Container: 24 }, port: 14 }
}

export const CompactPreset: C4LayoutOptions = {
  ...PublicationPreset,
  spacing: { node: { SoftwareSystem: 16 }, rank: { Container: 40 }, padding: { SoftwareSystem: 16, Container: 12 }, port: 8 }
}
