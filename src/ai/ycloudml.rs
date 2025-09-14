//! Example of using `Completion` interface for interacting with Yandex AI.
//!
//! This async function demonstrates a simple workflow with Yandex AI models:
//! - Load environment variables (YC_IAM_TOKEN, YC_IAM_FOLDER).
//! - Create a new session with a unique UUID.
//! - Initialize a `Completion` model with a system prompt and a user query.
//! - Run the model to get a response.
//! - Extract the assistant's first response and continue the conversation.
//!
//! This provides a simple, chainable interface for building conversational flows
//! with Yandex Cloud AI services.
//!
//! ```rust
//! use rqrs::prelude::*;
//!
//! async fn run() -> Result<()> {
//!     dotenv::dotenv().ok();
//!
//!     let jwt = std::env::var("YC_IAM_TOKEN")?;
//!     let folder_id = std::env::var("YC_IAM_FOLDER")?;
//!     let session_id = uuid::Uuid::new_v4().hyphenated().to_string();
//!
//!     let payload = complition::Payload::new(folder_id)
//!         .system("You are financial bot")?
//!         .user("who are you?")?;
//!     let rs = payload.run(&jwt, &session_id).await.unwrap();
//!
//!     let assistant = complition::Payload::assistant_text_first(rs)?;
//!     let payload = payload
//!         .assistant(&assistant)?
//!         .user("What u can to do?")?;
//!     payload.run(&jwt, &session_id).await.unwrap();
//!     Ok(())
//! }
//! ```

use crate::Result;
use serde::{Deserialize, Serialize};

static URL: &str = "https://llm.api.cloud.yandex.net";
static IAM_URL: &str = "https://iam.api.cloud.yandex.net";

pub struct YCloudML;

#[derive(Deserialize, Serialize)]
struct Response {
    #[serde(rename = "iamToken")]
    iam_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

impl YCloudML {
    pub fn new() -> Self {
        YCloudML
    }

    pub async fn oauth<S>(&self, token: S) -> Result<String>
    where
        S: Into<String>,
    {
        let payload = serde_json::json!({
            "yandexPassportOauthToken":token.into()
        });

        let client = reqwest::Client::new();
        let rs: serde_json::Value = client
            .post(format!("{IAM_URL}/iam/v1/tokens"))
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;
        let rs: Response = serde_json::from_value(rs)?;

        Ok(rs.iam_token)
    }
}

impl Default for YCloudML {
    fn default() -> Self {
        Self::new()
    }
}

pub mod complition;
pub mod image;
pub mod speechkit;

#[derive(Serialize, Debug, Clone)]
pub enum Model {
    Completion {
        payload: complition::Payload,
        uri: String,
        method: String,
    },
    Image {
        payload: image::Payload,
        uri: String,
        method: String,
    },
}

impl TryFrom<Model> for serde_json::Value {
    type Error = anyhow::Error;
    fn try_from(value: Model) -> Result<Self> {
        match value {
            Model::Completion { payload, .. } => Ok(serde_json::to_value(payload)?),
            Model::Image { payload, .. } => Ok(serde_json::to_value(payload)?),
        }
    }
}

impl Model {
    fn uri(&self) -> String {
        match self {
            Model::Completion { uri, .. } => uri.clone(),
            Model::Image { uri, .. } => uri.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_oauth() {
        dotenv::dotenv().ok();

        let token = std::env::var("YC_IAM_OAUTH").unwrap();
        let _ = YCloudML::new().oauth(&token).await.unwrap();
    }
}
