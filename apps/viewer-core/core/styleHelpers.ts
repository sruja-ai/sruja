import { Colors, getCssVar } from '@sruja/shared/utils/cssVars'

function getIconDataUri(svgContent: string): string {
  const cleaned = svgContent.replace(/\s+/g, ' ').trim()
  return `data:image/svg+xml,${encodeURIComponent(cleaned)}`
}

function getC4Icons() {
  const personIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>'
  )
  const systemIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line><line x1="7" y1="8" x2="7" y2="8.01"></line><line x1="12" y1="8" x2="12" y2="8.01"></line><line x1="17" y1="8" x2="17" y2="8.01"></line><line x1="7" y1="12" x2="17" y2="12"></line></svg>'
  )
  const containerIcon = getIconDataUri(
    '<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#334155" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>'
  )
  return { personIcon, systemIcon, containerIcon }
}

function getStyleValue(ele: any, key: string, defaultVal: any): any {
  try {
    const meta = ele.data('metadata')
    if (!meta || !Array.isArray(meta)) return defaultVal
    const entry = meta.find((m: any) => m && m.key === key)
    if (!entry) return defaultVal
    const value = entry.value
    return value != null && value !== '' ? value : defaultVal
  } catch (e) {
    return defaultVal
  }
}

function getIconFromMetadata(ele: any, defaultIcon: string): string {
  try {
    const iconName = getStyleValue(ele, 'style.icon', '')
    if (!iconName || typeof iconName !== 'string') {
      return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none'
    }
    const icons = getC4Icons()
    let result: string
    switch (iconName) {
      case 'person': result = icons.personIcon || defaultIcon || 'none'; break
      case 'system': result = icons.systemIcon || defaultIcon || 'none'; break
      case 'container': result = icons.containerIcon || defaultIcon || 'none'; break
      case 'database': result = getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>') || defaultIcon || 'none'; break
      case 'cloud': result = getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"></path></svg>') || defaultIcon || 'none'; break
      case 'server': result = getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="14" y="14" width="8" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>') || defaultIcon || 'none'; break
      case 'mobile': result = getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect><line x1="12" y1="18" x2="12.01" y2="18"></line></svg>') || defaultIcon || 'none'; break
      case 'browser': result = getIconDataUri('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="3" y1="9" x2="21" y2="9"></line><line x1="9" y1="21" x2="9" y2="9"></line></svg>') || defaultIcon || 'none'; break
      default: result = defaultIcon && defaultIcon !== '' ? defaultIcon : 'none'
    }
    return result && result !== '' ? result : 'none'
  } catch (e) {
    return defaultIcon && defaultIcon !== '' ? defaultIcon : 'none'
  }
}

function calculateNodeWidth(ele: any): number {
  const label = ele.data('label') || ''
  const type = ele.data('type')
  const charWidth = 7.5
  const textMaxWidth = type === 'person' ? 100 : (type === 'system' ? 110 : 120)
  const labelWidth = label.length * charWidth
  const textWidth = Math.min(labelWidth, textMaxWidth)
  let paddingX = 0
  switch (type) {
    case 'person':
      paddingX = 24
      break
    case 'system':
      paddingX = 66
      break
    case 'container':
      const iconName = getStyleValue(ele, 'style.icon', '')
      const hasIcon = iconName && typeof iconName === 'string' && iconName !== ''
      paddingX = hasIcon ? 52 : 24
      break
    case 'datastore':
    case 'queue':
      paddingX = 24
      break
    default:
      paddingX = 24
  }
  const minWidth = type === 'system' ? 120 : 80
  return Math.max(minWidth, textWidth + paddingX)
}

function calculateNodeHeight(ele: any): number {
  const label = ele.data('label') || ''
  const type = ele.data('type')
  const charWidth = 7.5
  const textMaxWidth = type === 'person' ? 100 : (type === 'system' ? 110 : 120)
  const lineHeight = 16
  const words = label.split(' ')
  let lineCount = 1
  let currentLineLen = 0
  for (const word of words) {
    const wordLen = word.length * charWidth
    if (currentLineLen + wordLen > textMaxWidth) {
      lineCount++
      currentLineLen = wordLen
    } else {
      currentLineLen += wordLen + charWidth
    }
  }
  let paddingY = 0
  switch (type) {
    case 'person':
      paddingY = 56
      break
    case 'system':
      paddingY = 32
      break
    case 'container':
    case 'datastore':
    case 'queue':
      paddingY = 24
      break
    default:
      paddingY = 24
  }
  const minHeight = type === 'person' ? 70 : (type === 'system' ? 50 : 40)
  return Math.max(minHeight, (lineCount * lineHeight) + paddingY)
}

