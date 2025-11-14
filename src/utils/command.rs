use anyhow::{anyhow, Result};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;

pub async fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<std::process::Output> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn {}: {}", cmd, e))?;

    let start = Instant::now();

    // Poll for completion with timeout
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process completed, collect output
                let output = child
                    .wait_with_output()
                    .await
                    .map_err(|e| anyhow!("Failed to read output: {}", e))?;
                return Ok(output);
            }
            Ok(None) => {
                // Still running, check timeout
                if start.elapsed() > timeout {
                    // Timeout exceeded, kill the process
                    let _ = child.kill().await;
                    let _ = child.wait().await; // Clean up
                    return Err(anyhow!(
                        "Command '{}' timed out after {:?}",
                        cmd,
                        timeout
                    ));
                }
                // Sleep briefly before next check
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => {
                let _ = child.kill().await;
                return Err(anyhow!("Error waiting for command: {}", e));
            }
        }
    }
}

pub async fn command_exists(cmd: &str) -> bool {
    if let Ok(output) = run_command_with_timeout(
        "which",
        &[cmd],
        Duration::from_secs(2),
    )
    .await {
        output.status.success()
    } else {
        false
    }
}

