# Hgitmap

A unified contribution heatmap aggregator that integrates your contributions from multiple git hosting platforms (GitHub, Gitea, GitLab) into one beautiful visualization.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)
![React](https://img.shields.io/badge/react-18+-blue.svg)
![PostgreSQL](https://img.shields.io/badge/postgresql-14+-blue.svg)

## Demo

Check out the live demo: [https://hgitmap.aka.cy/Doublefire.Chen](https://hgitmap.aka.cy/Doublefire.Chen)

## Features

- **Multi-Platform Support** - GitHub, GitLab, Gitea
- **Public Profiles** - Share your contribution heatmap with others via `/:username` URL
- **Privacy Controls** - Choose to display/hide private repository contributions and names
- **Customizable Themes** - Create custom heatmap themes with different color schemes, sizes, and layouts
- **Activity Overview** - Track total contributions, current streak, and longest streak
- **Embeddable Heatmaps** - Pre-rendered heatmap API

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
- Build essentials: `sudo apt install pkg-config libssl-dev build-essential`

### Step 1: Clone Repository
```bash
cd ~
git clone https://github.com/Doublefire-Chen/hgitmap
cd hgitmap
```

### Step 2: Database Setup

```bash
psql
```
```
CREATE USER hgitmap WITH PASSWORD 'strong-password';
CREATE DATABASE hgitmap OWNER hgitmap;
GRANT ALL PRIVILEGES ON DATABASE hgitmap TO hgitmap;
```


Use [./backend/db_schema/schema.sql](backend/db_schema/schema.sql) to create the database schema.

### Step 3: Build Application

```bash

# Build backend
cd backend
cp .env.example .env # edit as needed

# Gommand to generate encryption key
openssl rand -base64 32

# Configure .env
cp .env.example /opt/hgitmap/.env
vim /opt/hgitmap/.env

# Build release binary
sudo apt install pkg-config libssl-dev build-essential # install build essentials
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # only run it if Rust is not installed
cargo build --release # built binary located at ./target/release/backend
mkdir -p /opt/hgitmap
sudo chown -R www-data:www-data /opt/hgitmap
sudo chmod -R 755 /opt/hgitmap
cp ./target/release/backend /opt/hgitmap/backend

# Build frontend
cd ../frontend
pnpm install # or npm install if you use npm

# Create production .env
cp .env.example .env
vim .env

pnpm run build # the built files are located at ./dist
mkdir -p /var/www/hgitmap
cp -r dist/* /var/www/hgitmap/ # copy to your web server directory
```

### Step 4: Create Systemd Service

Copy the service configuration:

```bash
# Copy service file and edit as needed
cd ~/hgitmap
sudo cp configs/hgitmap.service /etc/systemd/system/

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable hgitmap
sudo systemctl start hgitmap
sudo systemctl status hgitmap
```

### Step 5: Configure Nginx

Copy and configure the Nginx configuration:

```bash
# Copy configuration and edit as needed
sudo cp configs/nginx.conf /etc/nginx/sites-available/hgitmap
sudo vim /etc/nginx/sites-available/hgitmap

# Enable site
sudo ln -s /etc/nginx/sites-available/hgitmap /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Obtain SSL certificate
sudo certbot --nginx -d hgitmap-example.com -d api.hgitmap-example.com

# Reload Nginx
sudo systemctl reload nginx
```

### Step 6: Access Application
The first user to register will become the admin. Access the application at `https://hgitmap-example.com`.

## Platform Authentication Setup

For detailed GitHub authentication setup, see
- [GitHub Setup](GITHUB_SETUP.md)
- [GitLab Setup](GITLAB_SETUP.md)
- [Gitea Setup](GITEA_SETUP.md)