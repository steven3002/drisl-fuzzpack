use std::fs;
use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::time::Duration;
use std::collections::HashMap;
use wait_timeout::ChildExt;
use serde::{Deserialize, Serialize};
use rand::Rng;
use ciborium::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IPCResult {
    pub status: String,
    pub version: String,
    pub fingerprint: Option<String>,
    pub error_reason: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DivergenceSignature {
    pub python: String,
    pub js: String,
    pub go: String,
    pub rust: String,
}

#[derive(Serialize)]
pub struct DaslTestFixture {
    pub name: String,
    pub description: String,
    pub cbor_hex: String,
    pub vector_profile: String,
    pub expected_go_dasl_behavior: String,
    pub strict_drisl_compliant: bool,
}

// --- TERMINAL POLISH HELPER ---
fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "accept" => "✅ ACCEPT".to_string(),
        "reject" => "❌ REJECT".to_string(),
        "crash"  => "💥 CRASH ".to_string(),
        "timeout"=> "⏳ TIMEOUT".to_string(),
        _        => format!("❓ {}", status.to_uppercase()),
    }
}

// --- PROFILE-AWARE CONSENSUS LOGIC ---
#[derive(Debug, PartialEq)]
enum ConsensusResult {
    Unanimous,
    ExpectedProfileDivergence, // e.g., ATProto strictness vs Core DRISL leniency
    FatalSplitBrain,           // A true bug within a profile group
}

// --- THE ORACLE (BYTE-LEVEL PRE-CHECKER) ---
#[derive(Debug, PartialEq)]
enum ExpectedOracleBaseline {
    MustReject(String),
    ValidPayload,
}

fn byte_level_pre_check(payload: &[u8], profile: &str) -> ExpectedOracleBaseline {
    // Indefinite Lengths (0x9f arrays, 0xbf maps, 0x7f strings)
    // DRISL strictly forbids all indefinite lengths.
    if payload.contains(&0x9f) || payload.contains(&0xbf) || payload.contains(&0x7f) {
        return ExpectedOracleBaseline::MustReject("Contains forbidden indefinite length marker".to_string());
    }

    // Forbidden Tags (0xc1 is Tag 1)
    // DRISL strictly forbids all tags except Tag 42.
    if payload.contains(&0xc1) {
        return ExpectedOracleBaseline::MustReject("Contains forbidden non-42 CBOR tag".to_string());
    }

    // ATProto Strict Float Rules (0xf9 is Float16, 0xfa is Float32)
    // ATProto explicitly forbids floats smaller than 64-bit (0xfb).
    if profile == "atproto_model" && (payload.contains(&0xf9) || payload.contains(&0xfa)) {
        return ExpectedOracleBaseline::MustReject("Profile 'atproto_model' forbids 16/32-bit floats".to_string());
    }

    ExpectedOracleBaseline::ValidPayload
}

fn evaluate_consensus(py: &str, js: &str, go: &str, rs: &str) -> ConsensusResult {
    // Group A: Core DRISL Parsers
    let core_agree = py == rs;
    // Group B: ATProto Strict Parsers
    let atproto_agree = js == go;

    if core_agree && atproto_agree {
        if py == go {
            return ConsensusResult::Unanimous;
        } else {
            // Rust/Py agree, Go/JS agree, but they disagree with each other.
            // This is an expected Applicability Profile difference (e.g., Float16).
            return ConsensusResult::ExpectedProfileDivergence;
        }
    }
    
    // If the peers inside a specific profile group disagree, it's a fatal zero-day.
    ConsensusResult::FatalSplitBrain
}

