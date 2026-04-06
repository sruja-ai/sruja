# CI Chromatic Workflow Documentation

## Overview

The Chromatic workflow provides automated visual testing and UI review for the Sruja Storybook components. This workflow runs on every push to ensure visual consistency across the design system.

## Workflow Configuration

### File Location
`.github/workflows/chromatic.yml`

### Trigger Configuration
```yaml
name: "Chromatic"

on: push
```

**Trigger Strategy**: Runs on every push to any branch, providing immediate visual feedback on component changes.

## Job Configuration

### Job: `chromatic`

**Runner**: `ubuntu-latest`
**Purpose**: Execute visual regression testing using Chromatic

### Step-by-Step Breakdown

#### 1. Checkout Code
```yaml
- name: Checkout code
  uses: actions/checkout@v6
  with:
    fetch-depth: 0
```

**Configuration Details**:
- **Action**: `actions/checkout@v6`
- **Fetch Depth**: `0` (full git history)
- **Purpose**: Required for Chromatic to access complete git history for baseline comparisons

#### 2. Setup Node.js Environment
```yaml
- uses: actions/setup-node@v6
  with:
    node-version: 22.20.0
```

**Configuration Details**:
- **Node Version**: `22.20.0` (pinned for consistency)
- **Purpose**: Ensures consistent Node.js environment across CI runs

#### 3. Install Dependencies
```yaml
- name: Install dependencies
  run: npm ci
```

**Configuration Details**:
- **Command**: `npm ci` (clean install)
- **Purpose**: Fast, reliable dependency installation using lock file

#### 4. Run Chromatic
```yaml
- name: Run Chromatic
  uses: chromaui/action@4c20b95e9d3209ecfdf9cd6aace6bbde71ba1694 # v13
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    workingDir: apps/storybook
```

**Configuration Details**:
- **Action**: `chromaui/action@v13` (pinned to specific commit)
- **Project Token**: Retrieved from GitHub secrets
- **Working Directory**: `apps/storybook`

## Security Configuration

### Required Secrets

#### `CHROMATIC_PROJECT_TOKEN`
- **Type**: Repository secret
- **Purpose**: Authentication with Chromatic service
- **Location**: Repository Settings → Secrets and variables → Actions
- **Format**: Alphanumeric token provided by Chromatic

### Security Best Practices

1. **Token Management**:
   - Store token as encrypted repository secret
   - Never commit token to code
   - Rotate tokens periodically
   - Use least-privilege access

2. **Action Pinning**:
   - Third-party actions are pinned to specific commits
   - Prevents supply chain attacks
   - Ensures reproducible builds

## Integration Points

### Storybook Configuration

The workflow expects Storybook to be configured in `apps/storybook/`:

```typescript
// apps/storybook/.storybook/main.ts
export default {
  stories: ['../src/**/*.stories.@(js|jsx|ts|tsx)'],
  addons: [
    '@storybook/addon-essentials',
    '@storybook/addon-interactions',
  ],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
}
```

### Package Configuration

**Package**: `apps/storybook/package.json`
```json
{
  "scripts": {
    "build-storybook": "storybook build",
    "chromatic": "chromatic --exit-zero-on-changes"
  },
  "devDependencies": {
    "chromatic": "^11.0.0"
  }
}
```

## Visual Testing Strategy

### Component Coverage

The workflow tests all Storybook stories:

```typescript
// Example story: apps/storybook/src/stories/Button.stories.tsx
import type { Meta, StoryObj } from '@storybook/react'
import { Button } from '@sruja/ui'

const meta: Meta<typeof Button> = {
  title: 'Components/Button',
  component: Button,
  parameters: {
    chromatic: { disableSnapshot: false }
  }
}

export default meta

type Story = StoryObj<typeof meta>

export const Primary: Story = {
  args: {
    variant: 'primary',
    children: 'Button'
  }
}
```

### Baseline Management

1. **Automatic Baselines**: Chromatic automatically creates baselines from main branch
2. **Manual Acceptance**: UI changes require manual review and acceptance
3. **Branch Comparisons**: Feature branches are compared against main branch baselines

## Monitoring and Troubleshooting

### Success Indicators

1. **Build Status**: Green checkmark in GitHub Actions
2. **Chromatic Dashboard**: Shows accepted changes and new baselines
3. **Pull Request**: UI Review check appears in PR status

