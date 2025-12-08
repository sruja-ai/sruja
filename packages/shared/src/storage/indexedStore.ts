type KV = {
  get: (key: string) => Promise<string | null>
  set: (key: string, value: string) => Promise<void>
}

function createLocal(): KV {
  return {
    async get(key) {
      if (typeof window === 'undefined') return null
      return window.localStorage.getItem(key)
    },
    async set(key, value) {
      if (typeof window === 'undefined') return
      window.localStorage.setItem(key, value)
    },
  }
}

function createIndexed(): Promise<KV> {
  return new Promise((resolve) => {
    if (typeof indexedDB === 'undefined') return resolve(createLocal())
    const req = indexedDB.open('sruja', 1)
    req.onupgradeneeded = () => {
      const db = req.result
      if (!db.objectStoreNames.contains('kv')) db.createObjectStore('kv')
    }
    req.onsuccess = () => {
      const db = req.result
      resolve({
        async get(key) {
          return new Promise((res) => {
            const tx = db.transaction('kv', 'readonly')
            const store = tx.objectStore('kv')
            const g = store.get(key)
            g.onsuccess = () => res((g.result as string) || null)
            g.onerror = () => res(null)
          })
        },
        async set(key, value) {
          return new Promise((res) => {
            const tx = db.transaction('kv', 'readwrite')
            const store = tx.objectStore('kv')
            const p = store.put(value, key)
            p.onsuccess = () => res()
            p.onerror = () => res()
          })
        },
      })
    }
    req.onerror = () => resolve(createLocal())
  })
}

let kvPromise: Promise<KV> | null = null

export async function getStore(): Promise<KV> {
  if (!kvPromise) kvPromise = createIndexed()
  return kvPromise
}

export async function storeSet(key: string, value: string) {
  const s = await getStore()
  return s.set(key, value)
}

export async function storeGet(key: string) {
  const s = await getStore()
  return s.get(key)
}
