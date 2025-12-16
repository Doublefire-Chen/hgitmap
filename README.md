# hgitmap
This repo help you integrate your contributions from different git host platforms into one heatmap.

# Technology Stack
- Frontend: Vite + React(SPA)
- Backend: Rust
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
- Profile page similar to GitHub profile page, but using contributions from multiple git platforms