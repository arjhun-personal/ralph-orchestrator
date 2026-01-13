//! # ralph-cli
//!
//! Binary entry point for the Ralph Orchestrator.
//!
//! This crate provides:
//! - CLI argument parsing using `clap`
//! - Application initialization and configuration
//! - Entry point to the headless orchestration loop

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use ralph_adapters::{detect_backend, CliBackend, CliExecutor};
use ralph_core::{EventLoop, RalphConfig, TerminationReason};
use std::io::{stdout, IsTerminal};
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info, warn};

/// Color output mode for terminal display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ColorMode {
    /// Automatically detect if stdout is a TTY
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

impl ColorMode {
    /// Returns true if colors should be used based on mode and terminal detection.
    fn should_use_colors(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => stdout().is_terminal(),
        }
    }
}

/// ANSI color codes for terminal output.
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
}

/// Ralph Orchestrator - Multi-agent orchestration framework
#[derive(Parser, Debug)]
#[command(name = "ralph", version, about)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "ralph.yml")]
    config: PathBuf,

    /// Override the prompt file
    #[arg(short, long)]
    prompt: Option<PathBuf>,

    /// Override max iterations
    #[arg(long)]
    max_iterations: Option<u32>,

    /// Override completion promise
    #[arg(long)]
    completion_promise: Option<String>,

    /// Dry run - show what would be executed without running
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Color output mode (auto, always, never)
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Ralph Orchestrator v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let mut config = if args.config.exists() {
        RalphConfig::from_file(&args.config)
            .with_context(|| format!("Failed to load config from {:?}", args.config))?
    } else {
        warn!("Config file {:?} not found, using defaults", args.config);
        RalphConfig::default()
    };

    // Normalize v1 flat fields into v2 nested structure
    config.normalize();

    // Apply CLI overrides (after normalization so they take final precedence)
    if let Some(prompt) = args.prompt {
        config.event_loop.prompt_file = prompt.to_string_lossy().to_string();
    }
    if let Some(max_iter) = args.max_iterations {
        config.event_loop.max_iterations = max_iter;
    }
    if let Some(promise) = args.completion_promise {
        config.event_loop.completion_promise = promise;
    }
    if args.verbose {
        config.verbose = true;
    }

    // Validate configuration and emit warnings
    let warnings = config.validate().context("Configuration validation failed")?;
    for warning in &warnings {
        eprintln!("{warning}");
    }

    // Handle auto-detection if backend is "auto"
    if config.cli.backend == "auto" {
        let priority = config.get_agent_priority();
        let detected = detect_backend(&priority, |backend| {
            config.adapter_settings(backend).enabled
        });

        match detected {
            Ok(backend) => {
                info!("Auto-detected backend: {}", backend);
                config.cli.backend = backend;
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(anyhow::Error::new(e));
            }
        }
    }

    if args.dry_run {
        println!("Dry run mode - configuration:");
        println!("  Mode: {}", config.mode);
        println!("  Prompt file: {}", config.event_loop.prompt_file);
        println!("  Completion promise: {}", config.event_loop.completion_promise);
        println!("  Max iterations: {}", config.event_loop.max_iterations);
        println!("  Max runtime: {}s", config.event_loop.max_runtime_seconds);
        println!("  Backend: {}", config.cli.backend);
        println!("  Git checkpoint: {}", config.git_checkpoint);
        println!("  Verbose: {}", config.verbose);
        if !warnings.is_empty() {
            println!("  Warnings: {}", warnings.len());
        }
        return Ok(());
    }

    // Run the orchestration loop
    run_loop(config, args.color).await
}

