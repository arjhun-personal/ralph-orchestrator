//! # ralph web
//!
//! Web dashboard development server launcher.
//!
//! This module provides the `ralph web` command that runs both the backend
//! and frontend dev servers in parallel.

use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use tokio::process::Command as AsyncCommand;

/// Arguments for the web subcommand
#[derive(Parser, Debug)]
pub struct WebArgs {
    /// Backend port (default: 3000)
    #[arg(long, default_value = "3000")]
    pub backend_port: u16,

    /// Frontend port (default: 5173)
    #[arg(long, default_value = "5173")]
    pub frontend_port: u16,

    /// Workspace root directory (default: current directory)
    #[arg(long)]
    pub workspace: Option<PathBuf>,
}

/// Run both backend and frontend dev servers in parallel
pub async fn execute(args: WebArgs) -> Result<()> {
    println!("🌐 Starting Ralph web servers...");
    println!(
        "   Backend: http://localhost:{}\n   Frontend: http://localhost:{}",
        args.backend_port, args.frontend_port
    );
    println!();

    // Determine workspace root: explicit flag or current directory
    let workspace_root = match args.workspace {
        Some(path) => {
            // Canonicalize to get absolute path
            path.canonicalize()
                .with_context(|| format!("Invalid workspace path: {}", path.display()))?
        }
        None => env::current_dir().context("Failed to get current directory")?,
    };

    println!("Using workspace: {}", workspace_root.display());

    // Compute absolute paths for backend and frontend directories
    // This ensures they work correctly regardless of where `ralph web` is invoked from
    let backend_dir = workspace_root.join("backend/ralph-web-server");
    let frontend_dir = workspace_root.join("frontend/ralph-web");

    // Spawn backend server
    // Pass RALPH_WORKSPACE_ROOT so the backend knows where to spawn ralph run from
    let mut backend = AsyncCommand::new("npm")
        .args(["run", "dev"])
        .current_dir(&backend_dir)
        .env("RALPH_WORKSPACE_ROOT", &workspace_root)
        .spawn()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to start backend server. Is npm installed and {} set up?\nError: {}",
                backend_dir.join("package.json").display(),
                e
            )
        })?;

    // Spawn frontend server
    let mut frontend = AsyncCommand::new("npm")
        .args(["run", "dev"])
        .current_dir(&frontend_dir)
        .spawn()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to start frontend server. Is npm installed and {} set up?\nError: {}",
                frontend_dir.join("package.json").display(),
                e
            )
        })?;

    println!("Press Ctrl+C to stop both servers");

    // Wait for both (Ctrl+C will terminate both)
    tokio::select! {
        r = backend.wait() => {
            println!("Backend exited: {:?}", r);
            // Kill frontend on backend exit
            let _ = frontend.start_kill();
            let _ = frontend.wait().await;
        }
        r = frontend.wait() => {
            println!("Frontend exited: {:?}", r);
            // Kill backend on frontend exit
            let _ = backend.start_kill();
            let _ = backend.wait().await;
        }
    }

    Ok(())
}
