// apps/social-publish/src/linkedin_post.ts
import { logger } from '@sruja/shared';
import { logError } from './utils/error-handler.js';

logger.setService('linkedin-publish');

const accessToken = process.env.LINKEDIN_ACCESS_TOKEN;
const title = process.env.POST_TITLE || 'New release';
const url = process.env.POST_URL;
const body = process.env.POST_BODY || '';

interface LinkedInProfile {
  sub: string;
}

interface LinkedInMe {
  id: string;
}

interface LinkedInPostResponse {
  id?: string;
}

async function getPersonUrn(accessToken: string): Promise<string> {
  const headers = {
    Authorization: `Bearer ${accessToken}`,
    'Content-Type': 'application/json',
  };

  // Try userinfo endpoint first
  try {
    const response = await fetch('https://api.linkedin.com/v2/userinfo', {
      headers,
    });

    if (response.ok) {
      const data = (await response.json()) as LinkedInProfile;
      if (data.sub) {
        return data.sub;
      }
    }
  } catch {
    // Silently continue to fallback
  }

  // Fallback to /me endpoint
  const meResponse = await fetch('https://api.linkedin.com/v2/me', {
    headers,
  });

  if (!meResponse.ok) {
    throw new Error(
      `Failed to get LinkedIn person URN: HTTP ${meResponse.status}`
    );
  }

  const meData = (await meResponse.json()) as LinkedInMe;
  if (!meData.id) {
    throw new Error('Failed to get LinkedIn person URN: no ID found');
  }

  return meData.id;
}

async function main() {
  try {
    const personUrn = await getPersonUrn(accessToken);

    const textContent = url
      ? `${title}\n\n${body}\n\n${url}`
      : `${title}\n\n${body}`;

    const postData: {
      author: string;
      lifecycleState: string;
      specificContent: {
        'com.linkedin.ugc.ShareContent': {
          shareCommentary: { text: string };
          shareMediaCategory: string;
          media?: Array<{
            status: string;
            description: { text: string };
            originalUrl: string;
            title: { text: string };
          }>;
        };
      };
      visibility: {
        'com.linkedin.ugc.MemberNetworkVisibility': string;
      };
    } = {
      author: `urn:li:person:${personUrn}`,
      lifecycleState: 'PUBLISHED',
      specificContent: {
        'com.linkedin.ugc.ShareContent': {
          shareCommentary: {
            text: textContent.slice(0, 3000), // LinkedIn limit
          },
          shareMediaCategory: url ? 'ARTICLE' : 'NONE',
        },
      },
      visibility: {
        'com.linkedin.ugc.MemberNetworkVisibility': 'PUBLIC',
      },
    };

    if (url) {
      postData.specificContent['com.linkedin.ugc.ShareContent'].media = [
        {
          status: 'READY',
          description: {
            text: body.slice(0, 200) || title,
          },
          originalUrl: url,
          title: {
            text: title,
          },
        },
      ];
    }

    const postResponse = await fetch('https://api.linkedin.com/v2/ugcPosts', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(postData),
    });

    if (!postResponse.ok) {
      throw new Error(`HTTP ${postResponse.status}`);
    }

    const responseData = (await postResponse.json()) as LinkedInPostResponse;
    logger.info('Post published successfully', { postId: responseData.id });
  } catch (error) {
    logError('LinkedIn', error);
    process.exit(0);
  }
}

main();

