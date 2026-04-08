use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq)]
pub enum ConsensusResult {
    Unanimous,
    ExpectedProfileDivergence,
    FatalSplitBrain,
}

#[derive(Debug, PartialEq)]
pub enum ExpectedOracleBaseline {
    MustReject(String),
    ValidPayload,
}