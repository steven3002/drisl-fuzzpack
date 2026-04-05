use std::io::{self, Read};
use std::panic;
use std::process;
use serde::Serialize;
use ipld_core::ipld::Ipld;
use base64::{engine::general_purpose::STANDARD, Engine as _};

const TARGET_VERSION: &str = "0.1.2";

#[derive(Serialize)]
struct IPCResult {
    status: &'static str,
    version: &'static str,
    fingerprint: Option<String>,
    error_reason: Option<String>,
}

fn print_result(status: &'static str, fingerprint: Option<String>, error_reason: Option<String>) {
    let result = IPCResult {
        status,
        version: TARGET_VERSION,
        fingerprint,
        error_reason,
    };
    
    if let Ok(json_str) = serde_json::to_string(&result) {
        println!("{}", json_str);
    }
    process::exit(0);
}

fn generate_semantic_fingerprint(data: &Ipld) -> String {
    match data {
        Ipld::Integer(i) => format!("int:{}", i),
        Ipld::Float(f) => format!("float:0x{:016x}", f.to_bits()),
        Ipld::String(s) => format!("str:{}", s),
        Ipld::Bytes(b) => format!("bytes:{}", STANDARD.encode(b)),
        Ipld::Bool(b) => format!("bool:{}", b),
        Ipld::List(l) => {
            let parts: Vec<String> = l.iter().map(|item| generate_semantic_fingerprint(item)).collect();
            format!("[{}]", parts.join(","))
        },
        Ipld::Map(m) => {
            // BTreeMap is automatically sorted alphabetically
            let parts: Vec<String> = m.iter().map(|(k, v)| {
                let key_str = format!("str:{}", k); 
                let val_str = generate_semantic_fingerprint(v);
                format!("[{},{}]", key_str, val_str)
            }).collect();
            format!("[{}]", parts.join(","))
        },
        Ipld::Link(c) => format!("cid:{}", c),
        Ipld::Null => "unknown:null".to_string(),
    }
}

fn main() {
    // Hijack panic hook to prevent ugly stderr dumps
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic occurred"
        };
        
        let err_reason = format!("panic: {}", msg);
        print_result("crash", None, Some(err_reason));
    }));

    // Read bytes
    let mut input_bytes = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut input_bytes) {
        print_result("crash", None, Some(format!("failed to read stdin: {}", e)));
        return;
    }

    if input_bytes.is_empty() {
        print_result("reject", None, Some("empty input".to_string()));
        return;
    }

    // Attempt decode and print fingerprint
    match serde_ipld_dagcbor::from_slice::<Ipld>(&input_bytes) {
        Ok(parsed_data) => {
            let fingerprint = generate_semantic_fingerprint(&parsed_data);
            print_result("accept", Some(fingerprint), None);
        }
        Err(e) => {
            print_result("reject", None, Some(e.to_string()));
        }
    }
}