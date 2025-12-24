# Authentication Methods Comparison: OAuth2 vs Personal Access Tokens

**Document Version:** 1.0
**Date:** 2025-01-24
**Research Focus:** Data accessibility and API capabilities for hgitmap

---

## Executive Summary

For **hgitmap's use case** (fetching contribution data, profiles, and activities), **Personal Access Tokens (PAT) provide equal or better data access** compared to OAuth2 on both GitHub and Gitea platforms.

### Key Findings

| Platform | OAuth2 Data Access | PAT Data Access | Winner for hgitmap |
|----------|-------------------|-----------------|-------------------|
| **GitHub** | Good (but org-restricted) | ✅ **Better** (full org access) | **PAT** |
| **Gitea** | ✅ Same (identical scopes) | ✅ Same (identical scopes) | **PAT** (simpler) |

---

## GitHub: OAuth2 vs Personal Access Token

### API Access Comparison

| Feature | OAuth2 Token | Personal Access Token | Notes |
|---------|--------------|----------------------|-------|
| **REST API** | ✅ Full access | ✅ Full access | Identical |
| **GraphQL API** | ✅ Full access | ✅ Full access | Identical |
| **Contribution Calendar** | ✅ Available | ✅ Available | Both use GraphQL |
| **User Profile** | ✅ Available | ✅ Available | Identical |
| **Repository Data** | ✅ Available | ✅ Available | Identical |
| **Private Repositories** | ✅ With `repo` scope | ✅ With `repo` scope | Identical |
| **Rate Limit** | 5,000 req/hour | 5,000 req/hour | Identical |

### ⚠️ Critical Difference: Organization Access

#### OAuth2 Apps
```
❌ BLOCKED if organization has "OAuth App access restrictions" enabled
✅ Works ONLY if organization owner approves the OAuth app
⚠️ Common in enterprise/company GitHub organizations
```

**Impact:** Users working at companies with OAuth restrictions **cannot access their work contributions** with OAuth apps.

#### Personal Access Tokens
```
✅ ALWAYS works (inherits user's permissions)
✅ NO organization approval needed
✅ User can access ANY organization they're a member of
```

**Source:** GitHub Documentation (2025)
- OAuth apps subject to organization access restrictions
- PATs are user-scoped and bypass OAuth app restrictions

### Token Types Available

#### 1. Classic Personal Access Tokens
- Broad scopes (repo, user, admin:org, etc.)
- Access to all repositories and organizations
- Legacy but still widely used

#### 2. Fine-Grained Personal Access Tokens (New)
- 50+ granular permissions
- Repository-specific access
- Read/Write basis per permission
- Recommended by GitHub (2025)

#### 3. OAuth2 Tokens
- User-approved scopes
- Third-party app delegation
- Subject to organization restrictions
- Auto-expiring (configurable)

### Rate Limits (2025)

All authentication methods have **identical rate limits**:
- **5,000 requests per hour** (authenticated)
- Scales with GitHub Apps (not relevant for hgitmap)

---

## Gitea: OAuth2 vs Personal Access Token

### API Access Comparison

| Feature | OAuth2 Token | Personal Access Token | Notes |
|---------|--------------|----------------------|-------|
| **REST API** | ✅ Identical | ✅ Identical | Same endpoints |
| **Contribution Data** | ✅ Available | ✅ Available | Activities API |
| **User Profile** | ✅ Available | ✅ Available | User API |
| **Repository Data** | ✅ Available | ✅ Available | Repos API |
| **Organizations** | ✅ Available | ✅ Available | No restrictions |
| **Granular Scopes** | ✅ Yes (v1.23+) | ✅ Yes (v1.23+) | Identical system |

### Key Finding: Identical Scope System

**Official Gitea Documentation (v1.23+):**
> "Access tokens obtained from Gitea's OAuth2 provider are accepted... OAuth2 supports the same granular scopes as in Personal Access Tokens."

#### Available Scopes (Both Methods)
```
activitypub:     read, write
admin:           read, write
issue:           read, write
notification:    read, write
organization:    read, write
package:         read, write
repository:      read, write
user:            read, write
```

#### OAuth2-Specific Scopes (OIDC)
```
openid
email
profile
groups
```

### Authentication Header Formats