export function getDefaultStyle(): any[] {
  const colors = {
    person: getCssVar('--color-primary-500'),
    personBorder: getCssVar('--color-primary-600'),
    system: getCssVar('--color-neutral-900'),
    systemBorder: getCssVar('--color-neutral-700'),
    container: Colors.background(),
    containerBorder: getCssVar('--color-neutral-700'),
    component: getCssVar('--color-neutral-100'),
    componentBorder: getCssVar('--color-neutral-400'),
    database: Colors.background(),
    databaseBorder: getCssVar('--color-neutral-600'),
    edge: Colors.neutral500(),
    edgeActive: Colors.info(),
    textDark: Colors.textPrimary(),
    textLight: '#ffffff',
  }
  const icons = getC4Icons()
  return [
    {
      selector: 'node',
      style: {
        'text-valign': 'center',
        'text-halign': 'center',
        'text-wrap': 'wrap',
        'text-max-width': 120,
        'font-family': 'Inter, system-ui, sans-serif',
        'font-size': 12,
        'font-weight': 500,
        'border-width': 2,
        'overlay-opacity': 0,
        'transition-property': 'background-color, border-color, text-rotation',
        'transition-duration': 300,
        'width': (ele: any) => calculateNodeWidth(ele),
        'height': (ele: any) => calculateNodeHeight(ele),
        'label': 'data(label)',
        'text-overflow-wrap': 'anywhere',
      },
    },
    {
      selector: 'node[type="person"]',
      style: {
        'background-color': (ele: any) => getStyleValue(ele, 'style.color', colors.person) || colors.person,
        'border-color': colors.personBorder,
        'color': colors.textLight,
        'label': 'data(label)',
        'shape': (ele: any) => getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
        'padding': 12,
        'padding-top': 44,
        'padding-bottom': 12,
        'padding-left': 12,
        'padding-right': 12,
        'text-max-width': 100,
        'background-image': (ele: any) => getIconFromMetadata(ele, icons.personIcon) || icons.personIcon,
        'background-width': '20px',
        'background-height': '20px',
        'background-position-x': '50%',
        'background-position-y': '12px',
        'background-repeat': 'no-repeat',
        'text-valign': 'bottom',
        'text-halign': 'center',
        'text-margin-y': 12,
        'text-margin-x': 0,
        'text-background-padding': '2px',
        'font-weight': 600,
        'font-size': 11,
      },
    },
    {
      selector: 'node[type="system"]',
      style: {
        'background-color': (ele: any) => getStyleValue(ele, 'style.color', colors.system) || colors.system,
        'border-color': colors.systemBorder,
        'color': colors.textLight,
        'label': 'data(label)',
        'shape': (ele: any) => getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
        'padding': 16,
        'padding-left': 50,
        'padding-right': 16,
        'padding-top': 16,
        'padding-bottom': 16,
        'text-max-width': 110,
        'background-image': (ele: any) => getIconFromMetadata(ele, icons.systemIcon) || icons.systemIcon,
        'background-width': '18px',
        'background-height': '18px',
        'background-position-x': '16px',
        'background-position-y': '50%',
        'background-repeat': 'no-repeat',
        'text-halign': 'left',
        'text-valign': 'center',
        'text-margin-x': 0,
        'text-margin-y': 0,
        'font-size': 14,
        'font-weight': 700,
        'text-transform': 'uppercase',
      },
    },
    {
      selector: 'node[type="container"]',
      style: {
        'background-color': (ele: any) => getStyleValue(ele, 'style.color', colors.container) || colors.container,
        'border-color': colors.containerBorder,
        'color': colors.textDark,
        'label': 'data(label)',
        'shape': (ele: any) => getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
        'padding': 12,
        'padding-left': (ele: any) => {
          const iconName = getStyleValue(ele, 'style.icon', '')
          const hasIcon = iconName && typeof iconName === 'string' && iconName !== ''
          return hasIcon ? 40 : 12
        },
        'padding-right': 12,
        'padding-top': 12,
        'padding-bottom': 12,
        'text-max-width': 120,
        'background-image': (ele: any) => {
          const iconName = getStyleValue(ele, 'style.icon', '')
          if (!iconName || typeof iconName !== 'string') {
            return 'none'
          }
          return getIconFromMetadata(ele, '')
        },
        'background-width': '16px',
        'background-height': '16px',
        'background-position-x': '12px',
        'background-position-y': '50%',
        'background-repeat': 'no-repeat',
        'text-halign': (ele: any) => {
          const iconName = getStyleValue(ele, 'style.icon', '')
          const hasIcon = iconName && typeof iconName === 'string' && iconName !== ''
          return hasIcon ? 'left' : 'center'
        },
        'text-valign': 'center',
        'text-margin-x': (ele: any) => {
          const iconName = getStyleValue(ele, 'style.icon', '')
          const hasIcon = iconName && typeof iconName === 'string' && iconName !== ''
          return hasIcon ? 8 : 0
        },
        'text-margin-y': 0,
      },
    },
    {
      selector: 'node[type="datastore"]',
      style: {
        'background-color': (ele: any) => getStyleValue(ele, 'style.color', colors.database) || colors.database,
        'border-color': colors.databaseBorder,
        'color': colors.textDark,
        'label': 'data(label)',
        'shape': (ele: any) => getStyleValue(ele, 'style.shape', 'barrel') || 'barrel',
        'padding': 12,
        'padding-left': 12,
        'padding-right': 12,
        'padding-top': 12,
        'padding-bottom': 12,
        'text-max-width': 120,
        'border-width': 2,
        'background-image': (ele: any) => {
          const icon = getIconFromMetadata(ele, '')
          return icon && icon !== '' ? icon : 'none'
        },
        'background-width': '16px',
        'background-height': '16px',
        'background-position-x': '50%',
        'background-position-y': '30%',
        'background-repeat': 'no-repeat',
        'text-halign': 'center',
        'text-valign': 'center',
        'text-margin-y': 8,
        'text-margin-x': 0,
      },
    },
    {
      selector: 'node[type="queue"]',
      style: {
        'background-color': (ele: any) => getStyleValue(ele, 'style.color', colors.container) || colors.container,
        'border-color': colors.containerBorder,
        'color': colors.textDark,
        'label': 'data(label)',
        'shape': (ele: any) => getStyleValue(ele, 'style.shape', 'round-rectangle') || 'round-rectangle',
        'padding': 12,
        'padding-left': 12,
        'padding-right': 12,
        'padding-top': 12,
        'padding-bottom': 12,
        'text-max-width': 120,
        'background-image': (ele: any) => {
          const icon = getIconFromMetadata(ele, '')
          return icon && icon !== '' ? icon : 'none'
        },
        'background-width': '16px',
        'background-height': '16px',
        'background-position-x': '50%',
        'background-position-y': '30%',
        'background-repeat': 'no-repeat',
        'text-halign': 'center',
        'text-valign': 'center',
        'text-margin-y': 8,
        'text-margin-x': 0,
      },
    },
    {
      selector: 'node[type="requirement"]',
      style: {
        'display': 'none'
      },
    },
    {
      selector: 'node[type="adr"]',
      style: {
        'display': 'none'
      },
    },
    {
      selector: 'node[type="deployment"]',
      style: {
        'background-color': '#f8fafc',
        'border-color': '#64748b',
        'color': '#334155',
        'label': 'data(label)',
        'shape': 'cut-rectangle',
        'padding': 16,
        'border-width': 2,
        'border-style': 'dashed',
      },
    },
    {
      selector: ':parent',
      style: {
        'background-color': '#f8fafc',
        'border-color': '#cbd5e1',
        'border-width': 1,
        'border-style': 'dashed',
        'label': 'data(label)',
        'text-valign': 'top',
        'text-halign': 'center',
        'text-margin-y': -10,
        'color': '#64748b',
        'font-size': 12,
        'font-weight': 600,
        'padding': 24,
        'text-transform': 'uppercase',
      },
    },
    {
      selector: '.collapsed',
      style: {
        'background-color': '#cbd5e1',
        'border-style': 'dashed',
        'label': 'data(label)',
        'text-valign': 'center',
        'text-halign': 'center',
        'width': 60,
        'height': 60
      }
    },
    {
      selector: 'edge',
      style: {
        'width': 2,
        'line-color': Colors.neutral500(),
        'target-arrow-color': Colors.neutral500(),
        'target-arrow-shape': 'triangle',
        'curve-style': 'bezier',
        'label': 'data(label)',
        'font-size': '10px',
        'color': colors.edge,
        'text-background-color': Colors.background(),
        'text-background-opacity': 1,
        'text-background-padding': '2px',
        'text-background-shape': 'round-rectangle',
        'text-border-opacity': 0,
        'text-rotation': 'autorotate',
        'arrow-scale': 1.2,
        'opacity': 0.95,
      },
    },
    {
      selector: 'node:selected',
      style: {
        'border-width': 4,
        'border-color': Colors.info(),
      },
    },
    {
      selector: 'edge:selected',
      style: {
        'line-color': Colors.info(),
        'target-arrow-color': Colors.info(),
        'color': Colors.info(),
        'width': 3,
      },
    },
    {
      selector: 'node:active',
      style: {
        'overlay-opacity': 0,
      },
    },
  ]
}

