# Hgitmap

A unified contribution heatmap aggregator that integrates your contributions from multiple git hosting platforms (GitHub, Gitea, GitLab) into one beautiful visualization.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![PostgreSQL](https://img.shields.io/badge/postgresql-14+-blue.svg)

## Features

- **Multi-Platform Support** - GitHub, GitLab, Gitea
- **Public Profiles** - Share your contribution heatmap with others via `/u/username`
- **Privacy Controls** - Choose to display/hide private repository contributions and names
- **Customizable Themes** - Create custom heatmap themes with different color schemes, sizes, and layouts
- **Activity Overview** - Track total contributions, current streak, and longest streak
- **Embeddable Heatmaps** - Pre-rendered heatmap API

## Public Profiles & Privacy Controls

### Single-User Application Design

**Important:** Hgitmap is designed as a **single-user application**. This means:
- Only one user account exists in the system
- The home page (`/`) always shows that user's contribution heatmap
- Visitors (not logged in) see the public profile view based on privacy settings
- The logged-in user sees the full authenticated dashboard with management controls

### How Public Profiles Work

Hgitmap provides two types of views for the single user:

1. **Public Profile View** (`/`) - When not logged in, visitors see the user's public heatmap, stats, and activity timeline (filtered by privacy settings)
2. **Authenticated Dashboard** (`/`) - When logged in, the user sees their full dashboard with settings, platform management, and complete data
3. **Public Profile Page** (`/u/:username`) - Direct link to the user's public profile (same as home page when not logged in)

### Privacy Settings

Users have granular control over what data is visible on their public profile:

| Setting | Description | Default |
|---------|-------------|---------|
| `show_private_contributions` | Include contributions to private repositories in the heatmap | `true` |
| `show_private_repo_names` | Display the actual names of private repositories in activity timeline | `false` |

**Privacy Behavior:**
- If `show_private_contributions = false`: Contributions to private repositories are excluded from the heatmap count and activity timeline
- If `show_private_contributions = true` but `show_private_repo_names = false`: Private contributions are counted in the heatmap, but repository names show as "Private Repository" in the activity timeline

### Backend API Requirements

The backend must implement the following **public** (unauthenticated) API endpoints:

#### Single User Info
```
GET /public/info
```

**Purpose:** Returns the single user's basic information for the single-user app design.

**Response:**
```json
{
  "username": "johndoe",
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### Public Contribution Data
```
GET /public/:username/contributions?from=YYYY-MM-DD&to=YYYY-MM-DD&platform=github
```

**Response:** Filtered contribution data based on user's privacy settings
```json
{
  "contributions": [
    {
      "date": "2024-01-15",
      "count": 5,
      "platform": "github",
      "repository_name": "my-repo"  // or "Private Repository" if show_private_repo_names=false
    }
  ]
}
```

#### Public Contribution Statistics
```
GET /public/:username/contributions/stats
```

**Response:** Aggregated statistics respecting privacy settings
```json
{
  "total_contributions": 1234,
  "current_streak": 15,
  "longest_streak": 42,
  "total_days": 365
}
```

#### Public Platform List
```
GET /public/:username/platforms
```

**Response:** List of connected platforms (public information only)
```json
{
  "platforms": [
    {
      "platform": "github",
      "platform_username": "johndoe",
      "last_synced_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### Public Activity Timeline
```
GET /public/:username/activities?from=YYYY-MM-DD&to=YYYY-MM-DD&limit=50&offset=0&platform=github
```

**Response:** Recent activities filtered by privacy settings
```json
{
  "activities": [
    {
      "id": "uuid",
      "date": "2024-01-15T14:30:00Z",
      "platform": "github",
      "activity_type": "commit",
      "repository_name": "my-repo",  // or "Private Repository"
      "description": "Updated README.md",
      "count": 3
    }
  ],
  "has_more": true
}
```

### Frontend Routes

| Route | Authentication | Description |
|-------|----------------|-------------|
| `/` | Public | Public profile view for visitors, authenticated dashboard for logged-in user |
| `/login` | Public | Login page for the single user |
| `/register` | Public | Registration page (typically disabled after first user creation) |
| `/u/:username` | Public | Direct link to public profile (same as home page for visitors) |
| `/settings/*` | Protected | User settings, platform management, theme configuration |

### Share Your Profile

The logged-in user can click the share button in the header to copy their public profile URL:
```
https://yourdomain.com/
```
or
```
https://yourdomain.com/u/username
```

Both URLs show the same public profile view when accessed by visitors (not logged in).

## Technology Stack

- **Frontend**: Vite + React(SPA)
- **Backend**: Rust
- **Database**: PostgreSQL
- **Design**: Single-user application (one account per installation)

## Supported Git Platforms

| Platform | OAuth | Personal Access Token | Self-Hosted |
|----------|-------|----------------------|-------------|
| GitHub | Yes | Yes | -- |
| GitLab | Yes | Yes | Yes |
| Gitea | Yes | Yes | Yes |

---

## How to install (Ubuntu as example)

### Prerequisites

- Rust
- Node.js
- PostgreSQL
- Nginx

### Step 1: Clone Repository
```bash
git clone https://github.com/Doublefire-Chen/hgitmap
cd hgitmap
```

### Step 2: Database Setup

Use ./backend/db_schema/schema.sql to create the database schema.

### Step 3: Build Application

```bash

# Build backend
cd backend
cp .env.example .env

# Generate encryption key
ENCRYPTION_KEY=$(openssl rand -base64 32)

# Configure .env
cp .env.example .env
vim .env

# Build release binary
cargo build --release

# Build frontend
cd ../frontend
npm install

# Create production .env
cp .env.example .env
vim .env

npm run build
```

### Step 4: Create Systemd Service

Create `/etc/systemd/system/hgitmap.service`:

```ini
[Unit]
Description=hgitmap Backend Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/hgitmap/backend
Environment="RUST_LOG=info"
ExecStart=/opt/hgitmap/backend/target/release/backend
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/hgitmap/backend

[Install]
WantedBy=multi-user.target
```

Set up permissions:

```bash
# Create application directory structure
sudo mkdir -p /opt/hgitmap
sudo chown -R www-data:www-data /opt/hgitmap

# Move your built application
sudo mv /path/to/hgitmap /opt/

# Set proper permissions
sudo chown -R www-data:www-data /opt/hgitmap
sudo chmod -R 755 /opt/hgitmap
```

Enable and start service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable hgitmap
sudo systemctl start hgitmap
sudo systemctl status hgitmap
```

### Step 5: Configure Nginx

Create `/etc/nginx/sites-available/hgitmap`:

```nginx
# Backend API server
upstream hgitmap_backend {
    server 127.0.0.1:3000;
    keepalive 32;
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name yourdomain.com;

    # Allow certbot for SSL certificate generation
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }

    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name yourdomain.com;

    # SSL configuration (will be added by certbot)
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Frontend static files
    root /opt/hgitmap/frontend/dist;
    index index.html;

    # Compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/json application/xml+rss image/svg+xml;

    # Backend API proxy
    location /auth/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 300s;
        proxy_connect_timeout 75s;
    }

    location /platforms/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    location /contributions/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /activities/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /settings/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /oauth/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /admin/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /heatmap/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /embed/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Cache embedded images
        proxy_cache_valid 200 10m;
        add_header X-Cache-Status $upstream_cache_status;
    }

    location /sync/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /public/ {
        proxy_pass http://hgitmap_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Cache public profile data for 5 minutes
        proxy_cache_valid 200 5m;
        add_header X-Cache-Status $upstream_cache_status;
    }

    # Frontend - serve static files with caching
    location /assets/ {
        root /opt/hgitmap/frontend/dist;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Frontend - SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
        expires -1;
        add_header Cache-Control "no-store, no-cache, must-revalidate, proxy-revalidate";
    }

    # Logs
    access_log /var/log/nginx/hgitmap_access.log;
    error_log /var/log/nginx/hgitmap_error.log;
}
```

Enable site and obtain SSL certificate:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/hgitmap /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Obtain SSL certificate (before uncommenting SSL lines)
sudo certbot --nginx -d yourdomain.com

# Reload Nginx
sudo systemctl reload nginx
```

### Step 6: Configure Firewall

```bash
# Allow SSH, HTTP, and HTTPS
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### Step 7: Create Your User Account

Since Hgitmap is a single-user application, you need to create your user account:

```bash
# Temporarily enable registration
sudo sed -i 's/ALLOW_REGISTRATION=false/ALLOW_REGISTRATION=true/' /opt/hgitmap/backend/.env
sudo systemctl restart hgitmap

# Register your account via the web UI (visit https://yourdomain.com/register)
# After registration, immediately disable registration to prevent additional accounts
sudo sed -i 's/ALLOW_REGISTRATION=true/ALLOW_REGISTRATION=false/' /opt/hgitmap/backend/.env
sudo systemctl restart hgitmap
```

**Important:** This is a single-user app. Only create ONE user account, then disable registration.

---

## Platform Authentication Setup

### GitHub

For detailed GitHub authentication setup, see [AUTHENTICATION.md](AUTHENTICATION.md#github-authentication).

**Personal Access Token Scopes:**
- `repo` - Full control of private repositories
- `read:user` - Read user profile data

Quick link: https://github.com/settings/tokens/new?scopes=repo,read:user&description=hgitmap

### GitLab

For detailed GitLab setup (including self-hosted instances), see [GITLAB_SETUP.md](GITLAB_SETUP.md).

**Personal Access Token Scopes:**
- `read_user` - Profile information
- `read_api` - API read access
- `read_repository` - Private repository access

**GitLab.com**: https://gitlab.com/-/user_settings/personal_access_tokens

**Self-hosted**: `https://your-gitlab.com/-/user_settings/personal_access_tokens`

### Gitea

For detailed Gitea authentication setup, see [AUTHENTICATION.md](AUTHENTICATION.md#gitea-authentication).

**Personal Access Token Scopes:**
- `read:repository` - Read repository data
- `read:user` - Read user profile data
- `read:organization` - Read organization data

**Location**: `https://your-gitea.com/user/settings/applications`

---

## OAuth Application Setup

To enable OAuth login for platforms:

1. **Create OAuth Application** on the platform (GitHub, GitLab, or Gitea)
2. **Configure in hgitmap** at Settings → Platforms → OAuth Apps tab
3. **Set Redirect URI** to: `https://yourdomain.com/oauth/{platform}/callback`

Replace `{platform}` with: `github`, `gitlab`, or `gitea`

For detailed OAuth setup instructions, see:
- [AUTHENTICATION.md](AUTHENTICATION.md) - GitHub and Gitea OAuth
- [GITLAB_SETUP.md](GITLAB_SETUP.md) - GitLab OAuth (official and self-hosted)

---

## Configuration Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `HOST` | No | `127.0.0.1` | Backend bind address |
| `PORT` | No | `3000` | Backend port |
| `JWT_SECRET` | Yes | - | Secret key for JWT tokens (change in production!) |
| `JWT_EXPIRATION_HOURS` | No | `24` | JWT token expiration time |
| `ALLOW_REGISTRATION` | No | `true` | Allow new user registration (disable after creating the single user) |
| `BASE_URL` | Yes | - | Public URL for OAuth callbacks |
| `ENCRYPTION_KEY` | Yes | - | AES-256 encryption key (base64, 32 bytes) |
| `RUST_LOG` | No | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

### Generate Secure Keys

```bash
# JWT Secret
openssl rand -base64 32

# Encryption Key (32 bytes)
openssl rand -base64 32
```

---

## Maintenance

### View Logs

```bash
# Backend logs
sudo journalctl -u hgitmap -f

# Nginx logs
sudo tail -f /var/log/nginx/hgitmap_access.log
sudo tail -f /var/log/nginx/hgitmap_error.log
```

### Restart Services

```bash
# Restart backend
sudo systemctl restart hgitmap

# Restart Nginx
sudo systemctl restart nginx
```

### Update Application

```bash
cd /opt/hgitmap

# Pull latest changes
sudo -u www-data git pull

# Rebuild backend
cd backend
sudo -u www-data cargo build --release

# Rebuild frontend
cd ../frontend
sudo -u www-data npm install
sudo -u www-data npm run build

# Restart service
sudo systemctl restart hgitmap
```

### Database Backup

```bash
# Backup
sudo -u postgres pg_dump hgitmap > hgitmap_backup_$(date +%Y%m%d).sql

# Restore
sudo -u postgres psql hgitmap < hgitmap_backup_20250101.sql
```

---

## Troubleshooting

### Backend won't start

```bash
# Check logs
sudo journalctl -u hgitmap -n 50

# Common issues:
# 1. Database connection - verify DATABASE_URL
# 2. Port conflict - ensure port 3000 is free
# 3. Missing .env - verify .env file exists
```

### Frontend shows connection error

```bash
# Verify backend is running
sudo systemctl status hgitmap

# Check Nginx proxy
sudo nginx -t
sudo tail -f /var/log/nginx/hgitmap_error.log
```

### OAuth callback fails

```bash
# Verify BASE_URL in backend .env matches your domain
# Check OAuth app redirect URI matches: https://yourdomain.com/oauth/{platform}/callback
# Ensure SSL certificate is valid
```

For more troubleshooting, see:
- [AUTHENTICATION.md - Troubleshooting](AUTHENTICATION.md#troubleshooting)
- [GITLAB_SETUP.md - Troubleshooting](GITLAB_SETUP.md#troubleshooting)

---

## API Documentation

See [API.md](API.md) for complete API documentation.

### Public Profile API

Access user profile data without authentication:

```bash
# Get single user info (for single-user app)
GET /public/info

# Get public contributions
GET /public/:username/contributions?from=2024-01-01&to=2024-12-31&platform=github

# Get public contribution stats
GET /public/:username/contributions/stats

# Get public platform list
GET /public/:username/platforms

# Get public activity timeline
GET /public/:username/activities?limit=50&offset=0&platform=github
```

**Note:** All public endpoints respect user privacy settings (`show_private_contributions` and `show_private_repo_names`).

### Embed API

Embed your heatmap in GitHub README or other platforms:

```markdown
![Contribution Heatmap](https://yourdomain.com/embed/username/theme-slug.png)
```

Supported formats: `png`, `svg`, `jpeg`, `webp`

---

## Security Considerations

1. **Change default secrets** - Always change `JWT_SECRET` in production
2. **Use HTTPS** - Always use SSL certificates in production
3. **Disable registration** - Set `ALLOW_REGISTRATION=false` immediately after creating your user account (single-user app)
4. **Regular updates** - Keep dependencies up to date
5. **Token encryption** - Access tokens are encrypted with AES-256
6. **Firewall** - Restrict access to PostgreSQL port (5432)
7. **Backup regularly** - Schedule regular database backups
8. **Single-user design** - This app is designed for one user only; do not create multiple accounts

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## License

GNU General Public License v3.0 - see [LICENSE](LICENSE) file for details

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

---

## Acknowledgments

- Inspired by GitHub's contribution graph
- Built with Rust and React
- Uses SeaORM for database operations
- Actix-web for backend framework

---

## Support

- [Documentation](AUTHENTICATION.md)
- [Issue Tracker](https://github.com/yourusername/hgitmap/issues)
- [Discussions](https://github.com/yourusername/hgitmap/discussions)

---

**Note**: Replace `yourdomain.com` with your actual domain throughout the configuration files.