### Common Issues

#### 1. Missing Project Token
**Error**: `Chromatic: Missing project token`
**Solution**: 
```bash
# Add token to repository secrets
echo "CHROMATIC_PROJECT_TOKEN=your_token_here" >> $GITHUB_ENV
```

#### 2. Storybook Build Failures
**Error**: `Build failed with errors`
**Solution**:
```bash
# Test locally first
cd apps/storybook
npm run build-storybook
```

#### 3. Snapshot Timeout
**Error**: `Snapshot timeout after 30000ms`
**Solution**:
```yaml
# Add timeout configuration
- name: Run Chromatic
  uses: chromaui/action@v13
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    workingDir: apps/storybook
    storybookBuildDir: storybook-static
    exitZeroOnChanges: true
    exitOnceUploaded: true
```

### Performance Optimization

#### Build Performance
```yaml
- name: Install dependencies
  run: |
    npm ci --prefer-offline --no-audit --no-fund
    npx playwright install-deps
```

#### Cache Configuration
```yaml
- name: Cache dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.npm
      node_modules
      apps/storybook/node_modules
    key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
```

## Branch Strategy Integration

### Main Branch Protection

```yaml
# .github/workflows/chromatic.yml (enhanced)
name: "Chromatic"

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  chromatic:
    runs-on: ubuntu-latest
    steps:
      # ... existing steps
      - name: Run Chromatic
        uses: chromaui/action@v13
        with:
          projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
          workingDir: apps/storybook
          autoAcceptChanges: github.ref == 'refs/heads/main'
          exitZeroOnChanges: true
```

### Pull Request Integration

The workflow integrates with pull requests by:
1. Posting visual diffs as PR comments
2. Blocking merge on unreviewed changes
3. Providing UI review checklist

## Advanced Configuration

### Conditional Execution

```yaml
- name: Run Chromatic
  if: github.event.pull_request.draft == false
  uses: chromaui/action@v13
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    workingDir: apps/storybook
    onlyChanged: true
```

### Monorepo Support

```yaml
- name: Check for storybook changes
  id: changes
  uses: dorny/paths-filter@v3
  with:
    filters: |
      storybook:
        - 'apps/storybook/**'
        - 'packages/ui/**'

- name: Run Chromatic
  if: steps.changes.outputs.storybook == 'true'
  uses: chromaui/action@v13
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    workingDir: apps/storybook
```

## Metrics and Reporting

### Key Metrics Tracked

1. **Visual Coverage**: Percentage of components with visual tests
2. **Change Detection**: Number of visual changes per build
3. **Review Time**: Time to review and accept changes
4. **False Positives**: Rate of unnecessary change notifications

### Reporting Integration

```yaml
- name: Upload Chromatic results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: chromatic-report
    path: apps/storybook/chromatic-report.json
```

## Maintenance

### Regular Tasks

1. **Token Rotation**: Update `CHROMATIC_PROJECT_TOKEN` quarterly
2. **Action Updates**: Monitor and update Chromatic action versions
3. **Story Coverage**: Add visual tests for new components
4. **Performance Review**: Optimize build times monthly

### Version Management

```bash
# Check for Chromatic updates
npm outdated chromatic

# Update Chromatic
npm update chromatic

# Test locally
npm run chromatic -- --project-token=$CHROMATIC_PROJECT_TOKEN
```

## Troubleshooting Guide

### Debug Mode

```yaml
- name: Run Chromatic (Debug)
  uses: chromaui/action@v13
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    workingDir: apps/storybook
    debug: true
    diagnostics: true
```

### Common Error Resolution

| Error | Cause | Solution |
|-------|-------|----------|
| `Build failed` | Storybook build error | Test locally with `npm run build-storybook` |
| `No stories found` | Incorrect story paths | Check `.storybook/main.ts` configuration |
| `Snapshot timeout` | Slow component rendering | Increase timeout or optimize components |
| `Token invalid` | Expired or incorrect token | Regenerate token in Chromatic dashboard |
| `Working directory not found` | Incorrect path | Verify `workingDir` points to storybook app |

### Support Resources

1. **Chromatic Documentation**: https://www.chromatic.com/docs/
2. **GitHub Action Repository**: https://github.com/chromaui/action
3. **Community Support**: Chromatic Discord community
4. **Enterprise Support**: Contact Chromatic support team