use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use blog_client::{BlogClient, Transport};

const DEFAULT_HTTP_SERVER: &str = "http://localhost:3000";
const DEFAULT_GRPC_SERVER: &str = "http://localhost:50051";
const TOKEN_FILE: &str = ".blog_token";

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "CLI client for the blog API", long_about = None)]
struct Cli {
    /// Use gRPC transport instead of HTTP
    #[arg(long, global = true)]
    grpc: bool,

    /// Server address (default: localhost:3000 for HTTP, localhost:50051 for gRPC)
    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new user
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },

    /// Login with existing credentials
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },

    /// Create a new post
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },

    /// Get a post by ID
    Get {
        #[arg(long)]
        id: i64,
    },

    /// Update a post
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },

    /// Delete a post
    Delete {
        #[arg(long)]
        id: i64,
    },

    /// List posts with pagination
    List {
        #[arg(long, default_value = "10")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
}

fn token_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(TOKEN_FILE)
}

fn load_token() -> Option<String> {
    std::fs::read_to_string(token_path()).ok()
}

fn save_token(token: &str) -> Result<()> {
    std::fs::write(token_path(), token).context("Failed to save token")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let server = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            DEFAULT_GRPC_SERVER.to_string()
        } else {
            DEFAULT_HTTP_SERVER.to_string()
        }
    });

    let transport = if cli.grpc {
        Transport::Grpc(server)
    } else {
        Transport::Http(server)
    };

    let mut client = BlogClient::new(transport)
        .await
        .context("Failed to create client")?;

    if let Some(token) = load_token() {
        client.set_token(token);
    }

    run_command(&mut client, cli.command).await
}

async fn run_command(client: &mut BlogClient, command: Commands) -> Result<()> {
    match command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let response = client
                .register(&username, &email, &password)
                .await
                .context("Registration failed")?;

            save_token(&response.token)?;

            println!("Registration successful!");
            println!("User ID: {}", response.user.id);
            println!("Username: {}", response.user.username);
            println!("Email: {}", response.user.email);
            println!("Token saved to {}", token_path().display());
        }

        Commands::Login { email, password } => {
            let response = client
                .login(&email, &password)
                .await
                .context("Login failed")?;

            save_token(&response.token)?;

            println!("Login successful!");
            println!("User ID: {}", response.user.id);
            println!("Username: {}", response.user.username);
            println!("Token saved to {}", token_path().display());
        }

        Commands::Create { title, content } => {
            let post = client
                .create_post(&title, &content)
                .await
                .context("Failed to create post")?;

            println!("Post created successfully!");
            print_post(&post);
        }

        Commands::Get { id } => {
            let post =
                client.get_post(id).await.context("Failed to get post")?;

            print_post(&post);
        }

        Commands::Update { id, title, content } => {
            let post = client
                .update_post(id, &title, &content)
                .await
                .context("Failed to update post")?;

            println!("Post updated successfully!");
            print_post(&post);
        }

        Commands::Delete { id } => {
            client
                .delete_post(id)
                .await
                .context("Failed to delete post")?;

            println!("Post {id} deleted successfully!");
        }

        Commands::List { limit, offset } => {
            let list = client
                .list_posts(limit, offset)
                .await
                .context("Failed to list posts")?;

            let end =
                (offset + i64::try_from(list.posts.len())?).min(list.total);
            println!("Posts ({}-{end} of {}):", offset + 1, list.total);
            println!("{}", "-".repeat(60));

            for post in &list.posts {
                println!(
                    "[{}] {} (by {})",
                    post.id,
                    post.title,
                    post.author_username.as_deref().unwrap_or("unknown")
                );
            }

            if list.posts.is_empty() {
                println!("No posts found.");
            }
        }
    }

    Ok(())
}

fn print_post(post: &blog_client::Post) {
    println!("ID: {}", post.id);
    println!("Title: {}", post.title);
    println!("Content: {}", post.content);
    println!(
        "Author: {} (ID: {})",
        post.author_username.as_deref().unwrap_or("unknown"),
        post.author_id
    );
    println!("Created: {}", post.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", post.updated_at.format("%Y-%m-%d %H:%M:%S"));
}
