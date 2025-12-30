There are two methods to authenticate Gitea accounts in hgitmap: OAuth and Personal Access Tokens (PATs). This document provides detailed instructions for both methods.
## Gitea OAuth

**Steps:**
1. Create a Gitea OAuth App at your Gitea instance's settings https://gitea.domain/user/settings/applications and record your Client ID and Client Secret.
   - **Application name:** Hgitmap
   - **Redirect URIs** `{BACKEND_URL}/oauth/gitea/callback` (replace `{BACKEND_URL}` with your backend URL, e.g., `http://localhost:8000` or your domain)
2. In Hgitmap admin panel (`settings/platforms?tab=oauth-apps`), add a new OAuth app with the Client ID, Client Secret
3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click gitea OAuth card and authorize the app

## Gitea Personal Access Token

**Steps:**
1. Go to https://gitea.domain/user/settings/applications and click "Generate token"

2. Select the required scopes as follows:
   - read:repository - Read repository data
   - read:user - Read user profile data
   - read:organization - Read organization data


3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click gitea PAT card and enter the generated token