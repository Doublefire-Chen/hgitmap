There are two methods to authenticate GitLab accounts in hgitmap: OAuth and Personal Access Tokens (PATs). This document provides detailed instructions for both methods.
## GitLab OAuth

**Steps:**
1. Create a GitLab OAuth App at https://gitlab.com/-/user_settings/applications or your domain and record your Application ID and Secret.
   - **Name:** Hgitmap
   - **Redirect URI:** Y`{BACKEND_URL}/oauth/gitlab/callback` (replace `{BACKEND_URL}` with your backend URL, e.g., `http://localhost:8000` or your domain)
   Scopes:
   - `read_user`
   - `read_api`
   - `read_repository`
2. In Hgitmap admin panel (`settings/platforms?tab=oauth-apps`), add a new OAuth app with the Client ID, Client Secret
3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click gitlab OAuth card and authorize the app

## GitLab Personal Access Token
**Steps:**
1. Go to https://gitlab.com/-/user_settings/personal_access_tokens or your domain and click "Add new token"

2. Select the required scopes as follows:
   - `read_user`
   - `read_api`
   - `read_repository`

3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click gitlab PAT card and enter the generated token