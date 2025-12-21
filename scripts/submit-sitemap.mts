#!/usr/bin/env zx
// scripts/submit-sitemap.mts
// Manual script to submit sitemap to search engines
// Can also be run in CI/CD

const SITE_URL = process.argv[2] || 'https://sruja.ai';
const SITEMAP_URL = `${SITE_URL}/sitemap.xml`;

console.log('=== Submitting Sitemap to Search Engines ===');
console.log(`Sitemap URL: ${SITEMAP_URL}\n`);

const searchEngines = [
  { name: 'Google', url: `https://www.google.com/ping?sitemap=${SITEMAP_URL}` },
  { name: 'Bing', url: `https://www.bing.com/ping?sitemap=${SITEMAP_URL}` },
  { name: 'Yandex', url: `https://webmaster.yandex.com/ping?sitemap=${SITEMAP_URL}` },
];

for (const engine of searchEngines) {
  console.log(`📤 Submitting to ${engine.name}...`);
  try {
    await $`curl -s ${engine.url}`;
    console.log(`✅ Successfully pinged ${engine.name}`);
  } catch {
    console.log(`❌ Failed to ping ${engine.name}`);
  }
}

console.log('\n=== Submission Complete ===\n');
console.log('Note: These pings notify search engines about sitemap updates.');
console.log('For full automation with Search Console API, see docs/SEO_SUBMISSION_AUTOMATION.md');

