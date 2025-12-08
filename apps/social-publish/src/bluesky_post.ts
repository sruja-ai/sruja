// apps/social-publish/src/bluesky_post.ts
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('bluesky-publish');

const identifier = process.env.BLUESKY_IDENTIFIER;
const password = process.env.BLUESKY_PASSWORD;
const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

async function authenticate() {
  const response = await fetch('https://bsky.social/xrpc/com.atproto.server.createSession', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      identifier,
      password,
    }),
  });

  if (!response.ok) {
    throw new Error(`Authentication failed: HTTP ${response.status}`);
  }

  const data = await response.json();
  return {
    accessJwt: data.accessJwt,
    did: data.did,
  };
}

async function main() {
  if (!identifier || !password) {
    logError('Bluesky', new Error('Missing required credentials'));
    process.exit(0);
  }

  try {
    const { accessJwt, did } = await authenticate();

    // Build post text (300 char limit)
    let text = `${title}`;
    
    if (body) {
      const bodyPreview = body.slice(0, 200).replace(/\n/g, ' ');
      text += `\n\n${bodyPreview}`;
    }
    
    if (url) {
      text += `\n\n${url}`;
    }
    
    // Trim to 300 characters
    if (text.length > 300) {
      const urlLength = url ? url.length + 2 : 0;
      const maxBodyLength = 300 - title.length - urlLength - 4;
      const bodyPreview = body.slice(0, Math.max(0, maxBodyLength)).replace(/\n/g, ' ');
      text = `${title}\n\n${bodyPreview}${url ? `\n\n${url}` : ''}`;
    }

    // Create facets for URL if present
    const facets = url ? [
      {
        index: {
          byteStart: text.indexOf(url),
          byteEnd: text.indexOf(url) + url.length,
        },
        features: [
          {
            $type: 'app.bsky.richtext.facet#link',
            uri: url,
          },
        ],
      },
    ] : [];

    const response = await fetch('https://bsky.social/xrpc/com.atproto.repo.createRecord', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${accessJwt}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        repo: did,
        collection: 'app.bsky.feed.post',
        record: {
          text,
          facets,
          createdAt: new Date().toISOString(),
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    logger.info('Post published successfully', { postId: data.uri });
  } catch (error) {
    logError('Bluesky', error);
    process.exit(0);
  }
}

main();

