use std::io::Write;
use crate::models::DivergenceSignature;
use crate::runner::run_adapter;

fn check_divergence_contract(test_payload: &[u8], target: &DivergenceSignature) -> bool {
    let rs = run_adapter("../adapters/serde_ipld/target/debug/serde_ipld_adapter", &[], test_payload, 1500).status;
    if rs != target.rust { return false; }

    let go = run_adapter("../adapters/go-dasl/go-adapter", &[], test_payload, 1500).status;
    if go != target.go { return false; }

    let py = run_adapter("../adapters/python-libipld/venv/bin/python", &["../adapters/python-libipld/adapter.py"], test_payload, 1500).status;
    if py != target.python { return false; }

    let js = run_adapter("node", &["../adapters/atcute/index.js"], test_payload, 1500).status;
    if js != target.js { return false; }

    true
}

pub fn shrink_payload(original_payload: &[u8], target_signature: &DivergenceSignature) -> Vec<u8> {
    let mut best_payload = original_payload.to_vec();
    let mut improved = true;
    let mut pass_number = 1;

    println!("  [MINIMIZER] Starting reduction on {} bytes...", best_payload.len());

    while improved {
        improved = false;

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

            if test_payload.is_empty() { continue; }

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