# Archived Tests

These tests were archived because they may not be relevant to the current state of the designer app, or they test features that have changed significantly.

## Archived Test Files

- `builder-wizard.spec.ts` - Consolidated into `builder.spec.ts`
- `code-tabs.spec.ts` - Code panel functionality covered in `app.spec.ts`
- `core-navigation.spec.ts` - Consolidated into `app.spec.ts`
- `create-new.spec.ts` - Basic functionality covered in `app.spec.ts`
- `details-panel.spec.ts` - Details view covered in `app.spec.ts`
- `examples-loading.spec.ts` - Consolidated into `app.spec.ts`
- `examples-search.spec.ts` - Examples functionality covered in `app.spec.ts`
- `export-and-shortcuts.spec.ts` - Consolidated into `import-export.spec.ts`
- `import-export-share.spec.ts` - Consolidated into `import-export.spec.ts`
- `likec4-integration.spec.ts` - Diagram rendering covered in `app.spec.ts`
- `navigation-collapse.spec.ts` - Navigation covered in `app.spec.ts`
- `navigation-panel.spec.ts` - Navigation covered in `app.spec.ts`
- `settings-and-mode.spec.ts` - Settings functionality can be added if needed
- `share-export.spec.ts` - Consolidated into `import-export.spec.ts`
- `share-panel.spec.ts` - Share functionality can be added if needed
- `smoke.spec.ts` - Consolidated into `app.spec.ts`
- `tabs-url.spec.ts` - URL state covered in `app.spec.ts`
- `template-gallery.spec.ts` - Template gallery can be added if still used
- `url-state.spec.ts` - URL state covered in `app.spec.ts`

## Current Test Structure

The current e2e test suite focuses on core functionality:

- `app.spec.ts` - Core app functionality (loading, view tabs, examples)
- `builder.spec.ts` - Builder wizard functionality
- `import-export.spec.ts` - Import/export features
- `ecommerce-quality.spec.ts` - Quality measurement (kept as specialized test)

## Restoring Tests

If you need to restore any of these tests, review them against the current app state and update selectors/assertions as needed.

