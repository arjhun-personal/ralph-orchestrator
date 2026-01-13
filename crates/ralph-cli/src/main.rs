//! # ralph-cli
//!
//! Binary entry point for the Ralph Orchestrator.
//!
//! This crate provides:
//! - CLI argument parsing using `clap`
//! - Application initialization and configuration
//! - Entry point to the headless orchestration loop
//! - Event history viewing via `ralph events`

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use ralph_adapters::{detect_backend, CliBackend, CliExecutor};
use ralph_core::{EventHistory, EventLogger, EventLoop, EventParser, EventRecord, RalphConfig, TerminationReason};
use ralph_proto::{Event, HatId};
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

/// Output format for events command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    #[default]
    Table,
    /// JSON format for programmatic access
    Json,
}

/// ANSI color codes for terminal output.
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
}

/// Ralph Orchestrator - Multi-agent orchestration framework
#[derive(Parser, Debug)]
#[command(name = "ralph", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // ─────────────────────────────────────────────────────────────────────────
    // Global options (available for all subcommands)
    // ─────────────────────────────────────────────────────────────────────────

    /// Path to configuration file
    #[arg(short, long, default_value = "ralph.yml", global = true)]
    config: PathBuf,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Color output mode (auto, always, never)
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    color: ColorMode,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the orchestration loop (default if no subcommand given)
    Run(RunArgs),

    /// View event history for debugging
    Events(EventsArgs),
}

/// Arguments for the run subcommand.
#[derive(Parser, Debug)]
struct RunArgs {
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
}

/// Arguments for the events subcommand.
#[derive(Parser, Debug)]
struct EventsArgs {
    /// Show only the last N events
    #[arg(long)]
    last: Option<usize>,

    /// Filter by topic (e.g., "build.blocked")
    #[arg(long)]
    topic: Option<String>,

    /// Filter by iteration number
    #[arg(long)]
    iteration: Option<u32>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    /// Path to events file (default: .agent/events.jsonl)
    #[arg(long)]
    file: Option<PathBuf>,

    /// Clear the event history
    #[arg(long)]
    clear: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Some(Commands::Run(args)) => run_command(cli.config, cli.verbose, cli.color, args).await,
        Some(Commands::Events(args)) => events_command(cli.color, args),
        None => {
            // Default to run with no overrides (backwards compatibility)
            let args = RunArgs {
                prompt: None,
                max_iterations: None,
                completion_promise: None,
                dry_run: false,
            };
            run_command(cli.config, cli.verbose, cli.color, args).await
        }
    }
}

async fn run_command(
    config_path: PathBuf,
    verbose: bool,
    color_mode: ColorMode,
    args: RunArgs,
) -> Result<()> {
    info!("Ralph Orchestrator v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let mut config = if config_path.exists() {
        RalphConfig::from_file(&config_path)
            .with_context(|| format!("Failed to load config from {:?}", config_path))?
    } else {
        warn!("Config file {:?} not found, using defaults", config_path);
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
    if verbose {
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
    run_loop(config, color_mode).await
}

fn events_command(color_mode: ColorMode, args: EventsArgs) -> Result<()> {
    let use_colors = color_mode.should_use_colors();

    let history = match args.file {
        Some(path) => EventHistory::new(path),
        None => EventHistory::default_path(),
    };

    // Handle clear command
    if args.clear {
        history.clear()?;
        if use_colors {
            println!("{}✓{} Event history cleared", colors::GREEN, colors::RESET);
        } else {
            println!("Event history cleared");
        }
        return Ok(());
    }

    if !history.exists() {
        if use_colors {
            println!(
                "{}No event history found.{} Run `ralph` to generate events.",
                colors::DIM,
                colors::RESET
            );
        } else {
            println!("No event history found. Run `ralph` to generate events.");
        }
        return Ok(());
    }

    // Read and filter events
    let mut records = if let Some(n) = args.last {
        history.read_last(n)?
    } else if let Some(ref topic) = args.topic {
        history.filter_by_topic(topic)?
    } else if let Some(iteration) = args.iteration {
        history.filter_by_iteration(iteration)?
    } else {
        history.read_all()?
    };

    // Apply secondary filters (topic + last, etc.)
    if args.last.is_some() {
        if let Some(ref topic) = args.topic {
            records.retain(|r| r.topic == *topic);
        }
        if let Some(iteration) = args.iteration {
            records.retain(|r| r.iteration == iteration);
        }
    }

    if records.is_empty() {
        if use_colors {
            println!("{}No matching events found.{}", colors::DIM, colors::RESET);
        } else {
            println!("No matching events found.");
        }
        return Ok(());
    }

    match args.format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&records)?;
            println!("{json}");
        }
        OutputFormat::Table => {
            print_events_table(&records, use_colors);
        }
    }

    Ok(())
}

