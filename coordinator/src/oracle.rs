use crate::models::{ExpectedOracleBaseline, ConsensusResult};

pub fn byte_level_pre_check(payload: &[u8], profile: &str) -> ExpectedOracleBaseline {
    if payload.contains(&0x9f) || payload.contains(&0xbf) || payload.contains(&0x7f) {
        return ExpectedOracleBaseline::MustReject("Contains forbidden indefinite length marker".to_string());
    }
    if payload.contains(&0xc1) {
        return ExpectedOracleBaseline::MustReject("Contains forbidden non-42 CBOR tag".to_string());
    }
    if profile == "atproto_model" && (payload.contains(&0xf9) || payload.contains(&0xfa)) {
        return ExpectedOracleBaseline::MustReject("Profile 'atproto_model' forbids 16/32-bit floats".to_string());
    }
    ExpectedOracleBaseline::ValidPayload
}

pub fn evaluate_consensus(py: &str, js: &str, go: &str, rs: &str) -> ConsensusResult {
    let core_agree = py == rs;
    let atproto_agree = js == go;

    if core_agree && atproto_agree {
        if py == go {
            return ConsensusResult::Unanimous;
        } else {
            return ConsensusResult::ExpectedProfileDivergence;
        }
    }
    ConsensusResult::FatalSplitBrain
}