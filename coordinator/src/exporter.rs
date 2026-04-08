use std::collections::HashMap;
use std::fs;
use crate::models::DaslTestFixture;


pub fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "accept" => "✅ ACCEPT".to_string(),
        "reject" => "❌ REJECT".to_string(),
        "crash"  => "💥 CRASH ".to_string(),
        "timeout"=> "⏳ TIMEOUT".to_string(),
        _        => format!("❓ {}", status.to_uppercase()),
    }
}


pub fn load_manifest() -> HashMap<String, String> {
    let manifest_path = "../corpus/manifest.json";
    if let Ok(data) = fs::read_to_string(manifest_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }

}


pub fn export_to_dasl_testing(manifest: &HashMap<String, String>) {
    println!("\n[EXPORTER] Translating findings to dasl-testing upstream format...");
    let findings_dir = "../corpus/findings";
    
    if let Ok(paths) = fs::read_dir(findings_dir) {
        let mut fixtures = Vec::new();

        for path in paths.filter_map(Result::ok) {
            let file_name = path.file_name().into_string().unwrap();
            if file_name.ends_with(".cbor") {
                let bytes = fs::read(&path.path()).unwrap();
                let hex_string = hex::encode(&bytes);

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
        fs::write("dasl_fixtures.json", fixture_json).expect("Failed to write fixtures");
        println!("✅ Exported {} regression cases to dasl_fixtures.json", fixtures.len());
    }
}