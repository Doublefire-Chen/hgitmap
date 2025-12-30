There are two methods to authenticate GitHub accounts in hgitmap: OAuth and Personal Access Tokens (PATs). This document provides detailed instructions for both methods.
## GitHub OAuth

**Steps:**
1. Create a GitHub OAuth App at https://github.com/settings/developers and record your Client ID and Client Secret.
   - **Application name:** Hgitmap
   - **Homepage URL:** Your hgitmap frontend URL (e.g., `http://localhost:3000` or your domain)
   - **Authorization callback URL:** `{BACKEND_URL}/oauth/github/callback` (replace `{BACKEND_URL}` with your backend URL, e.g., `http://localhost:8000` or your domain)
2. In Hgitmap admin panel (`settings/platforms?tab=oauth-apps`), add a new OAuth app with the Client ID, Client Secret
3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click github OAuth card and authorize the app

## GitHub Personal Access Token

**Steps:**
1. Go to https://github.com/settings/tokens and click "Generate new token"

2. Select the required scopes as follows:
    
   - **`repo`** - Full control of private repositories
   
     - ✅ Check the parent **`repo`** checkbox, this automatically includes all sub-options:
   
       - `repo:status` - Access commit status
       - `repo_deployment` - Access deployment status
       - `public_repo` - Access public repositories
       - `repo:invite` - Access repository invitations
       - `security_events` - Read and write security events
   
       **Alternative (Public repos only):**
   
     - If you only want to track public repository contributions, select only `public_repo` instead
   
   
   - Expand the `user` section and select ONLY the `read:user` sub-option, other sub-options not needed:
   
     - **`read:user`** - Read ALL user profile data
   
       - `user:email` - Not required
       - `user:follow` - Not required

3. In Hgitmap My platforms page(`settings/platforms?tab=platforms`), click github PAT card and enter the generated token