#### Personal Access Token
```http
Authorization: token YOUR_TOKEN_HERE
```

#### OAuth2 Token
```http
Authorization: bearer YOUR_OAUTH_TOKEN_HERE
```

### Rate Limits

**Gitea does NOT document different rate limits** for OAuth2 vs PAT:
- Both use same API endpoints
- Both subject to `MAX_RESPONSE_ITEMS` (default: 50)
- No evidence of preferential treatment

**Conclusion:** Identical or non-existent rate limiting.

---

## Data Accessibility Analysis

### What Data Does hgitmap Need?

1. **User Profile Data**
   - Avatar, name, bio, location, company
   - Follower/following counts
   - ✅ Both methods: Full access

2. **Contribution Calendar/Heatmap**
   - Daily contribution counts
   - Repository associations
   - Private repository flags
   - ✅ Both methods: Full access (with appropriate scopes)

3. **Activity Timeline**
   - Commits, PRs, Issues
   - Repository creation events
   - Organization activity
   - ✅ PAT: Full access
   - ⚠️ OAuth: Blocked by org restrictions (GitHub)

4. **Repository Data**
   - Repository lists
   - Privacy status
   - Primary language
   - ✅ Both methods: Full access

### Winner: Personal Access Token

**Reasons:**
1. ✅ **Equal data access** on all endpoints
2. ✅ **Better organization support** (GitHub - no restrictions)
3. ✅ **Simpler implementation** (no OAuth app registration)
4. ✅ **Multi-instance friendly** (Gitea - works with any instance)
5. ✅ **Easier testing** (no callback URL needed)
6. ✅ **User control** (direct token management)

---

## Security Comparison

### Token Lifetime

| Aspect | OAuth2 | Personal Access Token |
|--------|--------|----------------------|
| **Expiration** | ✅ Auto-expires (configurable) | ⚠️ No expiration (unless configured) |
| **Refresh** | ✅ Auto-refresh supported | ❌ Manual regeneration |
| **Revocation** | User via "Authorized Apps" | User via "Tokens" settings |
| **Scope Changes** | Requires re-authorization | User creates new token |

### Attack Surface

**OAuth2:**
- Leaked token expires automatically
- Limited scope damage
- Third-party app trust required

**PAT:**
- Leaked token valid until revoked
- User manages directly
- No third-party involved

**Winner:** OAuth2 (but PAT acceptable with user education)

---

## Implementation Complexity

### GitHub OAuth2 Setup
```
1. Register OAuth app at github.com/settings/developers
2. Get Client ID and Secret
3. Configure callback URL (http://localhost:8080/oauth/github/callback)
4. Store credentials in .env
5. Implement OAuth flow (authorize → callback → token exchange)
6. Handle token refresh
7. User clicks "Connect with OAuth" → redirects → approves → redirects back

Time: ~30 minutes initial setup, ~5 minutes per user
```

### GitHub PAT Setup
```
1. User goes to github.com/settings/tokens
2. Click "Generate new token"
3. Select scopes (repo, read:user)
4. Copy token
5. Paste in hgitmap

Time: ~30 seconds per user
```

### Gitea OAuth2 Setup (Per Instance)
```
1. Admin registers OAuth app in Gitea
2. Get Client ID and Secret for THIS instance
3. Configure in hgitmap backend (.env variables)
4. Update callback URL to include instance
5. Implement instance-specific OAuth handling
6. User clicks "Connect" → redirects to THAT Gitea → approves → redirects back

Time: ~45 minutes per instance
```

### Gitea PAT Setup
```
1. User goes to gitea.instance.com/user/settings/applications
2. Click "Generate New Token"
3. Select scopes
4. Copy token
5. Paste in hgitmap with instance URL

Time: ~30 seconds per user
```

**Winner:** PAT (significantly simpler)

---

## Real-World Scenarios

### Scenario 1: Individual Developer
**Use Case:** Tracking personal contributions from github.com and personal Gitea

**Best Choice:** PAT
- Quick setup (1 minute total)
- No OAuth app registration needed
- Works immediately

### Scenario 2: Team Using Company GitHub
**Use Case:** Company with OAuth app restrictions enabled

