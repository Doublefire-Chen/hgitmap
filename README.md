# hgitmap
This repo help you integrate your contributions from different git host platforms into one heatmap.

# Technology Stack
- Frontend: Vite + React (SPA)
- Backend: Rust (Actix-web + SeaORM)
- Database: PostgreSQL

# Supported Git Platforms
- GitHub
- Gitea
- GitLab

# Features
- Username/Password login
- Generate contribution heatmap from multiple git platforms
- Privacy controlled by user, you can decide if display contribution to private repositories, or you can display while hiding name of private repositories
- Api to privide pre-rendered heatmap image for embedding in other places, such as README.md in GitHub
- Customizable appearance of heatmap, like color scheme, size, etc.
- Dark mode support
- Profile page similar to GitHub profile page, but using contributions from multiple git platforms, which include "Activity overview", "Contribution activity" as well.

## Deployment

### Prerequisites
- Rust 1.75+
- PostgreSQL 14+
- Node.js 18+ (for frontend)

### Setup

1. **Database**
```bash
createdb hgitmap
psql -d hgitmap -f backend/db_schema/schema.sql
```

2. **Backend**
```bash
cd backend
cp .env.example .env
# Edit .env with your configuration
cargo build --release
./target/release/backend
```

3. **Frontend**
```bash
cd frontend
npm install
npm run build
# Serve the dist/ directory
```

For API documentation, see [API.md](API.md).