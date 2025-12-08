import { useEffect } from 'react'

export default function EngagementTracker() {
  useEffect(() => {
    const handler = (e: any) => {
      try {
        const now = new Date()
        const last = localStorage.getItem('sruja-last')
        const xp = parseInt(localStorage.getItem('sruja-xp') || '0', 10) + 1
        localStorage.setItem('sruja-xp', String(xp))
        if (last) {
          const lastDate = new Date(last)
          const diff = Math.floor((now.getTime() - lastDate.getTime()) / 86400000)
          if (diff >= 1) {
            const streak = parseInt(localStorage.getItem('sruja-streak') || '0', 10) + 1
            localStorage.setItem('sruja-streak', String(streak))
          }
        } else {
          localStorage.setItem('sruja-streak', '1')
        }
        localStorage.setItem('sruja-last', now.toISOString())
      } catch {}
    }
    window.addEventListener('sruja:event', handler as any)
    return () => window.removeEventListener('sruja:event', handler as any)
  }, [])
  return null
}