fn print_events_table(records: &[ralph_core::EventRecord], use_colors: bool) {
    use colors::*;

    // Header
    if use_colors {
        println!(
            "{BOLD}{DIM}  # │ Iteration │ Hat           │ Topic              │ Triggered      │ Payload{RESET}"
        );
        println!(
            "{DIM}────┼───────────┼───────────────┼────────────────────┼────────────────┼─────────────────{RESET}"
        );
    } else {
        println!(
            "  # | Iteration | Hat           | Topic              | Triggered      | Payload"
        );
        println!(
            "----|-----------|---------------|--------------------|-----------------|-----------------"
        );
    }

    for (i, record) in records.iter().enumerate() {
        let topic_color = get_topic_color(&record.topic);
        let triggered = record.triggered.as_deref().unwrap_or("-");
        let payload_preview = if record.payload.len() > 40 {
            format!("{}...", &record.payload[..40].replace('\n', " "))
        } else {
            record.payload.replace('\n', " ")
        };

        if use_colors {
            println!(
                "{DIM}{:>3}{RESET} │ {:>9} │ {:<13} │ {topic_color}{:<18}{RESET} │ {:<14} │ {DIM}{}{RESET}",
                i + 1,
                record.iteration,
                truncate(&record.hat, 13),
                truncate(&record.topic, 18),
                truncate(triggered, 14),
                payload_preview
            );
        } else {
            println!(
                "{:>3} | {:>9} | {:<13} | {:<18} | {:<14} | {}",
                i + 1,
                record.iteration,
                truncate(&record.hat, 13),
                truncate(&record.topic, 18),
                truncate(triggered, 14),
                payload_preview
            );
        }
    }

    // Footer
    if use_colors {
        println!(
            "\n{DIM}Total: {} events{RESET}",
            records.len()
        );
    } else {
        println!("\nTotal: {} events", records.len());
    }
}

fn get_topic_color(topic: &str) -> &'static str {
    use colors::*;
    if topic.starts_with("task.") {
        CYAN
    } else if topic.starts_with("build.done") {
        GREEN
    } else if topic.starts_with("build.blocked") {
        RED
    } else if topic.starts_with("build.") {
        YELLOW
    } else if topic.starts_with("review.") {
        MAGENTA
    } else {
        BLUE
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

async fn run_loop(config: RalphConfig, color_mode: ColorMode) -> Result<()> {
    let use_colors = color_mode.should_use_colors();

    // Read prompt file
    let prompt_content = std::fs::read_to_string(&config.event_loop.prompt_file)
        .with_context(|| format!("Failed to read prompt file: {}", config.event_loop.prompt_file))?;

    // Initialize event loop
    let mut event_loop = EventLoop::new(config.clone());
    event_loop.initialize(&prompt_content);

    // Initialize event logger for debugging
    let mut event_logger = EventLogger::default_path();

    // Log initial task.start event
    let start_event = Event::new("task.start", &prompt_content);
    let start_record = EventRecord::new(0, "loop", &start_event, Some(&HatId::new("planner")));
    if let Err(e) = event_logger.log(&start_record) {
        warn!("Failed to log start event: {}", e);
    }

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

        // Log events from output before processing
        log_events_from_output(&mut event_logger, iteration, &hat_id, &result.output, event_loop.registry());

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

/// Logs events parsed from output to the event history file.
fn log_events_from_output(
    logger: &mut EventLogger,
    iteration: u32,
    hat_id: &HatId,
    output: &str,
    registry: &ralph_core::HatRegistry,
) {
    let parser = EventParser::new();
    let events = parser.parse(output);

    for event in events {
        // Determine which hat will be triggered by this event
        let triggered = registry.find_by_trigger(event.topic.as_str());

        let record = EventRecord::new(iteration, hat_id.to_string(), &event, triggered);

        if let Err(e) = logger.log(&record) {
            warn!("Failed to log event {}: {}", event.topic, e);
        }
    }
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
