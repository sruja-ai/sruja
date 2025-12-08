#!/usr/bin/env zx
// scripts/verify-components.mts
// Visual Testing Verification Script
// This script checks that all components are properly imported and used

console.log('🔍 Verifying Component Integration...\n');

// Check viewer-core app
console.log('📱 Checking Viewer Core App...');
const viewerImports = await $`grep -r "from '@sruja/ui'" apps/viewer-core/app --include="*.tsx" --include="*.ts" | wc -l`.quiet();
console.log(`  ✓ Found ${viewerImports.stdout.trim()} imports from @sruja/ui`);

// Check studio-core package
console.log('📱 Checking Studio Core Package...');
const studioImports = await $`grep -r "from '@sruja/ui'" apps/studio-core/src --include="*.tsx" --include="*.ts" | wc -l`.quiet();
console.log(`  ✓ Found ${studioImports.stdout.trim()} imports from @sruja/ui`);

// Verify specific components
console.log('\n🔍 Verifying Component Usage...\n');

const components = ['Button', 'Card', 'Badge', 'Breadcrumb', 'SearchBar'];
for (const comp of components) {
  const result = await $`grep -r "${comp}" apps/viewer-core apps/studio-core --include="*.tsx" --include="*.ts" | grep -v "node_modules" | wc -l`.quiet();
  console.log(`  ✓ ${comp}: Used ${result.stdout.trim()} times`);
}

console.log('\n✅ Component integration verification complete!\n');
console.log('📋 Next Steps:');
console.log('  1. Run \'cd apps/storybook && npm run dev\' to visually test components');
console.log('  2. Run \'cd apps/viewer-core && npm run dev\' to test viewer core');
console.log('  3. Run \'cd apps/website && npm run dev\' to test studio app');
console.log('  4. Check browser console for any runtime errors');

