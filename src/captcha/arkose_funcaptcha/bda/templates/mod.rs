use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BDATemplate {
    #[serde(rename = "document__referrer")]
    pub document_referrer: Option<String>,
    #[serde(rename = "window__ancestor_origins")]
    pub window_ancestor_origins: Option<Vec<String>>,
    #[serde(rename = "window__tree_index")]
    pub window_tree_index: Option<Vec<i32>>,
    #[serde(rename = "window__tree_structure")]
    pub window_tree_structure: Option<String>,
    #[serde(rename = "window__location_href")]
    pub window_location_href: Option<String>,
    #[serde(rename = "client_config__sitedata_location_href")]
    pub client_config_sitedata_location_href: Option<String>,
    #[serde(rename = "client_config__surl")]
    pub client_config_surl: Option<String>,
    #[serde(rename = "client_config__language")]
    pub client_config_language: Option<String>,
}

impl BDATemplate {
    pub(crate) fn update(&self, mut enhanced_fp: &mut Value) -> Option<()> {
        let mut s = serde_json::to_value(&self).ok()?;
        for v in enhanced_fp.as_array_mut()? {
            let key = v.get("key")?.as_str()?;
            if let Some(val) = s.get_mut(key) {
                v["value"] = val.take();
            }
        }
        Some(())
    }
}