fn load_manifest() -> HashMap<String, String> {
    let manifest_path = "../corpus/manifest.json";
    if let Ok(data) = fs::read_to_string(manifest_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

// --- THE DASL-TESTING EXPORTER ---
pub fn export_to_dasl_testing(manifest: &HashMap<String, String>) {
    println!("\n[EXPORTER] Translating findings to dasl-testing upstream format...");
    let findings_dir = "../corpus/findings";
    
    if let Ok(paths) = std::fs::read_dir(findings_dir) {
        let mut fixtures = Vec::new();

        for path in paths.filter_map(Result::ok) {
            let file_name = path.file_name().into_string().unwrap();
            if file_name.ends_with(".cbor") {
                let bytes = std::fs::read(&path.path()).unwrap();
                let hex_string = hex::encode(&bytes);

                // Derive original profile if possible, fallback to "core_drisl"
                let base_seed = file_name.split("_RAW").next().unwrap_or(&file_name);
                let profile = manifest.get(base_seed).cloned().unwrap_or_else(|| "core_drisl".to_string());

                fixtures.push(DaslTestFixture {
                    name: file_name.replace(".cbor", ""),
                    description: "Auto-generated split-brain regression case from FuzzPack".to_string(),
                    cbor_hex: hex_string,
                    vector_profile: profile, 
                    expected_go_dasl_behavior: "error".to_string(),
                    strict_drisl_compliant: false,
                });
            }
        }

        let fixture_json = serde_json::to_string_pretty(&fixtures).unwrap();
        std::fs::write("dasl_fixtures.json", fixture_json).expect("Failed to write fixtures");
        println!("✅ Exported {} regression cases to dasl_fixtures.json", fixtures.len());
    }
}

// --- IPC EXECUTION ENGINE ---
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

// --- AST MUTATOR ENGINE ---
fn mutate_ast(value: &mut Value, rng: &mut impl Rng) {
    match value {
        Value::Float(f) => {
            if rng.gen_bool(0.5) {
                *f = (*f as f32) as f64; 
            }
        },
        Value::Bytes(b) => {
            if rng.gen_bool(0.3) {
                let tag_num = if rng.gen_bool(0.5) { 42 } else { rng.gen_range(1..100) };
                *value = Value::Tag(tag_num, Box::new(Value::Bytes(b.clone())));
            }
        },
        Value::Array(arr) => {
            for item in arr.iter_mut() { mutate_ast(item, rng); }
        },
        Value::Map(map) => {
            if rng.gen_bool(0.2) {
                map.push((Value::Integer(rng.gen_range(1..100).into()), Value::Text("corrupted".to_string())));
            }
            if rng.gen_bool(0.1) && !map.is_empty() {
                let clone = map[0].clone();
                map.push(clone);
            }
            for (k, v) in map.iter_mut() {
                mutate_ast(k, rng);
                mutate_ast(v, rng);
            }
        },
        Value::Tag(_, inner) => mutate_ast(inner, rng),
        _ => {}
    }
}

pub fn generate_mutant(seed_bytes: &[u8], rng: &mut impl Rng) -> Option<Vec<u8>> {
    let mut ast: Value = ciborium::from_reader(seed_bytes).ok()?;
    mutate_ast(&mut ast, rng);

    let mut mutated_bytes = Vec::new();
    ciborium::into_writer(&ast, &mut mutated_bytes).unwrap();

    if rng.gen_bool(0.15) {
        mutated_bytes.push(rng.gen());
    }

    Some(mutated_bytes)
}

// --- OPTIMIZED MINIMIZER ENGINE ---
fn check_divergence_contract(test_payload: &[u8], target: &DivergenceSignature) -> bool {
    // FAST FAIL : Run Rust first 
    let rs = run_adapter("../adapters/serde_ipld/target/debug/serde_ipld_adapter", &[], test_payload, 1500).status;
    if rs != target.rust { return false; } // If Rust doesn't match, instantly abort!

    // FAST FAIL : Run Go 
    let go = run_adapter("../adapters/go-dasl/go-adapter", &[], test_payload, 1500).status;
    if go != target.go { return false; } // If Go doesn't match, instantly abort!

    // FAST FAIL : Run Python 
    let py = run_adapter("../adapters/python-libipld/venv/bin/python", &["../adapters/python-libipld/adapter.py"], test_payload, 1500).status;
    if py != target.python { return false; }

    // FAST FAIL : Run Node.js last 
    let js = run_adapter("node", &["../adapters/atcute/index.js"], test_payload, 1500).status;
    if js != target.js { return false; }

    // If it survived all 4 checks, the divergence is perfectly preserved!
    true
}


pub fn shrink_payload(original_payload: &[u8], target_signature: &DivergenceSignature) -> Vec<u8> {
    let mut best_payload = original_payload.to_vec();
    let mut improved = true;
    let mut pass_number = 1;

    println!("  [MINIMIZER] Starting reduction on {} bytes...", best_payload.len());

    while improved {
        improved = false;
        use std::io::Write;

        if best_payload.len() > 2 {
            let test_payload = best_payload[0..best_payload.len() - 1].to_vec();
            if check_divergence_contract(&test_payload, target_signature) {
                best_payload = test_payload;
                improved = true;
                print!("\r  [MINIMIZER] Pass {}: Snipped trailing byte! Size: {} bytes...    ", pass_number, best_payload.len());
                std::io::stdout().flush().unwrap();
                continue;
            }
        }

        for i in 0..best_payload.len() {
            if i % 5 == 0 {
                print!("\r  [MINIMIZER] Pass {}: Testing byte {}/{} (Current size: {})...    ", pass_number, i, best_payload.len(), best_payload.len());
                std::io::stdout().flush().unwrap();
            }

            let mut test_payload = best_payload.clone();
            test_payload.remove(i);

            if test_payload.is_empty() {
                continue; 
            }

            if check_divergence_contract(&test_payload, target_signature) {
                best_payload = test_payload;
                improved = true;
                print!("\r  [MINIMIZER] Pass {}: SUCCESS! Snipped byte at index {}. New size: {} bytes...    ", pass_number, i, best_payload.len());
                std::io::stdout().flush().unwrap();
                break; 
            }
        }
        pass_number += 1;
    }

    println!("\n  [MINIMIZER] Finished! Shrunk to optimal payload size: {} bytes.", best_payload.len());
    best_payload
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let manifest = load_manifest();

    // =================================================================
    // REGRESSION REPLAY & EXPORT (cargo run -- replay)
    // =================================================================
    if args.len() > 1 && args[1] == "replay" {
        println!("===================================================");
        println!("          DRISL Split-Brain FuzzPack:             ");
        println!("             [ REGRESSION REPLAY ]                 ");
        println!("===================================================");

        let findings_dir = "../corpus/findings";
        let paths = fs::read_dir(findings_dir).expect("Failed to read findings directory");
        
        let mut files: Vec<_> = paths.filter_map(Result::ok).collect();
        files.sort_by_key(|dir| dir.path());

        for path in files {
            let file_name = path.file_name().into_string().unwrap();
            if !file_name.ends_with(".cbor") { continue; }

            let base_seed = file_name.split("_RAW").next().unwrap_or(&file_name);
            let profile = manifest.get(base_seed).cloned().unwrap_or_else(|| "core_drisl".to_string());

            let input_bytes = fs::read(&path.path()).unwrap();
            
            // The Oracle inspects the bytes BEFORE the adapters run
            let oracle_verdict = byte_level_pre_check(&input_bytes, &profile);

            println!("\n▶ Vector: {} [Profile: {}]", file_name, profile.to_uppercase());
            println!("  Size  : {} bytes", input_bytes.len());
            
            match oracle_verdict {
                ExpectedOracleBaseline::MustReject(reason) => {
                    println!("  ORACLE: 🛑 EXPECTED TO REJECT ({})", reason);
                },
                ExpectedOracleBaseline::ValidPayload => {
                    println!("  ORACLE: 🟢 EXPECTED TO ACCEPT (Valid DRISL/ATProto Payload)");
                }
            }
            println!("-------------------------------------------------------------------------");
            let py_res = run_adapter("../adapters/python-libipld/venv/bin/python", &["../adapters/python-libipld/adapter.py"], &input_bytes, 1500);
            let js_res = run_adapter("node", &["../adapters/atcute/index.js"], &input_bytes, 1500);
            let go_res = run_adapter("../adapters/go-dasl/go-adapter", &[], &input_bytes, 1500);
            let rs_res = run_adapter("../adapters/serde_ipld/target/debug/serde_ipld_adapter", &[], &input_bytes, 1500);

            println!("  Python   | {:<10} | {}", format_status(&py_res.status), py_res.fingerprint.as_deref().unwrap_or_else(|| py_res.error_reason.as_deref().unwrap_or_default()));
            println!("  JS/TS    | {:<10} | {}", format_status(&js_res.status), js_res.fingerprint.as_deref().unwrap_or_else(|| js_res.error_reason.as_deref().unwrap_or_default()));
            println!("  Go       | {:<10} | {}", format_status(&go_res.status), go_res.fingerprint.as_deref().unwrap_or_else(|| go_res.error_reason.as_deref().unwrap_or_default()));
            println!("  Rust     | {:<10} | {}", format_status(&rs_res.status), rs_res.fingerprint.as_deref().unwrap_or_else(|| rs_res.error_reason.as_deref().unwrap_or_default()));
            
            let consensus = evaluate_consensus(&py_res.status, &js_res.status, &go_res.status, &rs_res.status);
            match consensus {
                ConsensusResult::Unanimous => println!("  STATUS   : 🟢 UNANIMOUS CONSENSUS"),
                ConsensusResult::ExpectedProfileDivergence => println!("  STATUS   : 🟡 EXPECTED APPLICABILITY DIVERGENCE (Not Applicable)"),
                ConsensusResult::FatalSplitBrain => println!("  STATUS   : 🔴 FATAL SPLIT-BRAIN (Bug)"),
            }
            println!("-------------------------------------------------------------------------");
        }

        export_to_dasl_testing(&manifest);
        return;
    }

    // =================================================================
    // LIVE MUTATION ENGINE (cargo run)
    // =================================================================
    println!("===================================================");
    println!("            DRISL Split-Brain FuzzPack             ");
    println!("             [ LIVE MUTATION ENGINE ]              ");
    println!("===================================================");

    let corpus_dir = "../corpus/seeds";
    let paths = fs::read_dir(corpus_dir).expect("Failed to read corpus directory");
    let mut seeds: Vec<Vec<u8>> = Vec::new();

    for path in paths.filter_map(Result::ok) {
        if path.file_name().to_str().unwrap().ends_with(".cbor") {
            let bytes = fs::read(&path.path()).expect("Failed to read seed");
            seeds.push(bytes);
        }
    }

    if seeds.is_empty() {
        panic!("No seeds found in corpus/seeds! Put some .cbor files there to fuzz.");
    }

    let findings_dir = "../corpus/findings";
    fs::create_dir_all(findings_dir).expect("Failed to create findings dir");

    let mut iterations = 0;
    let mut rng = rand::thread_rng();

    println!("Loaded {} seeds. Commencing live fuzzing...", seeds.len());
    println!("Press Ctrl+C to stop.\n");

    loop {
        iterations += 1;

        let seed_idx = rng.gen_range(0..seeds.len());
        let test_payload = match generate_mutant(&seeds[seed_idx], &mut rng) {
            Some(p) => p,
            None => continue,
        };

        let py_res = run_adapter("../adapters/python-libipld/venv/bin/python", &["../adapters/python-libipld/adapter.py"], &test_payload, 1500);
        let js_res = run_adapter("node", &["../adapters/atcute/index.js"], &test_payload, 1500);
        let go_res = run_adapter("../adapters/go-dasl/go-adapter", &[], &test_payload, 1500);
        let rs_res = run_adapter("../adapters/serde_ipld/target/debug/serde_ipld_adapter", &[], &test_payload, 1500);

        if iterations % 10 == 0 {
            use std::io::Write;
            print!("\rFuzzing iteration: {} | Latest Payload: {} bytes...", iterations, test_payload.len());
            std::io::stdout().flush().unwrap();
        }
        
        let consensus = evaluate_consensus(&py_res.status, &js_res.status, &go_res.status, &rs_res.status);

        // We ONLY care if the consensus is a FATAL SPLIT BRAIN. 
        // If it's an expected profile divergence, the fuzzer safely ignores it!
        if consensus == ConsensusResult::FatalSplitBrain {
            println!("\n\n🚨 FATAL DIVERGENCE DETECTED ON ITERATION {} 🚨", iterations);
            println!("  Python : {}", py_res.status);
            println!("  JS/TS  : {}", js_res.status);
            println!("  Go     : {}", go_res.status);
            println!("  Rust   : {}", rs_res.status);

            let signature = DivergenceSignature {
                python: py_res.status.clone(),
                js: js_res.status.clone(),
                go: go_res.status.clone(),
                rust: rs_res.status.clone(),
            };

            let raw_filename = format!("{}/divergence_{}_RAW.cbor", findings_dir, iterations);
            fs::write(&raw_filename, &test_payload).expect("Failed to save raw finding");
            println!("💾 Saved RAW payload to: {} (in case minimizer hangs)", raw_filename);

            let optimized_payload = shrink_payload(&test_payload, &signature);

            let min_filename = format!("{}/divergence_{}_MINIMIZED.cbor", findings_dir, iterations);
            fs::write(&min_filename, &optimized_payload).expect("Failed to save minimized finding");
            
            println!("💾 Saved MINIMIZED payload to: {}", min_filename);
            println!("---------------------------------------------------");
        }
    }
}