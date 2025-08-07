use serde::{Deserialize, Serialize};

use super::{Model, URL};
use crate::Result;

pub static URI: &str = "/foundationModels/v1/completion";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    #[serde(rename = "modelUri")]
    pub model_uri: String,
    #[serde(rename = "completionOptions")]
    pub completion_options: serde_json::Value,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    alternatives: Vec<Alternative>,
    #[serde(rename = "modelVersion")]
    model_version: String,
    usage: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct Alternative {
    message: Message,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    role: String,
    text: String,
}

impl From<Payload> for Model {
    fn from(value: Payload) -> Self {
        Model::Completion {
            payload: value,
            uri: URI.into(),
            method: "POST".into(),
        }
    }
}

impl Alternative {
    pub fn status(&self) -> &str {
        &self.status
    }
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

impl Default for Payload {
    fn default() -> Self {
        Payload {
            model_uri: String::new(),
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
}

impl Payload {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Payload {
            model_uri: format!("gpt://{}/yandexgpt", folder_id.into()),
            ..Default::default()
        }
    }

    pub fn assistant_text_first(rs: Response) -> Result<String> {
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

    pub fn assistant<S>(mut self, text: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "assistant".into(),
            text: text.into(),
        });
        Ok(self)
    }

    pub fn user<S>(mut self, text: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "user".into(),
            text: text.into(),
        });
        Ok(self)
    }

    pub fn system<S>(mut self, text: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            role: "system".into(),
            text: text.into(),
        });
        Ok(self)
    }

    pub fn save_messages(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self.messages.clone())?)
    }

    pub fn load_messages(mut self, messages: serde_json::Value) -> Result<Self> {
        let messages: Vec<Message> = serde_json::from_value(messages)?;
        self.messages.extend(messages);
        Ok(self)
    }

    pub async fn run<S>(&self, jwt: S, session_id: S) -> Result<Response>
    where
        S: Into<String>,
    {
        let model: Model = self.clone().into();
        let uri = model.uri();
        let client = reqwest::Client::new();
        let rs: serde_json::Value = client
            .post(format!("{URL}{uri}"))
            .header("x-data-logging-enabled", "false")
            .header("X-Session-ID", session_id.into())
            .bearer_auth(jwt.into().trim())
            .json(&self)
            .send()
            .await?
            .json()
            .await?;
        let rs: Response = serde_json::from_value(rs["result"].clone())?;
        Ok(rs)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_text() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let folder_id = std::env::var("YC_IAM_FOLDER").unwrap();
        let session_id = uuid::Uuid::new_v4().hyphenated().to_string();

        let payload = complition::Payload::new(folder_id)
            .system("You are financial bot")
            .unwrap()
            .user("who are you?")
            .unwrap();
        let rs = payload.run(&jwt, &session_id).await.unwrap();
        let assistant = complition::Payload::assistant_text_first(rs).unwrap();

        let payload = payload
            .assistant(&assistant)
            .unwrap()
            .user("What u can to do?")
            .unwrap();
        let _ = payload.run(jwt, session_id).await.unwrap();

        let history = payload.save_messages().unwrap();
        payload.load_messages(history).unwrap();
    }
}
