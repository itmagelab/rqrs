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
//!     let model = Completion::new(folder_id)
//!         .system("You are financial bot")?
//!         .user("who are you?")?;
//!     let rs = model.run(&jwt, &session_id).await?;
//!
//!     let assistant = Completion::assistant_text_first(rs)?;
//!     let model = model
//!         .assistant(&assistant)?
//!         .user("What u can to do?")?;
//!     model.run(&jwt, &session_id).await?;
//!     Ok(())
//! }
//! ```

use crate::{
    Result,
    api::{Rq, Rs},
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct YCloudML {
    payload: serde_json::Value,
}

impl YCloudML {
    pub fn new(payload: serde_json::Value) -> Self {
        Self { payload }
    }

    pub async fn apply<S>(self, jwt: S, session_id: S) -> Result<Rs>
    where
        S: Into<String>,
    {
        let auth = format!("Bearer {}", jwt.into().trim());
        Rq::from_static("https://llm.api.cloud.yandex.net")?
            .uri("/foundationModels/v1/completion")
            .method("POST")?
            .add_header(b"x-data-logging-enabled", "false")?
            .add_header(b"X-Session-ID", session_id)?
            .add_secret_header(b"authorization", auth)?
            .with_json()?
            .load_payload(self.payload)?
            .apply()
            .await
    }

    pub fn rerun(&self) -> Rs {
        todo!()
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum Model {
    Completion(Completion),
}

impl From<Completion> for Model {
    fn from(value: Completion) -> Self {
        Self::Completion(value.clone())
    }
}

impl TryFrom<Model> for serde_json::Value {
    type Error = anyhow::Error;
    fn try_from(value: Model) -> Result<Self> {
        match value {
            Model::Completion(model) => {
                let json = serde_json::to_value(model)?;
                Ok(json)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Completion {
    #[serde(rename = "modelUri")]
    model_uri: String,
    #[serde(rename = "completionOptions")]
    completion_options: serde_json::Value,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct Response {
    alternatives: Vec<Alternative>,
    #[serde(rename = "modelVersion")]
    model_version: String,
    usage: serde_json::Value,
}

impl Response {
    pub fn model_version(&self) -> &str {
        &self.model_version
    }

    pub fn usage(&self) -> &serde_json::Value {
        &self.usage
    }

    pub fn first_alternatives(self) -> Result<String> {
        let alt = self
            .alternatives
            .into_iter()
            .find(|alt| alt.status.as_str() == "ALTERNATIVE_STATUS_FINAL")
            .ok_or_else(|| anyhow::anyhow!("No final alternative found"))?;

        Ok(alt.message.text)
    }
}

#[derive(Deserialize)]
pub struct Alternative {
    message: Message,
    status: String,
}

impl Alternative {
    pub fn status(&self) -> &str {
        &self.status
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    role: String,
    text: String,
}

impl Completion {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Completion {
            model_uri: format!("gpt://{}/yandexgpt", folder_id.into()),
            completion_options: serde_json::json!({
                "stream": false,
                "temperature": 0,
                "maxTokens": "2000",
                "reasoningOptions": {
                    "mode": "DISABLED"
                }
            }),
            messages: Vec::new(),
        }
    }

    pub fn assistant_text_first(rs: Rs) -> Result<String> {
        let result = rs
            .data
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("No result"))?
            .clone();
        let rs: Response = serde_json::from_value(result)?;
        rs.first_alternatives()
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.completion_options["maxTokens"] = serde_json::json!(max_tokens);
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.completion_options["temperature"] = serde_json::json!(temperature);
        self
    }

    pub fn assistant<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "assistant".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    pub fn user<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "user".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    pub fn system<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "system".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    pub async fn run<S>(&self, jwt: S, session_id: S) -> Result<Rs>
    where
        S: Into<String>,
    {
        let payload = serde_json::to_value(self)?;
        let rs = YCloudML::new(payload).apply(jwt, session_id).await?;
        Ok(rs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_some() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let folder_id = std::env::var("YC_IAM_FOLDER").unwrap();
        let session_id = uuid::Uuid::new_v4().hyphenated().to_string();

        let model = Completion::new(folder_id)
            .system("You are financial bot")
            .unwrap()
            .user("who are you?")
            .unwrap();
        let rs = model.run(&jwt, &session_id).await.unwrap();
        let assistant = Completion::assistant_text_first(rs).unwrap();
        let model = model
            .assistant(&assistant)
            .unwrap()
            .user("What u can to do?")
            .unwrap();
        let _ = model.run(&jwt, &session_id).await.unwrap();
    }
}
