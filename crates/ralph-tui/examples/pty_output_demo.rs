//! Demo: TUI running `ls -la` with live PTY output
//!
//! Run with: cargo run --example pty_output_demo

use ralph_adapters::pty_handle::PtyHandle;
use ralph_tui::Tui;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::{mpsc, watch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Spawn ls -la command
    let mut child = Command::new("ls")
        .arg("-la")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Create channels for PTY handle
    let (output_tx, output_rx) = mpsc::unbounded_channel();
    let (input_tx, _input_rx) = mpsc::unbounded_channel();
    let (control_tx, _control_rx) = mpsc::unbounded_channel();
    let (_terminated_tx, terminated_rx) = watch::channel(false);

    // Clone output_tx before moving it
    let output_tx_clone = output_tx.clone();

    // Spawn task to read stdout and send to output channel
    tokio::spawn(async move {
        let mut stdout = stdout;
        let mut buf = vec![0u8; 1024];
        while let Ok(n) = stdout.read(&mut buf).await {
            if n == 0 {
                break;
            }
            let _ = output_tx.send(buf[..n].to_vec());
        }
    });

    // Spawn task to read stderr and send to output channel
    tokio::spawn(async move {
        let mut stderr = stderr;
        let mut buf = vec![0u8; 1024];
        while let Ok(n) = stderr.read(&mut buf).await {
            if n == 0 {
                break;
            }
            let _ = output_tx_clone.send(buf[..n].to_vec());
        }
    });

    // Create PTY handle
    let pty_handle = PtyHandle {
        output_rx,
        input_tx,
        control_tx,
        terminated_rx,
    };

    // Create TUI with PTY handle
    let tui = Tui::new().with_pty(pty_handle);

    // Run TUI (will exit on Ctrl+C)
    tui.run().await?;

    // Wait for command to finish
    let _ = child.wait().await;

    Ok(())
}
