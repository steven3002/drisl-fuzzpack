use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::time::Duration;
use wait_timeout::ChildExt;
use crate::models::IPCResult;

pub fn run_adapter(executable: &str, args: &[&str], input_bytes: &[u8], timeout_ms: u64) -> IPCResult {
    let mut child = match Command::new(executable)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return fallback_result("crash", Some(format!("Failed to spawn: {}", e))),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input_bytes);
    } 

    let timeout = Duration::from_millis(timeout_ms);
    match child.wait_timeout(timeout) {
        Ok(Some(_status)) => {
            let mut stdout = child.stdout.take().unwrap();
            let mut output_string = String::new();
            stdout.read_to_string(&mut output_string).unwrap_or_default();

            match serde_json::from_str::<IPCResult>(&output_string) {
                Ok(result) => result,
                Err(_e) => {
                    let mut stderr = child.stderr.take().unwrap();
                    let mut err_string = String::new();
                    stderr.read_to_string(&mut err_string).unwrap_or_default();
                    let clean_err = err_string.trim().replace('\n', " | ");
                    fallback_result("crash", Some(format!("EOF. Log: {}", clean_err)))
                }
            }
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait(); 
            fallback_result("timeout", None)
        }
        Err(e) => {
            let _ = child.kill();
            fallback_result("crash", Some(format!("OS error: {}", e)))
        }
    }
}

fn fallback_result(status: &str, error_reason: Option<String>) -> IPCResult {
    IPCResult {
        status: status.to_string(),
        version: "unknown".to_string(),
        fingerprint: None,
        error_reason,
    }
}