# Gitea Personal Access Token Permissions Guide

**For hgitmap Users**
**Last Updated:** 2025-01-24

---

## Quick Answer

When creating a Gitea Personal Access Token for hgitmap, select **only these 4 read permissions**:

```
✅ user: read
✅ repository: read
✅ issue: read
✅ organization: read
```

**That's it!** These minimal permissions give hgitmap everything it needs.

---

## Step-by-Step: Creating a Gitea Token for hgitmap

### 1. Navigate to Token Settings

1. Log in to your Gitea instance
2. Click your **profile picture** (top right)
3. Click **Settings**
4. Click **Applications** in the left sidebar
5. Scroll to **Manage Access Tokens**
6. Click **Generate New Token**

### 2. Configure Token

**Token Name:** `hgitmap` (or any descriptive name)

**Select Permissions:** Check these boxes:

```
[ ] activitypub          ← Leave unchecked
    [ ] read
    [ ] write

[✓] issue                ← CHECK the main checkbox
    [✓] read             ← CHECK read
    [ ] write            ← Leave unchecked

[ ] misc                 ← Leave unchecked

[ ] notification         ← Leave unchecked

[✓] organization         ← CHECK the main checkbox
    [✓] read             ← CHECK read
    [ ] write            ← Leave unchecked

[ ] package              ← Leave unchecked

[✓] repository           ← CHECK the main checkbox
    [✓] read             ← CHECK read
    [ ] write            ← Leave unchecked

[✓] user                 ← CHECK the main checkbox
    [✓] read             ← CHECK read
    [ ] write            ← Leave unchecked
```

### 3. Generate and Copy Token

1. Click **Generate Token**
2. **Copy the token immediately** (it will only be shown once!)
3. Keep it safe - you'll paste it into hgitmap

---

## Permission Details

### ✅ Required Permissions (Read-Only)

| Permission | What hgitmap Uses It For |
|------------|-------------------------|
| **user: read** | • User profile information<br>• Avatar URL<br>• Display name, bio<br>• Location, company<br>• Follower/following counts |
| **repository: read** | • List of repositories<br>• Repository creation dates<br>• Public/private status<br>• Primary programming language<br>• Contribution data from repo activities |
| **issue: read** | • Pull request activities<br>• Issue creation/comment activities<br>• Timeline events for activity feed |
| **organization: read** | • Organization memberships<br>• Organization-related activities<br>• Organization avatar/name |

### ❌ NOT Required Permissions

| Permission | Why You Don't Need It |
|------------|----------------------|
| **activitypub** | ActivityPub is for federated social networking (like Mastodon). hgitmap doesn't use federated features. |
| **notification** | Notifications are personal alerts/inbox. hgitmap displays activities, not notifications. |
| **package** | Package registry access (npm, Docker, etc.). hgitmap doesn't track package data. |
| **misc** | Miscellaneous/unclear purpose. Best practice: don't grant unknown permissions. |

### ❌ Write Permissions

**Never select "write" permissions for any scope!**

hgitmap is **read-only**:
- ✅ Reads your contribution data
- ✅ Reads your profile
- ✅ Reads your activities
- ❌ Never creates commits
- ❌ Never modifies repositories
- ❌ Never creates issues/PRs
- ❌ Never changes settings

**All "write" checkboxes should remain unchecked.**

---

## Security Best Practices

### Principle of Least Privilege

Only grant the minimum permissions needed:
- ✅ **Read-only** access
- ✅ **Limited to 4 scopes** (user, repository, issue, organization)
- ✅ **No write access**
- ✅ **No unnecessary scopes**

### Token Management

1. **Never share your token** - Treat it like a password
2. **Revoke if compromised** - Go to Settings → Applications → Delete token
3. **Regenerate periodically** - Create new token, delete old one
4. **Use descriptive names** - Name tokens by purpose (e.g., "hgitmap")
5. **Review active tokens** - Regularly check Settings → Applications

### What If Token Is Leaked?

If your token is accidentally exposed:

1. **Revoke immediately:**
   - Gitea → Settings → Applications
   - Find the token → Click **Delete**

2. **Generate new token:**
   - Follow the steps above
   - Use new token in hgitmap

3. **Monitor activity:**
   - Check recent repository activities
   - Review audit logs (if available)

**Impact of leaked read-only token:**
- ⚠️ Attacker can read your data (profiles, repos, activities)
- ✅ Attacker CANNOT modify anything (read-only)
- ✅ No write access = Limited damage

---

## Troubleshooting

### "Invalid token" Error

**Causes:**
- Token copied incorrectly (extra spaces, incomplete)
- Token revoked in Gitea
- Wrong permissions selected

**Solution:**
1. Verify token was copied completely
2. Check token exists in Gitea Settings → Applications
3. Regenerate token with correct permissions
4. Try again in hgitmap

### "Failed to fetch contributions" Error

**Causes:**
- Missing `repository: read` permission
- Token doesn't have access to private repos

