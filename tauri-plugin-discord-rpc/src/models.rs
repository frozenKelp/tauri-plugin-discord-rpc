use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Assets {
  pub large_image: Option<String>,
  pub large_text: Option<String>,
  pub small_image: Option<String>,
  pub small_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Button {
  pub label: String,
  pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
  pub id: Option<String>,
  pub current_size: Option<i32>,
  pub max_size: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timestamps {
  pub start: Option<i64>,
  pub end: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
  pub state: Option<String>,
  pub details: Option<String>,
  pub assets: Option<Assets>,
  pub buttons: Option<Vec<Button>>,
  pub party: Option<Party>,
  pub timestamps: Option<Timestamps>,
}
