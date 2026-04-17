use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