**Solution:**
1. Verify `repository: read` is checked
2. Regenerate token if needed
3. Ensure you have access to the repositories in Gitea

### "Failed to fetch profile" Error

**Causes:**
- Missing `user: read` permission

**Solution:**
1. Verify `user: read` is checked
2. Regenerate token if needed

### "Failed to fetch activities" Error

**Causes:**
- Missing `issue: read` or `repository: read` permission

**Solution:**
1. Verify both `issue: read` and `repository: read` are checked
2. Regenerate token if needed

---

## Comparison with GitHub Token Scopes

For reference, here's how Gitea permissions map to GitHub:

| Gitea Permission | GitHub Equivalent | Purpose |
|-----------------|------------------|---------|
| `user: read` | `read:user` | User profile data |
| `repository: read` | `repo` (read access) | Repository data |
| `issue: read` | `repo` (includes issues/PRs) | PR and issue data |
| `organization: read` | `read:org` | Organization data |

**Note:** GitHub's `repo` scope is broader and includes issues/PRs, while Gitea separates them.

---

## Example: Complete Token Creation

Here's a complete walkthrough with screenshots descriptions:

### Screen 1: Applications Page
```
Settings → Applications

Manage Access Tokens
────────────────────────────────────
Generate new token to access Gitea API

[Generate New Token]
```

### Screen 2: Token Creation Form
```
Generate New Access Token
────────────────────────────────────

Token Name: hgitmap

Select permissions:

☐ activitypub
   ☐ read  ☐ write

☑ issue
   ☑ read  ☐ write          ← Select these

☐ misc

☐ notification

☑ organization
   ☑ read  ☐ write          ← Select these

☐ package

☑ repository
   ☑ read  ☐ write          ← Select these

☑ user
   ☑ read  ☐ write          ← Select these

[Generate Token]
```

### Screen 3: Token Generated
```
✅ Successfully generated token!

Your new access token:
────────────────────────────────────
a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
────────────────────────────────────

⚠️ Make sure to copy your personal access token now.
   You won't be able to see it again!

[Copy Token] [I have copied my token]
```

**Important:** Click **Copy Token** and paste it somewhere safe!

---

## Using Token in hgitmap

After generating your token:

1. **Open hgitmap** at http://localhost:5173
2. **Go to Settings** → Connected Platforms
3. **Click** "Connect with Personal Access Token"
4. **Select Platform:** Gitea
5. **Enter Instance URL:** `https://your-gitea-instance.com`
6. **Paste Token:** (the token you just copied)
7. **Click Connect**

hgitmap will validate the token and connect to your Gitea account!

---

## FAQ

### Q: Can I use the same token for multiple instances of hgitmap?

**A:** Yes! The token works anywhere you use it. However, for security:
- ✅ Use same token for development/production instances you control
- ❌ Don't share tokens between different services/apps

### Q: How long does the token last?

**A:** Gitea tokens don't expire by default, but check your instance settings. Some Gitea instances may enforce token expiration policies.

### Q: Can I have multiple tokens?

**A:** Yes! You can create multiple tokens with different permissions for different purposes:
- One for hgitmap (read-only)
- One for CI/CD (write access)
- One for scripts (specific scopes)

### Q: What if my Gitea instance requires 2FA?

**A:** Personal Access Tokens work with 2FA enabled. The token authenticates you directly, bypassing the 2FA login step for API access.

### Q: Can organization admins see my token?

**A:** No! Gitea only stores a hashed version. Even admins cannot see your token. If lost, you must regenerate it.

### Q: Will this token work with Gitea API directly?

**A:** Yes! This token works with Gitea's REST API. Example:
```bash
curl -H "Authorization: token YOUR_TOKEN" \
  https://gitea.example.com/api/v1/user
```

---

## Summary Checklist

Before connecting to hgitmap, verify:

- [ ] Created token in Gitea Settings → Applications
- [ ] Token name is descriptive (e.g., "hgitmap")
- [ ] Selected these 4 read permissions:
  - [ ] `user: read`
  - [ ] `repository: read`
  - [ ] `issue: read`
  - [ ] `organization: read`
- [ ] **NO write permissions selected**
- [ ] **NO unnecessary scopes selected**
- [ ] Copied token immediately (shown only once)
- [ ] Token stored safely
- [ ] Ready to paste into hgitmap

---

## Related Documentation

- [Authentication Methods Comparison](./AUTHENTICATION_COMPARISON.md) - OAuth vs PAT analysis
- [Gitea Official API Documentation](https://docs.gitea.com/development/api-usage)
- [Gitea OAuth2 Provider](https://docs.gitea.com/development/oauth2-provider)

---

## Support

If you encounter issues:

1. Check the Troubleshooting section above
2. Verify your token permissions in Gitea
3. Check hgitmap backend logs for errors
4. Regenerate token if needed

---

**Document Version:** 1.0
**Last Updated:** 2025-01-24
**Maintained by:** hgitmap development team
