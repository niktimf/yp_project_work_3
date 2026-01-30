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
blog-cli login --email "ivan@example.com" --password "secret123"
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
| `RATE_LIMIT_PER_SECOND` | No | 10 | Rate limit requests/second |
| `RATE_LIMIT_BURST` | No | 20 | Rate limit burst size |
| `CORS_MAX_AGE` | No | 3600 | CORS preflight cache (seconds) |

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