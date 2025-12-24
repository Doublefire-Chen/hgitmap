# Finding the Gitea Token Input in hgitmap Frontend

**Quick Guide for Users**

---

## 🎯 Where is the Token Input?

The Gitea token input is in the **Connected Platforms** section. Here's how to find it:

### Navigation Path

```
http://localhost:5173
  ↓
Login to your account
  ↓
Click "Settings" (top navigation)
  ↓
Click "Connected Platforms" tab
  ↓
Scroll down to "Connect Platform" section
  ↓
Click "Connect with Personal Access Token" button
  ↓
Select "Gitea" from dropdown
  ↓
Enter instance URL and token
```

---

## 📸 Visual Guide

### Screen 1: Settings Page
```
┌─────────────────────────────────────────────┐
│  hgitmap                            Settings │
├─────────────────────────────────────────────┤
│  [Overview] [Heatmap] [Activities]          │
│  [Settings] ← YOU ARE HERE                  │
├─────────────────────────────────────────────┤
│  Tabs:                                      │
│  [Connected Platforms] [Privacy] [Themes]   │
│   ↑ CLICK THIS TAB                          │
└─────────────────────────────────────────────┘
```

### Screen 2: Connected Platforms Tab
```
┌─────────────────────────────────────────────┐
│  Connected Platforms                        │
├─────────────────────────────────────────────┤
│  [Your existing GitHub account if any]     │
│                                             │
├─────────────────────────────────────────────┤
│  Connect Platform                           │
├─────────────────────────────────────────────┤
│  [Connect GitHub with OAuth]                │
│                                             │
│  [Connect with Personal Access Token]      │
│   ↑↑↑ CLICK THIS BUTTON ↑↑↑                │
└─────────────────────────────────────────────┘
```

### Screen 3: Token Form Appears
```
┌─────────────────────────────────────────────┐
│  Connect Platform                           │
├─────────────────────────────────────────────┤
│  Platform:                                  │
│  ┌─────────────────────┐                   │
│  │ GitHub         ▼   │ ← CLICK DROPDOWN  │
│  └─────────────────────┘                   │
│  Options:                                   │
│    • GitHub                                 │
│    • Gitea  ← SELECT THIS                  │
└─────────────────────────────────────────────┘
```

### Screen 4: Gitea Selected - Instance URL Appears
```
┌─────────────────────────────────────────────┐
│  Connect Platform                           │
├─────────────────────────────────────────────┤
│  Platform:                                  │
│  ┌─────────────────────┐                   │
│  │ Gitea          ▼   │                   │
│  └─────────────────────┘                   │
│                                             │
│  Instance URL:          ← NEW FIELD APPEARS│
│  ┌───────────────────────────────────────┐ │
│  │ https://gitea.example.com             │ │
│  └───────────────────────────────────────┘ │
│  Enter the full URL of your Gitea instance │
│                                             │
│  Create a Gitea Personal Access Token with │
│  read:repository, read:user, and            │
│  read:organization scopes.                  │
│  Go to your Gitea instance → Settings →    │
│  Applications → Generate New Token          │
│                                             │
│  Your Gitea token                           │
│  ┌───────────────────────────────────────┐ │
│  │ ••••••••••••••••••••••••••••••••••••  │ │
│  └───────────────────────────────────────┘ │
│                                             │
│  [Connect]  [Cancel]                        │
└─────────────────────────────────────────────┘
```

---

## 🔍 Still Can't Find It?

### Issue 1: Not Seeing the Button?

**Check:**
1. Are you logged in?
2. Are you on the Settings page?
3. Are you on the "Connected Platforms" tab (not Privacy or Themes)?
4. Scroll down - it's below any existing connected platforms

### Issue 2: Button Exists But Form Doesn't Show?

**Try:**
1. Click the "Connect with Personal Access Token" button
2. Wait 1 second (form should appear)
3. Check browser console for errors (F12 → Console tab)

### Issue 3: Dropdown Only Shows GitHub?

**Check:**
```javascript
// The dropdown should have this HTML:
<select id="platform-select">
  <option value="github">GitHub</option>
  <option value="gitea">Gitea</option>  ← Should exist
</select>
```

If Gitea option is missing, the frontend code might not have updated.

**Solution:**
```bash
# Restart the frontend dev server
cd frontend
# Press Ctrl+C to stop current server
npm run dev
# Wait for "Local: http://localhost:5173"
# Refresh browser (Ctrl+R or Cmd+R)
```

### Issue 4: Instance URL Field Not Appearing?

**This field only appears when:**
- Platform dropdown is set to "Gitea"
- It uses conditional rendering: `{selectedPlatform === 'gitea' && (...)`

**Try:**
1. Select "Gitea" from Platform dropdown
2. Wait 0.5 seconds
3. Instance URL field should appear below dropdown

---

