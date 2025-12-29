# GitHub Authentication Setup

This document explains how to connect your GitHub account to hgitmap using OAuth or Personal Access Tokens.

## Table of Contents

- [GitHub OAuth (Recommended)](#github-oauth-recommended)
- [GitHub Personal Access Token](#github-personal-access-token)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

---

## GitHub OAuth (Recommended)

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

---

## GitHub Personal Access Token

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

## Troubleshooting

### "Failed to start OAuth flow"

**Problem:** Error when clicking OAuth connect button

**Solutions:**
- Ensure OAuth app is configured in admin panel (`/admin/oauth-apps`)
- Verify OAuth app is enabled (`is_enabled = true`)
- Check that redirect URI matches exactly: `{BACKEND_URL}/oauth/github/callback`
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
- Check token hasn't expired
- Ensure backend can reach the GitHub API

### "No contributions showing"

**Problem:** Platform connected but no contributions displayed

**Solutions:**
- Click **"Sync"** button to manually trigger data fetch
- Ensure `repo` scope is granted (not just `public_repo`)
- Check that you have contributions in the selected date range
- Review backend logs for sync errors

---

## FAQ

**Q: What's the difference between OAuth and Personal Access Token?**

A: OAuth is more secure and user-friendly - tokens are managed by GitHub and can be easily revoked. PATs require manual creation and management but work without OAuth app configuration.

**Q: Can I use both OAuth and PAT?**

A: Yes, but you can only have one active connection per platform account. If you connect via PAT after OAuth, it will update the existing connection.

**Q: How do I revoke access?**

A:
- **For OAuth**: Go to GitHub Settings → Applications → Authorized OAuth Apps and revoke the application
- **For PAT**: Delete the token from GitHub Settings → Developer settings → Personal access tokens
- **In hgitmap**: Click "Remove" button on the platform card

**Q: Does hgitmap store my tokens?**

A: Yes, tokens are encrypted with AES-256 encryption and stored securely in the database. Only encrypted values are stored.

---

## Security Best Practices

1. **Use OAuth when possible** - OAuth is more secure than Personal Access Tokens
2. **Minimize token scopes** - Only grant the minimum required scopes
3. **Rotate tokens regularly** - If using PAT, rotate them periodically
4. **Never share tokens** - Tokens are like passwords, keep them secret
5. **Revoke unused tokens** - Remove tokens for platforms you no longer use

---

## Reference Links

- **GitHub Token Creation**: https://github.com/settings/tokens/new
- **GitHub OAuth Apps**: https://github.com/settings/developers
- **GitHub API Documentation**: https://docs.github.com/en/rest

---

*Last updated: 2025-12-29*
