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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
  pub id: String,
  pub username: String,
  pub discriminator: Option<String>,
  pub global_name: Option<String>,
  pub avatar: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
  pub state: Option<String>,
  pub details: Option<String>,
  pub state_url: Option<String>,
  pub details_url: Option<String>,
  pub assets: Option<Assets>,
  pub buttons: Option<Vec<Button>>,
  pub party: Option<Party>,
  pub timestamps: Option<Timestamps>,
  // 0 Playing, 2 Listening, 3 Watching, 5 Competing
  pub activity_type: Option<u8>,
  // 0 Name, 1 State, 2 Details — compact headline.
  pub status_display_type: Option<u8>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn activity_accepts_url_fields_camelcase() {
    let json = r#"{ "state": "s", "stateUrl": "https://a", "detailsUrl": "https://b" }"#;
    let a: Activity = serde_json::from_str(json).unwrap();
    assert_eq!(a.state.as_deref(), Some("s"));
    assert_eq!(a.state_url.as_deref(), Some("https://a"));
    assert_eq!(a.details_url.as_deref(), Some("https://b"));
  }

  #[test]
  fn user_serializes_camelcase() {
    let u = User {
      id: "1".into(),
      username: "bob".into(),
      discriminator: Some("0".into()),
      global_name: Some("Bob".into()),
      avatar: None,
    };
    let v = serde_json::to_value(&u).unwrap();
    assert_eq!(v["globalName"], "Bob");
    assert_eq!(v["id"], "1");
  }
}
