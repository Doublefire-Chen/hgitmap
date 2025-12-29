# Gitea Authentication Setup

This document explains how to connect your Gitea account to hgitmap using OAuth or Personal Access Tokens.

## Table of Contents

- [Gitea OAuth (Recommended)](#gitea-oauth-recommended)
- [Gitea Personal Access Token](#gitea-personal-access-token)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

---

## Gitea OAuth (Recommended)

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

---

## Gitea Personal Access Token

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

**Token Location:**
```
https://your-gitea.com/user/settings/applications
```

---

## Troubleshooting

### "Failed to start OAuth flow"

**Problem:** Error when clicking OAuth connect button

**Solutions:**
- Ensure OAuth app is configured in admin panel (`/admin/oauth-apps`)
- Verify OAuth app is enabled (`is_enabled = true`)
- Check that redirect URI matches exactly: `{BACKEND_URL}/oauth/gitea/callback`
- Verify Gitea instance URL is correct and accessible
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
- Verify instance URL is correct and accessible (including https:// prefix)
- Check token hasn't expired
- Ensure backend can reach your Gitea instance API
- Check if your Gitea instance has a valid SSL certificate

### "No contributions showing"

**Problem:** Platform connected but no contributions displayed

**Solutions:**
- Click **"Sync"** button to manually trigger data fetch
- Ensure all required scopes are granted (`read:repository`, `read:user`, `read:organization`)
- Check that you have contributions in the selected date range
- Review backend logs for sync errors
- Verify your Gitea instance is accessible from the backend server

### Self-Hosted Instance Issues

**Problem:** Cannot connect to self-hosted Gitea instance

**Solutions:**
- Ensure the instance URL is accessible from the backend server
- Check firewall rules allow connections from the backend server
- Verify SSL certificate is valid (or configure backend to accept self-signed certificates)
- Test connection manually: `curl https://your-gitea.com/api/v1/version`

---

## FAQ

**Q: What's the difference between OAuth and Personal Access Token?**

A: OAuth is more secure and user-friendly - tokens are managed by Gitea and can be easily revoked. PATs require manual creation and management but work without OAuth app configuration.

**Q: Can I connect multiple Gitea instances?**

A: Yes, you can connect to multiple self-hosted Gitea instances. Each requires separate OAuth app configuration or PAT.

**Q: Can I use both OAuth and PAT?**

A: Yes, but you can only have one active connection per Gitea account. If you connect via PAT after OAuth, it will update the existing connection.

**Q: How do I revoke access?**

A:
- **For OAuth**: Go to your Gitea instance Settings → Applications → Authorized Applications and revoke the application
- **For PAT**: Delete the token from your Gitea instance Settings → Applications
- **In hgitmap**: Click "Remove" button on the platform card

**Q: Does hgitmap support Gitea Actions?**

A: Currently, hgitmap tracks commits, pull requests, and issues. Gitea Actions support may be added in future releases.

**Q: Does hgitmap store my tokens?**

A: Yes, tokens are encrypted with AES-256 encryption and stored securely in the database. Only encrypted values are stored.

---

## Security Best Practices

1. **Use OAuth when possible** - OAuth is more secure than Personal Access Tokens
2. **Minimize token scopes** - Only grant the minimum required scopes
3. **Rotate tokens regularly** - If using PAT, rotate them periodically
4. **Never share tokens** - Tokens are like passwords, keep them secret
5. **Revoke unused tokens** - Remove tokens for platforms you no longer use
6. **Use HTTPS** - Always use HTTPS for self-hosted instances
7. **Valid SSL certificates** - Use valid SSL certificates for production instances

---

## Reference Links

- **Gitea Documentation**: https://docs.gitea.com/
- **Gitea API Documentation**: https://docs.gitea.com/api/

---

*Last updated: 2025-12-29*
