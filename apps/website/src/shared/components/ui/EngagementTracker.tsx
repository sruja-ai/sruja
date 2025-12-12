import { useEffect, useRef } from 'react'
import { createStringStorage } from '@/shared/utils/storage'

export default function EngagementTracker() {
  const tRef = useRef<number>(0)
  useEffect(() => {
    const lastStorage = createStringStorage('sruja-last', '')
    const xpStorage = createStringStorage('sruja-xp', '0')
    const streakStorage = createStringStorage('sruja-streak', '0')
    const allow = new Set<string>([
      'page.view',
      'tutorial.diagram_show',
      'tutorial.open_playground',
      'quiz.view',
      'quiz.complete',
    ])
    
    try {
      const initEvt = new CustomEvent('sruja:engagement', { detail: { xp: parseInt(xpStorage.get() || '0', 10), streak: parseInt(streakStorage.get() || '0', 10), last: lastStorage.get() || '' } })
      window.dispatchEvent(initEvt)
    } catch (err) { console.warn('engagement init dispatch failed', err) }

    const handler = (e: CustomEvent<{ type: string }>) => {
      const nowMs = Date.now()
      if (nowMs - tRef.current < 5000) return
      const now = new Date()
      const last = lastStorage.get()
      const ty = (e.detail && e.detail.type) || ''
      if (!allow.has(ty)) return
      const xp = parseInt(xpStorage.get() || '0', 10) + 1
      xpStorage.set(String(xp))
      if (last) {
        const lastDate = new Date(last)
        const diff = Math.floor((now.getTime() - lastDate.getTime()) / 86400000)
        if (diff >= 1) {
          const streak = parseInt(streakStorage.get() || '0', 10) + 1
          streakStorage.set(String(streak))
        }
      } else {
        streakStorage.set('1')
      }
      lastStorage.set(now.toISOString())
      tRef.current = nowMs
      try {
        const updEvt = new CustomEvent('sruja:engagement', { detail: { xp: parseInt(xpStorage.get() || '0', 10), streak: parseInt(streakStorage.get() || '0', 10), last: lastStorage.get() || '' } })
        window.dispatchEvent(updEvt)
      } catch (err) { console.warn('engagement update dispatch failed', err) }
    }
    window.addEventListener('sruja:event', handler as EventListener)
    return () => window.removeEventListener('sruja:event', handler as EventListener)
  }, [])
  return null
}