**OAuth:** ❌ **BLOCKED** - Cannot access company org data
**PAT:** ✅ **WORKS** - Full access to company contributions

**Winner:** PAT (only option that works)

### Scenario 3: Self-Hosted Gitea Instance
**Use Case:** Personal or company Gitea at git.mycompany.com

**OAuth:** Requires admin to register app
**PAT:** Works immediately

**Winner:** PAT (no admin dependency)

### Scenario 4: Public Gitea Instance (try.gitea.io)
**Use Case:** Many users using same public Gitea

**OAuth:** Register once, all users benefit
**PAT:** Each user creates token

**Winner:** OAuth (better UX for shared instances) - but PAT still works fine

---

## Performance Comparison

### API Request Efficiency

| Aspect | OAuth2 | PAT | Notes |
|--------|--------|-----|-------|
| Request overhead | Same | Same | Both send Authorization header |
| Token size | Slightly larger | Smaller | Negligible difference |
| Rate limits | 5,000/hr (GitHub) | 5,000/hr (GitHub) | Identical |
| Rate limits | N/A (Gitea) | N/A (Gitea) | Both unlimited/same |

**Winner:** Tie (no measurable difference)

---

## Recommendation for hgitmap

### Current Implementation: ✅ OPTIMAL

**GitHub:**
- ✅ OAuth2 (primary) - Better UX for most users
- ✅ PAT (fallback) - Workaround for org restrictions

**Gitea:**
- ✅ PAT only - Simplicity, multi-instance support

### Why This Works Best

1. **Data Access:** PAT provides equal or better access
2. **Organization Support:** PAT bypasses GitHub org restrictions
3. **Simplicity:** PAT works immediately with any Gitea instance
4. **User Control:** Users manage their own tokens
5. **Testing:** Easier to develop and test locally
6. **Multi-Instance:** PAT scales to unlimited Gitea instances

### When to Add Gitea OAuth2

Consider adding OAuth2 for Gitea if:
- ❌ Public Gitea instance with many hgitmap users
- ❌ Want to provide "official" integration feeling
- ❌ Users demand one-click connect experience

**Current Status:** Not needed for data access

---

## Technical Details

### GitHub GraphQL Query (Both Methods)
```graphql
query($username: String!, $from: DateTime!, $to: DateTime!) {
  user(login: $username) {
    contributionsCollection(from: $from, to: $to) {
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            date
            contributionCount
          }
        }
      }
    }
  }
}
```

**Works identically with:**
- OAuth token: `Authorization: bearer <token>`
- PAT: `Authorization: bearer <token>` or `Authorization: token <token>`

### Gitea Activities API (Both Methods)
```http
GET /api/v1/users/{username}/activities/feeds
Authorization: token YOUR_TOKEN (PAT)
Authorization: bearer YOUR_TOKEN (OAuth)
```

**Returns identical data structure.**

---

## Sources & References

### GitHub Documentation (2025)
- Managing Personal Access Tokens: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens
- Differences Between GitHub Apps and OAuth Apps: https://docs.github.com/en/developers/apps/differences-between-github-apps-and-oauth-apps
- Fine-grained Personal Access Tokens: https://github.blog/security/application-security/introducing-fine-grained-personal-access-tokens-for-github/

### Gitea Documentation (v1.23+)
- API Usage: https://docs.gitea.com/development/api-usage
- OAuth2 Provider: https://docs.gitea.com/development/oauth2-provider
- API Reference: https://docs.gitea.com/api/1.21/

### Key Findings
1. OAuth apps subject to organization access restrictions (GitHub)
2. Gitea OAuth2 and PAT use identical scope system (v1.23+)
3. No rate limit differences documented for either platform
4. Both methods access same API endpoints with same capabilities

---

## Conclusion

**For hgitmap's primary goal (aggregating contribution data), Personal Access Tokens provide the optimal balance of:**
- ✅ Data accessibility (equal or better than OAuth)
- ✅ Simplicity (30 seconds vs 30 minutes setup)
- ✅ Compatibility (works everywhere, bypasses restrictions)
- ✅ User control (direct token management)

**The current implementation is architecturally sound and should remain PAT-focused.**

---

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-24 | Initial research and documentation |

---

**Maintained by:** hgitmap development team
**Last Updated:** 2025-01-24
