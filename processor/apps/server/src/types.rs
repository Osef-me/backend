use bridge::ManiaSkill;
use serde::{Deserialize, Serialize};

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
