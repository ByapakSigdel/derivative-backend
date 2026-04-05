# Derivative Backend API

Production-ready Rust backend for the Derivative visual programming platform.

## Features

- **Authentication**: JWT-based auth with access and refresh tokens
- **User Management**: Full CRUD with admin controls
- **Projects**: Create, update, delete, and clone visual programming projects
- **Community**: Likes, comments, and view tracking
- **Real-time Collaboration**: WebSocket support for live project editing
- **File Uploads**: Avatar and project asset management
- **Full-Text Search**: PostgreSQL-powered search across projects
- **Admin Panel**: User management and moderation tools

## Tech Stack

- **Language**: Rust (2021 edition)
- **Web Framework**: Axum 0.7+
- **Runtime**: Tokio
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: Argon2
- **Validation**: validator
- **WebSockets**: Native Axum WebSocket support

## Quick Start

### Prerequisites

- Rust 1.75+ (latest stable)
- PostgreSQL 14+
- SQLx CLI (for migrations)

### Installation

1. **Clone the repository**
   ```bash
   cd backend
   ```

2. **Install SQLx CLI**
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

3. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

4. **Create database**
   ```bash
   createdb derivative
   ```

5. **Run migrations**
   ```bash
   sqlx database create
   sqlx migrate run
   ```

6. **Build and run**
   ```bash
   cargo run --release
   ```

The server will start at `http://localhost:8080`

## API Documentation

### Authentication

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword"
}
```

Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "full_name": "John Doe",
    "user_type": "user"
  }
}
```

#### Refresh Token
```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJ..."
}
```

#### Get Current User
```http
GET /api/auth/me
Authorization: Bearer <access_token>
```

#### Logout
```http
POST /api/auth/logout
Authorization: Bearer <access_token>
```

### Projects

#### List User's Projects
```http
GET /api/user-projects?page=1&per_page=20&category=tutorial&difficulty=beginner&search=game
Authorization: Bearer <access_token>
```

#### Create Project
```http
POST /api/user-projects
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "title": "My First Project",
  "description": "A tutorial project",
  "difficulty": "beginner",
  "category": "tutorial",
  "nodes": [],
  "edges": [],
  "tags": ["tutorial", "beginner"],
  "is_public": false
}
```

#### Get Project
```http
GET /api/user-projects/:id
Authorization: Bearer <access_token>
```

#### Update Project
```http
PATCH /api/user-projects/:id
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "title": "Updated Title",
  "is_public": true
}
```

#### Delete Project
```http
DELETE /api/user-projects/:id
Authorization: Bearer <access_token>
```

#### Clone Project
```http
POST /api/user-projects/:id/clone
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "title": "My Clone"
}
```

#### Get User's Project Stats
```http
GET /api/user-projects/stats
Authorization: Bearer <access_token>
```

#### List Public Projects (Community)
```http
GET /api/user-projects/public?page=1&per_page=20&featured=true&sort_by=like_count
```

### Community Features

#### Toggle Like
```http
POST /api/user-projects/:id/like
Authorization: Bearer <access_token>
```

#### Get Like Status
```http
GET /api/user-projects/:id/like
Authorization: Bearer <access_token>
```

#### Record View
```http
POST /api/user-projects/:id/view
Content-Type: application/json

{
  "view_duration": 120,
  "referrer": "https://example.com"
}
```

#### Get Comments
```http
GET /api/user-projects/:id/comments?page=1&per_page=20
```

#### Add Comment
```http
POST /api/user-projects/:id/comments
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "content": "Great project!",
  "parent_id": null
}
```

#### Update Comment
```http
PATCH /api/user-projects/:id/comments/:comment_id
Authorization: Bearer <access_token>
Content-Type: application/json

{
  "content": "Updated comment"
}
```

#### Delete Comment
```http
DELETE /api/user-projects/:id/comments/:comment_id
Authorization: Bearer <access_token>
```

### Admin (Admin Only)

