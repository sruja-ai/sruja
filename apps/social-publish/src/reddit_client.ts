import { logger } from '@sruja/shared'
import { fetch } from 'undici'

type TokenResponse = {
  access_token: string
  token_type: string
  expires_in: number
  scope: string
}

export async function getAccessToken(params: {
  clientId: string
  clientSecret: string
  username: string
  password: string
  userAgent: string
}): Promise<string> {
  const { clientId, clientSecret, username, password, userAgent } = params

  const body = new URLSearchParams({
    grant_type: 'password',
    username,
    password,
  })

  const auth = Buffer.from(`${clientId}:${clientSecret}`).toString('base64')

  const resp = await fetch('https://www.reddit.com/api/v1/access_token', {
    method: 'POST',
    headers: {
      Authorization: `Basic ${auth}`,
      'User-Agent': userAgent,
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body,
  })

  if (!resp.ok) {
    const text = await resp.text()
    throw new Error(`Reddit OAuth failed (${resp.status}): ${text}`)
  }

  const json = (await resp.json()) as TokenResponse
  return json.access_token
}

export async function getAccessTokenFromRefreshToken(params: {
  clientId: string
  clientSecret: string
  refreshToken: string
  userAgent: string
}): Promise<string> {
  const { clientId, clientSecret, refreshToken, userAgent } = params
  const body = new URLSearchParams({
    grant_type: 'refresh_token',
    refresh_token: refreshToken,
  })
  const auth = Buffer.from(`${clientId}:${clientSecret}`).toString('base64')
  const resp = await fetch('https://www.reddit.com/api/v1/access_token', {
    method: 'POST',
    headers: {
      Authorization: `Basic ${auth}`,
      'User-Agent': userAgent,
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body,
  })
  if (!resp.ok) {
    const text = await resp.text()
    throw new Error(`Reddit OAuth refresh failed (${resp.status}): ${text}`)
  }
  const json = (await resp.json()) as TokenResponse
  return json.access_token
}

export async function submitPost(params: {
  token: string
  userAgent: string
  subreddit: string
  title: string
  url?: string
  text?: string
}): Promise<void> {
  const { token, userAgent, subreddit, title, url, text } = params

  const body = new URLSearchParams({
    api_type: 'json',
    sr: subreddit,
    title,
    kind: url ? 'link' : 'self',
    resubmit: 'true',
  })

  if (url) body.set('url', url)
  if (text) body.set('text', text)

  const resp = await fetch('https://oauth.reddit.com/api/submit', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'User-Agent': userAgent,
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body,
  })

  const raw = await resp.text()
  if (!resp.ok) {
    throw new Error(`Submit failed (${resp.status}): ${raw}`)
  }

  try {
    const parsed = JSON.parse(raw)
    const errors = parsed?.json?.errors as unknown[] | undefined
    if (errors && errors.length) {
      logger.warn('Reddit API returned errors', { errors })
    }
  } catch {
    // non-json response; ignore
  }
}
