use serde::{Deserialize, Serialize};

/// POST /processor.Processor/Calculate request body.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcRequest {
    pub calc_type: String,
    pub centirates: Vec<u32>,
    #[serde(flatten)]
    pub input: InputKind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputKind {
    #[serde(rename = "normalizedHash")]
    Hash(String),
    #[serde(rename = "file")]
    File(FileInput),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInput {
    pub extension: String,
    /// Base64-encoded raw file bytes.
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcResponse {
    pub normalized_hash: String,
    pub results: Vec<RateResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateResult {
    pub centirate: u32,
    pub rating: f64,
    pub mania_skill: ManiaSkill,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManiaSkill {
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}
