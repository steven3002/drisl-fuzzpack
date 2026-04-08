mod models;
mod oracle;
mod runner;
mod mutator;
mod minimizer;
mod exporter;

use std::fs;
use std::io::Write;
use rand::Rng;
use models::{ConsensusResult, DivergenceSignature, ExpectedOracleBaseline};
use oracle::{byte_level_pre_check, evaluate_consensus};
use runner::run_adapter;
use mutator::generate_mutant;
use minimizer::shrink_payload;
use exporter::{load_manifest, export_to_dasl_testing, format_status};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let manifest = load_manifest();

    if args.len() > 1 && args[1] == "replay" {
        println!("===================================================");
        println!("          DRISL Split-Brain FuzzPack:              ");
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
            print!("\rFuzzing iteration: {} | Latest Payload: {} bytes...", iterations, test_payload.len());
            std::io::stdout().flush().unwrap();
        }
        
        let consensus = evaluate_consensus(&py_res.status, &js_res.status, &go_res.status, &rs_res.status);

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