use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::time::Duration;
use wait_timeout::ChildExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IPCResult {
    pub status: String,
    pub version: String,
    pub fingerprint: Option<String>,
    pub error_reason: Option<String>,
}

/// Spawns a target adapter, pipes binary input, and strictly enforces the timeout.
pub fn run_adapter(
    executable: &str,
    args: &[&str],
    input_bytes: &[u8],
    timeout_ms: u64,
) -> IPCResult {
    
    // Spawn the child process with piped streams
    let mut child = match Command::new(executable)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture to prevent terminal spam
        .spawn() 
    {
        Ok(c) => c,
        Err(e) => return fallback_result("crash", Some(format!("Failed to spawn: {}", e))),
    };

    // Write binary data and EXPLICITLY DROP stdin to signal EOF
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(input_bytes) {
            return fallback_result("crash", Some(format!("Failed to write stdin: {}", e)));
        }
    } // stdin goes out of scope here and is closed.

    // Enforce the strict budget
    let timeout = Duration::from_millis(timeout_ms);
    match child.wait_timeout(timeout) {
        Ok(Some(_status)) => {
            // Process finished within timeout
            let mut stdout = child.stdout.take().unwrap();
            let mut output_string = String::new();
            stdout.read_to_string(&mut output_string).unwrap_or_default();

            // Parse the JSON contract
            match serde_json::from_str::<IPCResult>(&output_string) {
                Ok(result) => result,
                Err(e) => fallback_result(
                    "crash", 
                    Some(format!("Adapter returned invalid JSON: {}. Output was: {}", e, output_string))
                ),
            }
        }
        Ok(None) => {
            // TIMEOUT REACHED - Kill the zombie process
            let _ = child.kill();
            let _ = child.wait(); 
            fallback_result("timeout", None)
        }
        Err(e) => {
            // System error waiting for child
            let _ = child.kill();
            fallback_result("crash", Some(format!("OS error waiting for process: {}", e)))
        }
    }
}

// Utility to generate safe responses when the IPC completely breaks down
fn fallback_result(status: &str, error_reason: Option<String>) -> IPCResult {
    IPCResult {
        status: status.to_string(),
        version: "unknown".to_string(),
        fingerprint: None,
        error_reason,
    }
}