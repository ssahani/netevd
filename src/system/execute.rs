// SPDX-License-Identifier: LGPL-3.0-or-later

//! Script execution

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Execute all scripts in a directory with provided environment variables
pub async fn execute_scripts(
    directory: &str,
    env_vars: HashMap<String, String>,
) -> Result<()> {
    let dir_path = Path::new(directory);

    if !dir_path.exists() {
        debug!("Script directory does not exist: {}", directory);
        return Ok(());
    }

    if !dir_path.is_dir() {
        warn!("Script path is not a directory: {}", directory);
        return Ok(());
    }

    // Read directory entries
    let mut entries = fs::read_dir(dir_path)
        .await
        .with_context(|| format!("Failed to read directory: {}", directory))?;

    let mut scripts = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .with_context(|| format!("Failed to read directory entry in: {}", directory))?
    {
        let path = entry.path();

        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        // Check if file is executable
        let metadata = fs::metadata(&path)
            .await
            .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            debug!("Skipping non-executable file: {:?}", path);
            continue;
        }

        scripts.push(path);
    }

    // Sort scripts by name for deterministic execution order
    scripts.sort();

    if scripts.is_empty() {
        debug!("No executable scripts found in: {}", directory);
        return Ok(());
    }

    info!("Executing {} scripts in: {}", scripts.len(), directory);

    // Execute each script
    for script_path in scripts {
        match execute_script(&script_path, &env_vars).await {
            Ok(_) => {
                info!("Successfully executed script: {:?}", script_path);
            }
            Err(e) => {
                warn!("Failed to execute script {:?}: {}", script_path, e);
            }
        }
    }

    Ok(())
}

/// Execute a single script with environment variables
async fn execute_script(script_path: &Path, env_vars: &HashMap<String, String>) -> Result<()> {
    debug!("Executing script: {:?}", script_path);

    let mut cmd = Command::new(script_path);

    // Set environment variables
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    // Configure stdio
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Execute the command
    let output = cmd
        .output()
        .await
        .with_context(|| format!("Failed to execute script: {:?}", script_path))?;

    // Log output
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Script stdout: {}", stdout.trim());
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            debug!("Script stderr: {}", stderr.trim());
        } else {
            warn!("Script stderr: {}", stderr.trim());
        }
    }

    // Check exit status
    if !output.status.success() {
        anyhow::bail!(
            "Script {:?} exited with status: {}",
            script_path,
            output.status
        );
    }

    Ok(())
}
