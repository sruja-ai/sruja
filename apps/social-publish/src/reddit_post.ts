// apps/social-publish/src/reddit_post.ts
import Snoowrap from 'snoowrap';
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('reddit-publish');

const clientId = process.env.REDDIT_CLIENT_ID;
const clientSecret = process.env.REDDIT_CLIENT_SECRET;
const username = process.env.REDDIT_USERNAME;
const password = process.env.REDDIT_PASSWORD;
const userAgent = process.env.REDDIT_USER_AGENT || 'sruja-lang-bot/1.0';
const subredditName = process.env.REDDIT_SUBREDDIT;

const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

async function main() {
  if (!clientId || !clientSecret || !username || !password || !subredditName) {
    logError('Reddit', new Error('Missing required credentials or subreddit'));
    process.exit(0); // Exit successfully to continue workflow
  }

  try {
    const reddit = new Snoowrap({
      userAgent,
      clientId,
      clientSecret,
      username,
      password,
    });

    const subreddit = reddit.getSubreddit(subredditName);

    if (url) {
      await subreddit.submitLink({ title, url });
    } else {
      const bodyTrimmed = body.slice(0, 20000);
      await subreddit.submitSelfpost({ title, text: bodyTrimmed });
    }
    logger.info('Post published successfully', { subreddit: subredditName, title });
  } catch (error) {
    logError('Reddit', error);
    process.exit(0); // Exit successfully to continue workflow
  }
}

main();