## 🧪 Testing the UI

### Quick Test Checklist

- [ ] Navigate to Settings → Connected Platforms
- [ ] Click "Connect with Personal Access Token"
- [ ] Form appears with Platform dropdown
- [ ] Change dropdown from "GitHub" to "Gitea"
- [ ] "Instance URL" field appears
- [ ] Instructions change to mention Gitea scopes
- [ ] Token placeholder changes to "Your Gitea token"
- [ ] Can enter instance URL (e.g., https://try.gitea.io)
- [ ] Can enter token in password field
- [ ] "Connect" button is clickable

### Example: Complete Flow

1. **Start:** http://localhost:5173/settings
2. **Click Tab:** "Connected Platforms"
3. **Click Button:** "Connect with Personal Access Token"
4. **Form Appears**
5. **Change Dropdown:** GitHub → Gitea
6. **Enter Instance:** `https://try.gitea.io`
7. **Enter Token:** `your-gitea-token-here`
8. **Click:** "Connect"
9. **Success:** Gitea account appears in platform list

---

## 🐛 Debugging Steps

If the UI still doesn't work, check these:

### 1. Browser Console (F12)
```javascript
// Check for JavaScript errors
// Should NOT see errors like:
❌ Uncaught TypeError: Cannot read property...
❌ Failed to compile...

// Should see logs like:
✅ [Vite] connected
✅ [HMR] connected
```

### 2. React DevTools (Browser Extension)
```
Components → PlatformConnector
  Props:
    (none - it's a standalone component)
  State:
    showPATForm: true      ← Should be true after clicking button
    selectedPlatform: "gitea"  ← Should be "gitea" after selection
    instanceUrl: "..."     ← Your instance URL
    patToken: "..."        ← Your token (hidden)
```

### 3. Network Tab (F12 → Network)
```
When clicking "Connect", should see:

POST /api/platforms/connect
Request Payload:
{
  "platform": "gitea",
  "access_token": "your-token",
  "instance_url": "https://gitea.example.com"
}
```

### 4. File Verification
```bash
# Check file was saved correctly
cat frontend/src/components/PlatformConnector.jsx | grep -A 5 "selectedPlatform === 'gitea'"

# Should output something like:
# {selectedPlatform === 'gitea' && (
#   <div className="instance-url-input">
#     <label htmlFor="instance-url">Instance URL:</label>
```

---

## 💡 Common Mistakes

### ❌ Wrong Page
```
Looking at: /platforms
Correct:    /settings (then Connected Platforms tab)
```

### ❌ GitHub Selected
```
Platform dropdown: GitHub (default)
Need to change to: Gitea
```

### ❌ Not Clicking Button First
```
Form is hidden by default
Must click: "Connect with Personal Access Token"
Then form appears
```

### ❌ Browser Cache
```
Old version cached in browser
Solution: Hard refresh (Ctrl+Shift+R or Cmd+Shift+R)
Or: Clear browser cache
```

---

## 📋 Component Code Location

If you want to verify the code yourself:

**File:** `/Users/developer/Github/hgitmap/frontend/src/components/PlatformConnector.jsx`

**Key Lines:**
- Line 10: `const [selectedPlatform, setSelectedPlatform] = useState('github')`
- Line 12: `const [instanceUrl, setInstanceUrl] = useState('')`
- Line 297-309: Platform selector dropdown
- Line 311-325: Instance URL input (conditional)
- Line 351-358: Token input field

**To view:**
```bash
cd /Users/developer/Github/hgitmap/frontend
cat src/components/PlatformConnector.jsx | grep -A 20 "Connect Platform"
```

---

## ✅ Expected Behavior

### When Everything Works:

1. **Platform Dropdown** changes from "GitHub" to "Gitea"
2. **Instance URL field** appears immediately
3. **Instructions** update to mention Gitea scopes
4. **Token placeholder** changes to "Your Gitea token"
5. **Validation** checks that instance URL is provided (for Gitea)
6. **Connect button** sends platform, token, and instance_url to backend
7. **Success** shows Gitea account in connected platforms list

---

## 🆘 Still Having Issues?

If none of the above helps:

1. **Take a screenshot** of your Settings → Connected Platforms page
2. **Check browser console** (F12) for errors
3. **Verify file timestamp:**
   ```bash
   ls -la frontend/src/components/PlatformConnector.jsx
   # Should show: Dec 24 17:11 (or later)
   ```
4. **Try hard refresh:** Ctrl+Shift+R (Windows) or Cmd+Shift+R (Mac)
5. **Restart frontend:**
   ```bash
   cd frontend
   # Kill server (Ctrl+C)
   npm run dev
   ```

---

**Last Updated:** 2025-01-24
**Component File:** `/frontend/src/components/PlatformConnector.jsx`
**Status:** ✅ Code is implemented and should be visible
