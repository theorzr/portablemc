//! JSON schemas structures for serde deserialization.

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};

use crate::base;


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MojangManifest {
    /// A map associated the latest versions.
    pub latest: MojangManifestLatest,
    /// List of all versions.
    pub versions: Vec<MojangManifestVersion>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct MojangManifestLatest {
    pub release: String,
    pub snapshot: String,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MojangManifestVersion {
    pub id: String,
    pub r#type: String,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    #[serde(flatten)]
    pub download: base::serde::Download,
    /// Unknown, used by official launcher.
    pub compliance_level: Option<u32>,
}
