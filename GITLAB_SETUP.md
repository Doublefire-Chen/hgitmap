# GitLab Setup Guide

This guide explains how to set up both **official GitLab.com** and **self-hosted GitLab** instances with hgitmap.

## Table of Contents

- [GitLab.com (Official) Setup](#gitlabcom-official-setup)
- [Self-Hosted GitLab Setup](#self-hosted-gitlab-setup)
- [How It Works](#how-it-works)
- [Troubleshooting](#troubleshooting)

---

## GitLab.com (Official) Setup

### OAuth Setup for gitlab.com

**Step 1: Create OAuth Application on GitLab.com**

1. Go to https://gitlab.com/-/user_settings/applications
2. Click **"Add new application"**
3. Fill in the details:
   - **Name**: `hgitmap`
   - **Redirect URI**: `http://localhost:3000/oauth/gitlab/callback`
     - For production: `https://yourdomain.com/oauth/gitlab/callback`
   - **Confidential**: ✓ Check this
   - **Scopes**:
     - ✅ `read_user`
     - ✅ `read_api`
4. Click **"Save application"**
5. Copy the **Application ID** and **Secret**

**Step 2: Add OAuth App to hgitmap Admin Panel**

1. Go to `http://localhost:5173/admin/oauth-apps`
2. Click **"Add OAuth App"**
3. Fill in:
   - **Platform**: GitLab
   - **Instance Name**: `GitLab.com` (or any name you prefer)
   - **Instance URL**: `https://gitlab.com` (explicit URL required)
   - **Client ID**: Paste the Application ID from GitLab
   - **Client Secret**: Paste the Secret from GitLab
   - **Is Default**: ✓ Check this
   - **Is Enabled**: ✓ Check this
4. Click **"Save"**

**OAuth Scopes:** When users authorize the OAuth app, GitLab will grant the scopes you configured in Step 1 (`read_user` and `read_api`). For OAuth apps, you should request the same scopes as PAT tokens for consistency.

**Step 3: Connect Your Account**

1. Go to the Platforms page
2. Click **"Connect GitLab with OAuth"**
3. You should see "Connecting to: GitLab.com"
4. Click **"Connect"**
5. Authorize on GitLab.com
6. You'll be redirected back to hgitmap

### Personal Access Token (PAT) for gitlab.com

**Step 1: Create Personal Access Token**

1. Go to https://gitlab.com/-/user_settings/personal_access_tokens
2. Click **"Add new token"**
3. Fill in:
   - **Token name**: `hgitmap`
   - **Expiration date**: Choose an expiration (optional)
   - **Scopes** (select exactly these 3):
     - ✅ `read_user` - Profile information and user data
     - ✅ `read_api` - Read access to API endpoints (events, projects, groups)
     - ✅ `read_repository` - Access to private repository contributions
4. Click **"Create personal access token"**
5. Copy the token (starts with `glpat-`)

**Why these scopes?**
- `read_user`: Validates token and fetches profile data (username, avatar, bio, location)
- `read_api`: Grants read access to GitLab API for fetching contribution history and projects
- `read_repository`: Ensures contributions from **private repositories** are included in your heatmap

**Step 2: Connect to hgitmap**

1. Go to the Platforms page
2. Click **"Connect with Personal Access Token"**
3. Select **"GitLab"** from the platform dropdown
4. Enter **Instance URL**: `https://gitlab.com` (explicit URL required)
5. Paste your token (starts with `glpat-`)
6. Click **"Connect"**

---

## Self-Hosted GitLab Setup

### OAuth Setup for Self-Hosted GitLab

**Step 1: Create OAuth Application on Your GitLab Instance**

1. Go to `https://gitlab.example.com/-/user_settings/applications`
   - Replace `gitlab.example.com` with your GitLab instance URL
2. Click **"Add new application"**
3. Fill in the details:
   - **Name**: `hgitmap`
   - **Redirect URI**: `http://localhost:3000/oauth/gitlab/callback`
     - For production: `https://yourdomain.com/oauth/gitlab/callback`
   - **Confidential**: ✓ Check this
   - **Scopes**:
     - ✅ `read_user`
     - ✅ `read_api`
4. Click **"Save application"**
5. Copy the **Application ID** and **Secret**

**Step 2: Add OAuth App to hgitmap Admin Panel**

1. Go to `http://localhost:5173/admin/oauth-apps`
2. Click **"Add OAuth App"**
3. Fill in:
   - **Platform**: GitLab
   - **Instance Name**: `Company GitLab` (or any descriptive name)
   - **Instance URL**: `https://gitlab.example.com` (your full GitLab URL)
   - **Client ID**: Paste the Application ID from your GitLab
   - **Client Secret**: Paste the Secret from your GitLab
   - **Is Default**: ✓ Check this if it's your primary GitLab
   - **Is Enabled**: ✓ Check this
4. Click **"Save"**

**Step 3: Connect Your Account**

1. Go to the Platforms page
2. Click **"Connect GitLab with OAuth"**
3. Select your self-hosted instance from the dropdown (if you have multiple)
4. Click **"Connect"**
5. Authorize on your GitLab instance
6. You'll be redirected back to hgitmap

### Personal Access Token (PAT) for Self-Hosted GitLab

**Step 1: Create Personal Access Token**

1. Go to `https://gitlab.example.com/-/user_settings/personal_access_tokens`
   - Replace `gitlab.example.com` with your GitLab instance URL
2. Click **"Add new token"**
3. Fill in:
   - **Token name**: `hgitmap`
   - **Expiration date**: Choose an expiration (optional)
   - **Scopes** (select exactly these 3):
     - ✅ `read_user` - Profile information and user data
     - ✅ `read_api` - Read access to API endpoints (events, projects, groups)
     - ✅ `read_repository` - Access to private repository contributions
4. Click **"Create personal access token"**
5. Copy the token (starts with `glpat-`)

**Why these scopes?**
- `read_user`: Validates token and fetches profile data (username, avatar, bio, location)
- `read_api`: Grants read access to GitLab API for fetching contribution history and projects
- `read_repository`: Ensures contributions from **private repositories** are included in your heatmap

**Step 2: Connect to hgitmap**

1. Go to the Platforms page
2. Click **"Connect with Personal Access Token"**
3. Select **"GitLab"** from the platform dropdown
4. Enter your **Instance URL**: `https://gitlab.example.com`
5. Paste your token (starts with `glpat-`)
6. Click **"Connect"**

---

## How It Works

### Instance URL Handling

hgitmap requires explicit instance URLs for all GitLab instances, including the official gitlab.com:

| Instance Type | Instance URL Value | Example |
|--------------|-------------------|---------|
| **GitLab.com (Official)** | Full URL | `https://gitlab.com` |
| **Self-Hosted GitLab** | Full URL | `https://gitlab.company.com` |

**Important:** All GitLab connections require an explicit instance URL. This ensures consistency and clarity when managing multiple GitLab accounts.

### Database Storage

```sql
-- GitLab.com example
instance_url = 'https://gitlab.com'

-- Self-hosted example
instance_url = 'https://gitlab.company.com'
```

### Backend API Endpoints

The backend requires explicit instance URLs for all GitLab connections:

```rust
// OAuth Authorize (instance_url parameter is required)
GET /oauth/gitlab/authorize?instance_url=https://gitlab.com          // For gitlab.com
GET /oauth/gitlab/authorize?instance_url={URL}                       // For self-hosted

// OAuth Callback
GET /oauth/gitlab/callback?code={code}&state={state}
```

### Frontend Behavior

**OAuth Flow:**
1. Fetches all configured GitLab OAuth apps from admin panel
2. Displays dropdown if multiple instances are configured
3. Auto-selects default instance (usually gitlab.com)
4. User clicks "Connect" and is redirected to the appropriate GitLab instance

**PAT Flow:**
1. User selects "GitLab" platform
2. Instance URL field appears (required for all GitLab instances)
3. User enters full URL (e.g., `https://gitlab.com` or `https://gitlab.company.com`)
4. Backend validates and connects to the specified instance

---

## Multiple GitLab Instances

You can configure **multiple GitLab instances** simultaneously:

### Example Configuration

1. **GitLab.com** (official)
   - Instance URL: `https://gitlab.com`
   - Instance Name: `GitLab.com`
   - Is Default: ✓

2. **Company GitLab** (self-hosted)
   - Instance URL: `https://gitlab.company.com`
   - Instance Name: `Company GitLab`
   - Is Default: ☐

3. **Project GitLab** (another self-hosted)
   - Instance URL: `https://gitlab.project.org`
   - Instance Name: `Project GitLab`
   - Is Default: ☐

When connecting via OAuth, users will see a dropdown to select which instance they want to connect to.

---

## API Differences

### GitLab.com
- API Base URL: `https://gitlab.com/api/v4`
- OAuth URL: `https://gitlab.com/oauth/authorize`
- Token URL: `https://gitlab.com/oauth/token`

### Self-Hosted GitLab
- API Base URL: `https://gitlab.example.com/api/v4`
- OAuth URL: `https://gitlab.example.com/oauth/authorize`
- Token URL: `https://gitlab.example.com/oauth/token`

hgitmap automatically constructs the correct URLs based on the `instance_url`.

---

## Troubleshooting

### "No GitLab OAuth apps configured"

**Problem:** When clicking "Connect GitLab with OAuth", you see an error message.

**Solution:**
1. Ensure you've created an OAuth app in the admin panel (`/admin/oauth-apps`)
2. Check that the OAuth app is enabled (`is_enabled = true`)
3. Verify the platform is set to "gitlab"

### "Invalid instance URL"

**Problem:** Error when trying to connect with instance URL.

**Solution:**
1. Ensure the URL is complete: `https://gitlab.example.com` (not `gitlab.example.com`)
2. Don't include trailing slashes: `https://gitlab.example.com` (not `https://gitlab.example.com/`)
3. Verify the URL is reachable from your hgitmap backend server

### "OAuth redirect URI mismatch"

**Problem:** OAuth flow fails with redirect URI error.

**Solution:**
1. Check the redirect URI in your GitLab OAuth application settings
2. Must match exactly: `http://localhost:3000/oauth/gitlab/callback`
3. For production, use your actual domain: `https://yourdomain.com/oauth/gitlab/callback`

### "Failed to validate GitLab token"

**Problem:** PAT connection fails with token validation error.

**Solution:**
1. Verify token has correct scopes: `read_user` and `read_api`
2. Check token hasn't expired
3. For self-hosted: Ensure hgitmap backend can reach the GitLab instance

### "No contributions showing"

**Problem:** Connected but no contribution data appears.

**Solution:**
1. Click **"Sync"** button to manually trigger data fetch
2. Check that you have contributions in the selected date range
3. GitLab Events API has limitations - older events might not be available
4. Review backend logs for sync errors

---

## Security Notes

1. **OAuth is recommended** over PAT for better security
2. **Tokens are encrypted** at rest using AES-256 encryption
3. **Use HTTPS** for self-hosted instances in production
4. **Rotate tokens regularly** if using PAT
5. **Minimal scopes**: Only `read_user`, `read_api`, and `read_repository` are required
   - These are **read-only** permissions
   - hgitmap **cannot** modify your repositories or settings
   - Follows the principle of least privilege

---

## Supported GitLab Versions

- **GitLab.com**: Always supported (latest version)
- **Self-Hosted**: GitLab CE/EE 12.0+ (API v4)

---

## Required Scopes Reference

### Personal Access Token (PAT) Scopes

When creating a GitLab Personal Access Token, select **exactly these 3 scopes**:

| Scope | Purpose | Why Required |
|-------|---------|--------------|
| `read_user` | Profile access | Validates token and fetches your profile information (username, email, avatar, bio, location, company) |
| `read_api` | API read access | Grants read-only access to GitLab API endpoints for fetching contribution history, events, and project metadata |
| `read_repository` | Private repo access | Ensures contributions from **private repositories** are included. Without this, only public repository contributions may be visible |

**Important Notes:**
- ✅ All scopes are **read-only** - hgitmap cannot modify your data
- ✅ Minimum permissions required for full functionality
- ❌ Do **not** select `api` or `write_*` scopes (too permissive)
- ❌ Do **not** select registry, runner, or k8s scopes (not needed)

### OAuth Application Scopes

When creating a GitLab OAuth Application, configure the same scopes in the application settings:
- `read_user`
- `read_api`

**Note:** OAuth apps don't have a separate `read_repository` scope selection in the UI. The `read_api` scope for OAuth applications includes repository read access when authorized by users.

---

*Last updated: 2025-12-25*
