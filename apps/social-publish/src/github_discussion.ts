// apps/social-publish/src/github_discussion.ts
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('github-discussion');

const token = process.env.GITHUB_TOKEN;
const repo = process.env.GITHUB_REPOSITORY;
const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

async function main() {
  if (!token || !repo) {
    logError('GitHub Discussion', new Error('Missing required credentials'));
    process.exit(0);
  }

  try {
    // Build discussion body
    let discussionBody = `## ${title}\n\n`;
    
    if (body) {
      discussionBody += `${body}\n\n`;
    }
    
    if (url) {
      discussionBody += `🔗 [View Release](${url})\n\n`;
    }
    
    discussionBody += `---\n\n_This discussion was automatically created from a release._`;

    // First, check if discussions are enabled and get category ID
    // Default to "Announcements" category (ID 1) or "General" (ID 2)
    const categoryId = process.env.GITHUB_DISCUSSION_CATEGORY_ID || '1';

    const response = await fetch(`https://api.github.com/repos/${repo}/discussions`, {
      method: 'POST',
      headers: {
        'Authorization': `token ${token}`,
        'Accept': 'application/vnd.github+json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        title: `Release: ${title}`,
        body: discussionBody,
        category: {
          id: parseInt(categoryId, 10),
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    logger.info('Discussion created successfully', { discussionId: data.number, url: data.html_url });
  } catch (error) {
    logError('GitHub Discussion', error);
    process.exit(0);
  }
}

main();

