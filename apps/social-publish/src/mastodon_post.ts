// apps/social-publish/src/mastodon_post.ts
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('mastodon-publish');

const accessToken = process.env.MASTODON_ACCESS_TOKEN;
const instanceUrl = process.env.MASTODON_INSTANCE_URL;
const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

async function main() {
  if (!accessToken || !instanceUrl) {
    logError('Mastodon', new Error('Missing required credentials'));
    process.exit(0);
  }

  try {
    // Build status content (500 char limit)
    let status = `${title}`;
    
    if (body) {
      const bodyPreview = body.slice(0, 300).replace(/\n/g, ' ');
      status += `\n\n${bodyPreview}`;
    }
    
    if (url) {
      status += `\n\n${url}`;
    }
    
    // Trim to 500 characters
    if (status.length > 500) {
      const urlLength = url ? url.length + 2 : 0;
      const maxBodyLength = 500 - title.length - urlLength - 4;
      const bodyPreview = body.slice(0, Math.max(0, maxBodyLength)).replace(/\n/g, ' ');
      status = `${title}\n\n${bodyPreview}${url ? `\n\n${url}` : ''}`;
    }

    // Ensure instance URL doesn't have trailing slash
    const baseUrl = instanceUrl.replace(/\/$/, '');
    const apiUrl = `${baseUrl}/api/v1/statuses`;

    const response = await fetch(apiUrl, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        status,
        visibility: 'public',
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    logger.info('Post published successfully', { postId: data.id, instance: instanceUrl });
  } catch (error) {
    logError('Mastodon', error);
    process.exit(0);
  }
}

main();

