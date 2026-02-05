# Blog Platform

Rust workspace project with HTTP/gRPC server, client library, CLI, and WASM frontend.

## Structure

```
├── blog-server/   # HTTP (axum) + gRPC (tonic) server
├── blog-client/   # Client library for HTTP and gRPC
├── blog-cli/      # Command-line interface
└── blog-wasm/     # WebAssembly frontend
```

## Quick Start

### 1. Start the server

```bash
# Set environment variables
cp blog-server/.env.example blog-server/.env
# Edit .env with your database credentials

# Run server
cargo run -p blog-server
```

Server runs on:
- HTTP: `http://localhost:3000`
- gRPC: `http://localhost:50051`

### 2. Use CLI

```bash
# Build CLI
cargo build -p blog-cli --release

# Or run directly with cargo
cargo run -p blog-cli -- <command>
```

## CLI Usage

### Authentication

```bash
# Register a new user
blog-cli register --username "ivan" --email "ivan@example.com" --password "secret123"

# Login
blog-cli login --username "ivan" --password "secret123"
```

Token is automatically saved to `~/.blog_token`.

### Posts CRUD

```bash
# Create a post
blog-cli create --title "My First Post" --content "Hello, World!"

# Get a post by ID
blog-cli get --id 1

# Update a post
blog-cli update --id 1 --title "Updated Title" --content "New content"

# Delete a post
blog-cli delete --id 1

# List posts with pagination
blog-cli list --limit 20 --offset 0
```

### Using gRPC transport

Add `--grpc` flag to any command:

```bash
blog-cli --grpc register --username "ivan" --email "ivan@example.com" --password "secret123"
blog-cli --grpc create --title "Post via gRPC" --content "Content"
blog-cli --grpc list
```

### Custom server address

```bash
# HTTP
blog-cli --server "http://api.example.com:3000" list

# gRPC
blog-cli --grpc --server "http://grpc.example.com:50051" list
```

## API Endpoints

### HTTP API (v1)

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/api/v1/health` | No | Health check |
| POST | `/api/v1/auth/register` | No | Register user |
| POST | `/api/v1/auth/login` | No | Login |
| GET | `/api/v1/posts/` | No | List posts |
| POST | `/api/v1/posts/` | Yes | Create post |
| GET | `/api/v1/posts/{id}` | No | Get post |
| PUT | `/api/v1/posts/{id}` | Yes | Update post |
| DELETE | `/api/v1/posts/{id}` | Yes | Delete post |

### gRPC Methods

- `Register`, `Login`
- `CreatePost`, `GetPost`, `UpdatePost`, `DeletePost`, `ListPosts`

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `JWT_SECRET` | Yes | - | JWT signing secret (min 32 chars) |
| `CORS_ALLOWED_ORIGINS` | Yes | - | Comma-separated allowed origins |
| `DATABASE_MAX_CONNECTIONS` | No | 5 | Max DB pool connections |
| `JWT_TOKEN_EXPIRY_HOURS` | No | 24 | JWT token lifetime in hours |
| `HTTP_HOST` | No | 0.0.0.0 | HTTP server bind address |
| `HTTP_PORT` | No | 3000 | HTTP server port |
| `GRPC_HOST` | No | 0.0.0.0 | gRPC server bind address |
| `GRPC_PORT` | No | 50051 | gRPC server port |
| `RATE_LIMIT_PER_SECOND` | No | 10 | Rate limit requests/second |
| `RATE_LIMIT_BURST` | No | 20 | Rate limit burst size |
| `CORS_MAX_AGE` | No | 3600 | CORS preflight cache (seconds) |
| `PAGINATION_DEFAULT_LIMIT` | No | 10 | Default page size |
| `PAGINATION_MAX_LIMIT` | No | 100 | Maximum page size |

## Docker

### Quick Start

```bash
docker compose up --build
```

Open **http://localhost:8080** in your browser.

### Services

| Service | Port | Description |
|---------|------|-------------|
| `frontend` | 8080 | Nginx serving the WASM frontend |
| `blog-server` | 3000 | HTTP API |
| `blog-server` | 50051 | gRPC API |
| `postgres` | 5432 | PostgreSQL database |

### Configuration

Secrets are stored in `.env` (not committed to git):

```bash
POSTGRES_DB=blog_db
POSTGRES_USER=blog_user
POSTGRES_PASSWORD=blog_password
JWT_SECRET=your-secret-key-minimum-32-characters
```

Non-secret settings are defined directly in `docker-compose.yml`.

### Architecture

```
browser :8080 --> nginx (WASM frontend)
browser :3000 --> blog-server (HTTP API) --> postgres
                  blog-server (gRPC :50051)
```

The WASM frontend makes API calls directly to `localhost:3000` from the browser. CORS is configured to allow requests from `http://localhost:8080`.

### Build Details

- **blog-server** — multi-stage build: `rust:1.93-slim-bookworm` (build) + `debian:bookworm-slim` (runtime)
- **frontend** — multi-stage build: `rust:1.93-slim-bookworm` + `wasm-pack` (build) + `nginx:1.27-alpine` (serve)
- Non-root user in the server container
- Healthcheck on `/api/v1/health`
- Dependency caching via separate `COPY` of `Cargo.toml`/`Cargo.lock` before source code

## WASM Frontend

### Build

```bash
# Install wasm-pack if not installed
cargo install wasm-pack

# Build WASM module
cd blog-wasm
wasm-pack build --target web
```

### Run

```bash
# Start a local HTTP server
cd blog-wasm
python3 -m http.server 8080

# Or use any other static file server
npx serve .
```

Open `http://localhost:8080` in your browser.

### Features

- User registration and login
- JWT token stored in localStorage
- Create, edit, delete posts (authenticated users only)
- View all posts (public)
- Pagination

## Development

```bash
# Check all packages
cargo check --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace

# Format code
cargo fmt --all
```