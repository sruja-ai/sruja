import type { Core } from 'cytoscape'
import type { LayoutOptions } from './types/layout'
import { logger } from '@sruja/shared'

export async function waitForContainerSize(el: HTMLElement): Promise<void> {
  const rect = el.getBoundingClientRect()
  if (rect.width > 0 && rect.height > 0) return
  await new Promise<void>((resolve) => {
    let resolved = false
    const check = () => {
      const r = el.getBoundingClientRect()
      if (r.width > 0 && r.height > 0 && !resolved) {
        resolved = true
        resolve()
      }
    }
    if (typeof ResizeObserver !== 'undefined') {
      const ro = new ResizeObserver(() => check())
      ro.observe(el)
      check()
      setTimeout(() => {
        if (!resolved) {
          ro.disconnect()
          resolve()
        }
      }, 2000)
    } else {
      let attempts = 0
      const max = 20
      const id = setInterval(() => {
        attempts++
        check()
        if (resolved || attempts >= max) {
          clearInterval(id)
          resolve()
        }
      }, 100)
    }
  })
}

export function computeLayoutOptions(usePreset: boolean, engine: string | undefined): LayoutOptions {
  if (usePreset) {
    return {
      name: 'preset',
      fit: true,
      padding: 50,
      animate: true,
      animationDuration: 500,
    }
  }
  const name = engine && ['dagre', 'cose', 'fcose', 'elk'].includes(engine) ? engine : 'dagre'
  return {
    name,
    rankDir: 'TB',
    nodeSep: 50,
    rankSep: 100,
    padding: 50,
    animate: true,
    animationDuration: 500,
    fit: true,
  }
}

export function createLayoutStopHandler(cy: Core, container: HTMLElement, onDone: () => void): () => void {
  let retries = 0
  const maxRetries = 5
  return () => {
    if (!cy || cy.destroyed()) return
    const rect = container.getBoundingClientRect()
    logger.debug('Layout stopped, container dimensions', { width: rect.width, height: rect.height })
    if (rect.width > 0 && rect.height > 0) {
      cy.fit(undefined, 80)
      cy.off('layoutstop', onDone as any)
    } else {
      retries++
      if (retries >= maxRetries) {
        logger.warn('Container still has zero dimensions after max retries, giving up')
        cy.off('layoutstop', onDone as any)
        return
      }
      logger.warn(`Container still has zero dimensions at layoutstop, will retry (${retries}/${maxRetries})`)
      setTimeout(() => {
        if (cy && !cy.destroyed()) {
          const retryRect = container.getBoundingClientRect()
          if (retryRect.width > 0 && retryRect.height > 0) {
            cy.fit(undefined, 80)
            cy.off('layoutstop', onDone as any)
          }
        }
      }, 200)
    }
  }
}

export function fitPresetWithRetry(cy: Core, container: HTMLElement): void {
  const rect = container.getBoundingClientRect()
  if (rect.width > 0 && rect.height > 0) {
    cy.fit(undefined, 80)
    return
  }
  logger.warn('Container has zero dimensions, will retry fit after delay')
  let retryCount = 0
  const maxRetries = 5
  const tryFit = () => {
    if (!cy || cy.destroyed()) return
    retryCount++
    const r = container.getBoundingClientRect()
    if (r.width > 0 && r.height > 0) {
      cy.fit(undefined, 80)
    } else if (retryCount < maxRetries) {
      logger.warn(`Container still has zero dimensions, retrying (${retryCount}/${maxRetries})`)
      setTimeout(tryFit, 200)
    } else {
      logger.warn('Container still has zero dimensions after max retries, giving up')
    }
  }
  setTimeout(tryFit, 200)
}

