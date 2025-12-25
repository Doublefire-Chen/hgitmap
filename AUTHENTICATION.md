# Authentication Guide

This document explains how to connect your git platform accounts to hgitmap using OAuth or Personal Access Tokens.

## Table of Contents

- [GitHub Authentication](#github-authentication)
  - [GitHub OAuth](#github-oauth-recommended)
  - [GitHub Personal Access Token](#github-personal-access-token)
- [Gitea Authentication](#gitea-authentication)
  - [Gitea OAuth](#gitea-oauth-recommended)
  - [Gitea Personal Access Token](#gitea-personal-access-token)
- [GitLab Authentication](#gitlab-authentication)

---

## GitHub Authentication

### GitHub OAuth (Recommended)

OAuth is the recommended method for GitHub as it's more secure and easier to manage.

**Prerequisites:**
1. Admin must configure GitHub OAuth app in the admin panel at `/admin/oauth-apps`
2. OAuth app must have redirect URI: `{BACKEND_URL}/oauth/github/callback`
   - Example for development: `http://localhost:3000/oauth/github/callback`

**Steps:**
1. Click **"Connect GitHub with OAuth"** button on the Platforms page
2. You'll be redirected to GitHub for authorization
3. Click "Authorize" on GitHub
4. You'll be redirected back to hgitmap automatically

**Scopes used by OAuth:**
- `read:user` - Read user profile data
- `repo` - Access to repositories (including private repos)

### GitHub Personal Access Token

Personal Access Tokens provide an alternative authentication method.

**Required Scopes:**

When creating a GitHub Personal Access Token at https://github.com/settings/tokens/new:

1. **`repo`** - Full control of private repositories
   - ✅ Check the parent **`repo`** checkbox
   - This automatically includes all sub-options:
     - `repo:status` - Access commit status
     - `repo_deployment` - Access deployment status
     - `public_repo` - Access public repositories
     - `repo:invite` - Access repository invitations
     - `security_events` - Read and write security events

   **Alternative (Public repos only):**
   - If you only want to track public repository contributions, select only `public_repo` instead

2. **`read:user`** - Read ALL user profile data
   - ⚠️ **Important**: Expand the `user` section and select ONLY the `read:user` sub-option
   - ❌ Do NOT select the parent `user` checkbox (too broad - allows account modifications)
   - Other sub-options not needed:
     - `user:email` - Not required
     - `user:follow` - Not required

**Quick Link:**
```
https://github.com/settings/tokens/new?scopes=repo,read:user&description=hgitmap
```

**Steps:**
1. Click **"Connect with Personal Access Token"** button
2. Select **"GitHub"** from the platform dropdown
3. Click the **"Create token →"** link (opens GitHub with pre-selected scopes)
4. Generate the token on GitHub
5. Copy the token (starts with `ghp_`)
6. Paste it into hgitmap and click **"Connect"**

**Token Format:**
- Starts with `ghp_`
- Example: `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

**Why These Scopes?**

- **`repo`**: Required to read commit history and contributions from both public and private repositories. Without this, you'll only see public contributions.
- **`read:user`**: Required to fetch your GitHub username, avatar, and profile information when connecting the account.

---

## Gitea Authentication

### Gitea OAuth (Recommended)

OAuth is the recommended method for Gitea as it's more secure and easier to manage.

**Prerequisites:**

1. **On your Gitea instance:**
   - Go to **Settings** → **Applications** → **Manage OAuth2 Applications**
   - Click **"Create a new OAuth2 Application"**
   - Fill in:
     - **Application Name**: `hgitmap` (or your preferred name)
     - **Redirect URIs**: `{BACKEND_URL}/oauth/gitea/callback`
       - Example for development: `http://localhost:3000/oauth/gitea/callback`
   - Click **"Create Application"**
   - Copy the **Client ID** and **Client Secret**

2. **In hgitmap admin panel:**
   - Go to `/admin/oauth-apps`
   - Click **"Add OAuth App"**
   - Fill in:
     - **Platform**: Gitea
     - **Instance Name**: Your Gitea instance name (e.g., "My Gitea Server")
     - **Instance URL**: Your Gitea URL (e.g., `https://gitea.example.com`)
     - **Client ID**: (paste from Gitea)
     - **Client Secret**: (paste from Gitea)
     - **Is Default**: ✓ Check this if it's your primary Gitea instance

**Steps:**
1. Click **"Connect Gitea with OAuth"** button on the Platforms page
2. Select your Gitea instance (if you have multiple configured)
3. Click **"Connect"**
4. You'll be redirected to your Gitea instance for authorization
5. Click "Authorize" on Gitea
6. You'll be redirected back to hgitmap automatically

**Scopes used by OAuth:**
- `read:repository` - Read repository data
- `read:user` - Read user profile data
- `read:organization` - Read organization data

### Gitea Personal Access Token

Personal Access Tokens provide an alternative authentication method for Gitea.

**Required Scopes:**

When creating a Gitea Personal Access Token on your Gitea instance:

1. Go to your Gitea instance → **Settings** → **Applications** → **Generate New Token**
2. Select the following scopes:
   - ✅ `read:repository` - Read repository data
   - ✅ `read:user` - Read user profile data
   - ✅ `read:organization` - Read organization data

**Steps:**
1. Click **"Connect with Personal Access Token"** button
2. Select **"Gitea"** from the platform dropdown
3. Enter your Gitea instance URL (e.g., `https://gitea.example.com`)
4. Create a token on your Gitea instance with the required scopes
5. Copy the token
6. Paste it into hgitmap and click **"Connect"**

---

## GitLab Authentication

GitLab support is planned for a future release.

**Status:** 🚧 Coming soon

The implementation will support both:
- GitLab OAuth (gitlab.com and self-hosted instances)
- GitLab Personal Access Tokens

---

## Security Best Practices

1. **Use OAuth when possible** - OAuth is more secure than Personal Access Tokens as tokens can be revoked through the platform's UI
2. **Minimize token scopes** - Only grant the minimum required scopes
3. **Rotate tokens regularly** - If using PAT, rotate them periodically
4. **Never share tokens** - Tokens are like passwords, keep them secret
5. **Revoke unused tokens** - Remove tokens for platforms you no longer use

---

## Troubleshooting

### "Failed to start OAuth flow"

**Problem:** Error when clicking OAuth connect button

**Solutions:**
- Ensure OAuth app is configured in admin panel (`/admin/oauth-apps`)
- Verify OAuth app is enabled (`is_enabled = true`)
- Check that redirect URI matches exactly: `{BACKEND_URL}/oauth/{platform}/callback`
- Restart backend after adding OAuth app configuration

### "Invalid state token"

**Problem:** Error during OAuth callback

**Solutions:**
- State tokens expire after 10 minutes - try again
- Clear browser cookies and try again
- Check that your system clock is synchronized

### "Failed to connect platform"

**Problem:** Error when connecting with Personal Access Token

**Solutions:**
- Verify token has correct scopes (see sections above)
- For Gitea: Verify instance URL is correct and accessible
- Check token hasn't expired
- Ensure backend can reach the git platform API

### "No contributions showing"

**Problem:** Platform connected but no contributions displayed

**Solutions:**
- Click **"Sync"** button to manually trigger data fetch
- For GitHub: Ensure `repo` scope is granted (not just `public_repo`)
- Check that you have contributions in the selected date range
- Review backend logs for sync errors

---

## FAQ

**Q: What's the difference between OAuth and Personal Access Token?**

A: OAuth is more secure and user-friendly - tokens are managed by the platform and can be easily revoked. PATs require manual creation and management but work without OAuth app configuration.

**Q: Can I use both OAuth and PAT for the same platform?**

A: Yes, but you can only have one active connection per platform account. If you connect via PAT after OAuth, it will update the existing connection.

**Q: How do I revoke access?**

A:
- **For OAuth**: Go to your platform's settings and revoke the OAuth application
- **For PAT**: Delete the token from your platform's settings
- **In hgitmap**: Click "Remove" button on the platform card

**Q: How often does hgitmap sync my data?**

A: You can configure automatic sync intervals in Settings → Sync Settings. Manual sync is always available via the "Sync" button.

**Q: Does hgitmap store my tokens?**

A: Yes, tokens are encrypted with AES-256 encryption and stored securely in the database. Only encrypted values are stored.

---

## Reference Links

- **GitHub Token Creation**: https://github.com/settings/tokens/new
- **GitHub OAuth Apps**: https://github.com/settings/developers
- **Gitea Documentation**: https://docs.gitea.com/

---

*Last updated: 2025-12-25*