async fn run_loop(config: RalphConfig, color_mode: ColorMode) -> Result<()> {
    let use_colors = color_mode.should_use_colors();

    // Read prompt file
    let prompt_content = std::fs::read_to_string(&config.event_loop.prompt_file)
        .with_context(|| format!("Failed to read prompt file: {}", config.event_loop.prompt_file))?;

    // Initialize event loop
    let mut event_loop = EventLoop::new(config.clone());
    event_loop.initialize(&prompt_content);

    // Create CLI executor
    let backend = CliBackend::from_config(&config.cli);
    let executor = CliExecutor::new(backend);

    info!(
        "Starting {} mode with {} iterations max",
        if config.is_single_mode() { "single-hat" } else { "multi-hat" },
        config.event_loop.max_iterations
    );

    // Main orchestration loop
    loop {
        // Check termination before execution
        if let Some(reason) = event_loop.check_termination() {
            print_termination(&reason, event_loop.state(), use_colors);
            break;
        }

        // Get next hat to execute
        let hat_id = match event_loop.next_hat() {
            Some(id) => id.clone(),
            None => {
                warn!("No hats with pending events, terminating");
                break;
            }
        };

        let iteration = event_loop.state().iteration + 1;
        info!("Iteration {}: executing hat '{}'", iteration, hat_id);

        // Build prompt for this hat
        let prompt = if config.is_single_mode() {
            event_loop.build_single_prompt(&prompt_content)
        } else {
            match event_loop.build_prompt(&hat_id) {
                Some(p) => p,
                None => {
                    error!("Failed to build prompt for hat '{}'", hat_id);
                    continue;
                }
            }
        };

        // Execute the prompt
        let result = executor.execute(&prompt, stdout()).await?;

        // Process output
        if let Some(reason) = event_loop.process_output(&hat_id, &result.output, result.success) {
            print_termination(&reason, event_loop.state(), use_colors);
            break;
        }

        // Handle checkpointing (only if git_checkpoint is enabled)
        if config.git_checkpoint && event_loop.should_checkpoint() {
            if create_checkpoint(event_loop.state().iteration)? {
                event_loop.record_checkpoint();
            }
        }
    }

    Ok(())
}

fn print_termination(reason: &TerminationReason, state: &ralph_core::LoopState, use_colors: bool) {
    use colors::*;

    // Determine status color and message based on termination reason
    let (color, icon, label) = match reason {
        TerminationReason::CompletionPromise => (GREEN, "✓", "Completion promise detected"),
        TerminationReason::MaxIterations => (YELLOW, "⚠", "Maximum iterations reached"),
        TerminationReason::MaxRuntime => (YELLOW, "⚠", "Maximum runtime exceeded"),
        TerminationReason::MaxCost => (YELLOW, "⚠", "Maximum cost exceeded"),
        TerminationReason::ConsecutiveFailures => (RED, "✗", "Too many consecutive failures"),
        TerminationReason::Stopped => (CYAN, "■", "Manually stopped"),
    };

    let separator = "─".repeat(58);

    if use_colors {
        println!("\n{BOLD}┌{separator}┐{RESET}");
        println!(
            "{BOLD}│{RESET} {color}{BOLD}{icon}{RESET} Loop terminated: {color}{label}{RESET}"
        );
        println!("{BOLD}├{separator}┤{RESET}");
        println!("{BOLD}│{RESET}   Iterations:  {CYAN}{}{RESET}", state.iteration);
        println!(
            "{BOLD}│{RESET}   Elapsed:     {CYAN}{:.1}s{RESET}",
            state.elapsed().as_secs_f64()
        );
        if state.checkpoint_count > 0 {
            println!(
                "{BOLD}│{RESET}   Checkpoints: {CYAN}{}{RESET}",
                state.checkpoint_count
            );
        }
        if state.cumulative_cost > 0.0 {
            println!(
                "{BOLD}│{RESET}   Cost:        {CYAN}${:.2}{RESET}",
                state.cumulative_cost
            );
        }
        println!("{BOLD}└{separator}┘{RESET}");
    } else {
        println!("\n+{}+", "-".repeat(58));
        println!("| {icon} Loop terminated: {label}");
        println!("+{}+", "-".repeat(58));
        println!("|   Iterations:  {}", state.iteration);
        println!("|   Elapsed:     {:.1}s", state.elapsed().as_secs_f64());
        if state.checkpoint_count > 0 {
            println!("|   Checkpoints: {}", state.checkpoint_count);
        }
        if state.cumulative_cost > 0.0 {
            println!("|   Cost:        ${:.2}", state.cumulative_cost);
        }
        println!("+{}+", "-".repeat(58));
    }
}

/// Creates a git checkpoint and returns true if the commit succeeded.
fn create_checkpoint(iteration: u32) -> Result<bool> {
    info!("Creating checkpoint at iteration {}", iteration);

    let status = Command::new("git")
        .args(["add", "-A"])
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        warn!("git add failed");
        return Ok(false);
    }

    let message = format!("ralph: checkpoint at iteration {iteration}");
    let status = Command::new("git")
        .args(["commit", "-m", &message, "--allow-empty"])
        .status()
        .context("Failed to run git commit")?;

    if !status.success() {
        warn!("git commit failed (may be nothing to commit)");
        return Ok(false);
    }

    Ok(true)
}