#### List Users
```http
GET /api/admin/users?page=1&per_page=20&search=john&user_type=user
Authorization: Bearer <admin_access_token>
```

#### Create User
```http
POST /api/admin/users
Authorization: Bearer <admin_access_token>
Content-Type: application/json

{
  "email": "newuser@example.com",
  "full_name": "New User",
  "password": "securepassword",
  "user_type": "user"
}
```

#### Update User
```http
PATCH /api/admin/users/:id
Authorization: Bearer <admin_access_token>
Content-Type: application/json

{
  "is_active": false
}
```

#### Delete User
```http
DELETE /api/admin/users/:id
Authorization: Bearer <admin_access_token>
```

### File Uploads

#### Upload Avatar
```http
POST /api/users/avatar
Authorization: Bearer <access_token>
Content-Type: multipart/form-data

avatar: <file>
```

Response:
```json
{
  "url": "/api/uploads/avatars/uuid_timestamp.jpg",
  "filename": "uuid_timestamp.jpg"
}
```

### WebSocket

Connect to project collaboration room:
```
ws://localhost:8080/ws/projects/:project_id?token=<access_token>
```

#### Client Messages
```json
// Ping
{"type": "ping"}

// Project updated
{
  "type": "project_updated",
  "project_id": "uuid",
  "nodes": [...],
  "edges": [...]
}

// Cursor move
{
  "type": "cursor_move",
  "project_id": "uuid",
  "x": 100.5,
  "y": 200.3
}
```

#### Server Messages
```json
// User joined
{
  "type": "user_joined",
  "project_id": "uuid",
  "user_id": "uuid",
  "user_name": "John",
  "timestamp": "2024-01-01T00:00:00Z"
}

// Project updated
{
  "type": "project_updated",
  "project_id": "uuid",
  "user_id": "uuid",
  "timestamp": "2024-01-01T00:00:00Z",
  "payload": {
    "nodes": [...],
    "edges": [...]
  }
}
```

### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "database": {
    "connected": true,
    "pool_size": 10,
    "pool_idle": 8,
    "pool_active": 2
  }
}
```

## Error Responses

All errors follow this format:
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Email is required",
    "details": ["field: email"]
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| DATABASE_ERROR | 500 | Database operation failed |
| VALIDATION_ERROR | 400 | Request validation failed |
| UNAUTHORIZED | 401 | Authentication required |
| INVALID_CREDENTIALS | 401 | Wrong email or password |
| TOKEN_EXPIRED | 401 | JWT token expired |
| INVALID_TOKEN | 401 | Malformed or invalid token |
| FORBIDDEN | 403 | Access denied |
| NOT_FOUND | 404 | Resource not found |
| CONFLICT | 409 | Resource already exists |
| FILE_TOO_LARGE | 413 | Upload exceeds size limit |
| RATE_LIMIT_EXCEEDED | 429 | Too many requests |
| INTERNAL_SERVER_ERROR | 500 | Unexpected error |

## Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

## Development

### Run with auto-reload
```bash
cargo install cargo-watch
cargo watch -x run
```

### Run tests
```bash
cargo test
```

### Check for issues
```bash
cargo clippy
```

### Format code
```bash
cargo fmt
```

## Production Deployment

1. Build release binary:
   ```bash
   cargo build --release
   ```

2. The binary is at `target/release/derivative-backend`

3. Configure production environment variables

4. Run with a process manager (systemd, supervisor, etc.)

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/derivative-backend /usr/local/bin/
CMD ["derivative-backend"]
```

## Performance

- Target: < 50ms API response time (excluding DB)
- WebSocket: 1000+ concurrent connections
- Connection pooling for database efficiency
- Async I/O throughout

## Security

- Argon2id password hashing
- JWT with short-lived access tokens
- Refresh token rotation
- CORS configuration
- Input validation on all endpoints
- Parameterized SQL queries (SQLx)
- File upload validation

## License

MIT
# derivative-backend
# derivative-backend
