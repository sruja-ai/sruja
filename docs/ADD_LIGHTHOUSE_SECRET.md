# Adding Lighthouse CI Secret to GitHub

## 🔐 Quick Guide

If you've acquired a Lighthouse CI key (GitHub App token), you can add it as either:
- **Repository secret** (only for this repo)
- **Organization secret** (shared across multiple repos) ✅ Recommended if you have multiple repos

## 📍 Option 1: Organization Secret (Recommended for Multiple Repos)

### When to Use:
- ✅ You have multiple repositories
- ✅ Want to share the same secret
- ✅ Easier to manage centrally
- ✅ Organization admin access required

### Step-by-Step:

#### 1. Go to Organization Settings

1. **Go to your GitHub organization** (e.g., `https://github.com/sruja-ai`)
2. **Click "Settings"** (top navigation, right side)
3. **In left sidebar**, click **"Secrets and variables"**
4. **Click "Actions"** (under Secrets and variables)

#### 2. Add New Organization Secret

1. **Click "New organization secret"** button (top right)
2. **Name**: Enter `LHCI_GITHUB_APP_TOKEN`
   - ⚠️ **Important**: Must be exactly this name (case-sensitive)
3. **Secret**: Paste your Lighthouse CI key/token
4. **Repository access**: Choose:
   - **All repositories** (easiest)
   - **Selected repositories** (choose specific repos)
5. **Click "Add secret"**

#### 3. Verify Secret

1. You should see `LHCI_GITHUB_APP_TOKEN` in the organization secrets list
2. The value will be hidden (shown as `••••••••`)
3. Shows which repositories have access
4. You can edit or delete it anytime

### Benefits:
- ✅ Share across multiple repos
- ✅ Single place to update
- ✅ Better for teams/organizations
- ✅ Can restrict to specific repos if needed

## 📍 Option 2: Repository Secret (Single Repo Only)

### When to Use:
- ✅ Single repository only
- ✅ Repo-specific secret needed
- ✅ Personal repositories

### Step-by-Step:

#### 1. Go to Repository Settings

1. **Open your GitHub repository**
2. **Click "Settings"** (top navigation, right side)
3. **In left sidebar**, click **"Secrets and variables"**
4. **Click "Actions"** (under Secrets and variables)

#### 2. Add New Repository Secret

1. **Click "New repository secret"** button (top right)
2. **Name**: Enter `LHCI_GITHUB_APP_TOKEN`
   - ⚠️ **Important**: Must be exactly this name (case-sensitive)
3. **Secret**: Paste your Lighthouse CI key/token
4. **Click "Add secret"**

#### 3. Verify Secret

1. You should see `LHCI_GITHUB_APP_TOKEN` in the secrets list
2. The value will be hidden (shown as `••••••••`)
3. You can edit or delete it anytime

## 🔧 Update Workflow (Already Done!)

The workflow is already configured to use the secret. You just need to uncomment it:

### Current Status:
```yaml
# Optional: If you want to store results in GitHub, uncomment:
# env:
#   LHCI_GITHUB_APP_TOKEN: ${{ secrets.LHCI_GITHUB_APP_TOKEN }}
```

### After Adding Secret:

The workflow will automatically use it once you uncomment those lines. However, the current setup works fine without it - results are still saved as artifacts.

## ✅ What Happens After Adding Secret

Once the secret is added and workflow updated:

1. **Results stored in GitHub**:
   - Lighthouse results visible in GitHub UI
   - Historical data tracking
   - Trends over time

2. **Better integration**:
   - Results appear in GitHub Actions UI
   - Can compare runs
   - Better reporting

3. **Still works without it**:
   - Current setup saves results as artifacts
   - Temporary public storage links available
   - No functionality lost

## 🎯 Alternative: Keep Using Artifacts

You don't *need* to add the secret - the current setup works perfectly:

- ✅ Results saved as artifacts
- ✅ Downloadable HTML reports
- ✅ Temporary public storage links
- ✅ All free, no secrets needed

The secret is only needed if you want:
- Results stored in GitHub (not just artifacts)
- Historical tracking in GitHub UI
- Better integration with GitHub Actions

## 📝 Quick Reference

**Secret Name**: `LHCI_GITHUB_APP_TOKEN`  
**Location**: Repository Settings → Secrets and variables → Actions  
**Required**: No (optional enhancement)  
**Benefit**: Results stored in GitHub, better UI integration

## 🔗 Where to Find Settings

### For Organization Secret:
**Direct path:**
- Organization → Settings → Secrets and variables → Actions → New organization secret

**URL format:**
- `https://github.com/organizations/[org]/settings/secrets/actions`
- Example: `https://github.com/organizations/sruja-ai/settings/secrets/actions`

### For Repository Secret:
**Direct path:**
- Repository → Settings → Secrets and variables → Actions → New repository secret

**URL format:**
- `https://github.com/[org]/[repo]/settings/secrets/actions`
- Example: `https://github.com/sruja-ai/sruja/settings/secrets/actions`

## ⚠️ Security Notes

- Secrets are encrypted
- Only visible to repository admins
- Can't view secret value after saving (only edit/delete)
- Safe to use in public repositories
- Not visible in logs

## 🚀 Next Steps

1. ✅ Choose: Organization secret (recommended) or Repository secret
2. ✅ Add secret: `LHCI_GITHUB_APP_TOKEN`
3. ✅ Workflow already configured to use it automatically
4. ✅ Run workflow to test
5. ✅ Check results in GitHub UI

## 💡 Which Should You Choose?

### Use Organization Secret If:
- ✅ You have multiple repositories
- ✅ Want to share the secret
- ✅ Organization admin (or have permissions)
- ✅ Want centralized management

### Use Repository Secret If:
- ✅ Single repository only
- ✅ Personal repository
- ✅ Different secret per repo
- ✅ No organization admin access

## 📊 Comparison

| Feature | Organization Secret | Repository Secret |
|---------|---------------------|-------------------|
| **Scope** | Multiple repos | Single repo |
| **Management** | Centralized | Per-repo |
| **Access Control** | Can restrict to repos | Repo-only |
| **Best For** | Teams/Organizations | Single projects |
| **Permissions** | Org admin needed | Repo admin needed |

## 📚 Related

- [How to Check Lighthouse Results](./HOW_TO_CHECK_LIGHTHOUSE_RESULTS.md)
- [Lighthouse CI Free Guide](./LIGHTHOUSE_CI_FREE.md)
- [GitHub Secrets Documentation](https://docs.github.com/en/actions/security-guides/encrypted-secrets)

