import { createContext, useContext, useEffect, useMemo, useState } from 'react'
import { initPosthog, capture, identify, isReady, enableAutoTracking } from '@sruja/shared'

type Ctx = {
  capture: (event: string, props?: Record<string, any>) => void
  identify: (id: string, props?: Record<string, any>) => void
  ready: boolean
}

const PosthogCtx = createContext<Ctx>({ capture: () => {}, identify: () => {}, ready: false })

export function usePosthog() {
  return useContext(PosthogCtx)
}

export type PosthogProviderProps = {
  apiKey: string
  host?: string
  options?: Record<string, any>
  auto?: boolean
  children: any
}

export function PosthogProvider({ apiKey, host, options, auto = true, children }: PosthogProviderProps) {
  const [ready, setReady] = useState(false)
  useEffect(() => {
    if (typeof window === 'undefined') return
    // Only initialize if we have a valid API key
    if (apiKey && apiKey.trim() !== '') {
      initPosthog({ apiKey, host, options }).then(() => {
        if (auto) enableAutoTracking()
        setReady(true)
      })
    } else {
      // If no API key, mark as ready but don't initialize (silent fail)
      setReady(true)
    }
  }, [apiKey, host, options])
  const value = useMemo<Ctx>(() => ({ capture, identify, ready: ready || isReady() }), [ready])
  return <PosthogCtx.Provider value={value}>{children}</PosthogCtx.Provider>
}
