// apps/social-publish/src/reddit_post.ts
import { getAccessToken, getAccessTokenFromRefreshToken, submitPost } from './reddit_client.js'
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('reddit-publish');

const clientId = process.env.REDDIT_CLIENT_ID;
const clientSecret = process.env.REDDIT_CLIENT_SECRET;
const username = process.env.REDDIT_USERNAME;
const password = process.env.REDDIT_PASSWORD;
const refreshToken = process.env.REDDIT_REFRESH_TOKEN;
const userAgent = process.env.REDDIT_USER_AGENT || 'sruja-lang-bot/1.0';
const subredditName = process.env.REDDIT_SUBREDDIT;

const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

async function main() {
  const hasRefresh = !!clientId && !!clientSecret && !!refreshToken && !!subredditName
  const hasPasswordGrant = !!clientId && !!clientSecret && !!username && !!password && !!subredditName
  if (!hasRefresh && !hasPasswordGrant) {
    logError('Reddit', new Error('Missing required credentials or subreddit'))
    process.exit(0)
  }

  try {
    const token = hasRefresh
      ? await getAccessTokenFromRefreshToken({ clientId, clientSecret, refreshToken, userAgent })
      : await getAccessToken({ clientId, clientSecret, username, password, userAgent })

    const bodyTrimmed = body.slice(0, 20000)
    await submitPost({
      token,
      userAgent,
      subreddit: subredditName,
      title,
      url,
      text: url ? undefined : bodyTrimmed,
    })
    logger.info('Post published successfully', { subreddit: subredditName, title })
  } catch (error) {
    logError('Reddit', error);
    process.exit(0); // Exit successfully to continue workflow
  }
}

main();
