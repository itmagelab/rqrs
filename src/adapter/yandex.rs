//!```rust
//!use rqrs::adapter::yandex::{YandexGPT, RequestResult};
//!
//!async fn ask_for_yandex() -> rqrs::Result<()> {
//!    dotenv::dotenv().ok();
//!
//!    let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
//!    let rq = YandexGPT::new("b1gq7ngu525oc4nnpo0b", jwt, None)
//!        .system("You are financial bot")?
//!        .user("who are you?")?;
//!    let ans = rq.commit().await?;
//!    let mut req_result: RequestResult = serde_json::from_value(ans["result"].clone())?;
//!    let assistant = req_result.text();
//!    dbg!(&assistant);
//!    let rq = rq
//!        .assistant(&assistant)?
//!        .user("What u can to do?")?;
//!    let ans = rq.commit().await?;
//!    let mut req_result: RequestResult = serde_json::from_value(ans["result"].clone())?;
//!    let assistant = req_result.text();
//!    dbg!(assistant);
//!    Ok(())
//!}
//!````
use std::borrow::Cow;

use crate::api::Rq;

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Message, TextAdapter};

#[derive(Debug, Clone)]
pub struct YandexGPT<'a> {
    folder_id: Cow<'a, str>,
    jwt: Cow<'a, str>,
    session_id: String,
    messages: Vec<Message>,
}

impl TextAdapter<'_> for YandexGPT<'_> {
    fn messages(&mut self, message: super::Message) {
        self.messages.push(message);
    }
}

#[derive(Serialize, Deserialize)]
pub struct RequestResult {
    alternatives: Vec<Alternative>,
    #[serde(rename = "modelVersion")]
    model_version: String,
    usage: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct Alternative {
    message: Message,
    status: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Payload {
    #[serde(rename = "modelUri")]
    model_uri: String,
    #[serde(rename = "completionOptions")]
    completion_options: serde_json::Value,
    messages: Vec<Message>,
}

impl Payload {
    fn new(model_uri: String, messages: Vec<Message>) -> Self {
        Self {
            model_uri,
            completion_options: json!({
                "stream": false,
                "temperature": 0,
                "maxTokens": "2000",
                "reasoningOptions": {
                    "mode": "DISABLED"
                }
            }),
            messages,
        }
    }
}

impl<'a> YandexGPT<'a> {
    pub fn new<S>(folder_id: S, jwt: String, session_id: Option<String>) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let uuid = uuid::Uuid::new_v4().hyphenated().to_string();
        let session_id = session_id.unwrap_or(uuid);
        Self {
            folder_id: folder_id.into(),
            jwt: jwt.into(),
            session_id,
            messages: vec![],
        }
    }

    fn payload(&self) -> Result<serde_json::Value> {
        let payload = Payload::new(self.model_uri(), self.messages.clone());
        Ok(serde_json::to_value(&payload)?)
    }

    fn model_uri(&self) -> String {
        format!("gpt://{}/yandexgpt", self.folder_id)
    }

    fn rq(&self) -> Result<Rq> {
        let auth = format!("Bearer {}", self.jwt.trim());
        Rq::from_static("https://llm.api.cloud.yandex.net")?
            .uri("/foundationModels/v1/completion")
            .method("POST")?
            .add_header(b"x-data-logging-enabled", "false")?
            .add_header(b"X-Session-ID", self.session_id.clone())?
            .add_secret_header(b"authorization", auth)?
            .with_json()?
            .load_payload(self.payload()?)
    }

    pub async fn apply(&self) -> Result<serde_json::Value> {
        let rs = self.rq()?.apply().await?;
        if rs.data.is_null() {
            tracing::error!(?rs.raw);
            return Err(anyhow::anyhow!("Empty response: {:#?}", rs.raw));
        }
        Ok(rs.data)
    }
}

impl RequestResult {
    pub fn text(&mut self) -> String {
        self.alternatives.remove(0).message.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

    fn type_of<T>(_: &T) -> &'static str {
        type_name::<T>()
    }

    #[tokio::test]
    async fn test_ask_for_yandex() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let folder_id = std::env::var("YC_IAM_FOLDER").unwrap();
        let rq = YandexGPT::new(folder_id, jwt, None)
            .system("You are financial bot")
            .unwrap()
            .user("who are you?")
            .unwrap();
        let rs = rq.apply().await.unwrap();
        let mut req_result: RequestResult = serde_json::from_value(rs["result"].clone()).unwrap();
        let assistant = req_result.text();
        let rq = rq
            .assistant(&assistant)
            .unwrap()
            .user("What u can to do?")
            .unwrap();
        let rs = rq.apply().await.unwrap();
        let mut req_result: RequestResult = serde_json::from_value(rs["result"].clone()).unwrap();
        let assistant = req_result.text();
        assert_eq!(type_of(&assistant), "alloc::string::String");
    }
}
