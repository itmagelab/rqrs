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
use serde::Serialize;

static URL: &str = "https://llm.api.cloud.yandex.net";

pub mod complition;
pub mod image;

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

    fn method(&self) -> String {
        match self {
            Model::Completion { method, .. } => method.clone(),
            Model::Image { method, .. } => method.clone(),
        }
    }
}
