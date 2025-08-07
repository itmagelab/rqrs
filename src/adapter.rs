//!```rust
//!use rqrs::adapter::{YandexGPT, RequestResult};
//!
//!async fn ask_for_yandex() -> rqrs::Result<()> {
//!    dotenv::dotenv().ok();
//!
//!    let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
//!    let rq = YandexGPT::new("b1gq7ngu525oc4nnpo0b", &jwt)
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

use super::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct YandexGPT<'a> {
    folder_id: Cow<'a, str>,
    jwt: Cow<'a, str>,
    uuid: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct RequestResult {
    alternatives: Vec<Alternatives>,
    #[serde(rename = "modelVersion")]
    model_version: String,
    usage: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct Alternatives {
    message: Message,
    status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    text: String,
}

// serde_json::json!({
//     "modelUri": self.model_uri(),
//     "completionOptions": {
//         "stream": false,
//         "temperature": 0.6,
//         "maxTokens": "2000",
//         "reasoningOptions": {
//             "mode": "DISABLED"
//         }
//     },
//     "messages": self.messages,
// })
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
                "temperature": 0.6,
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
    pub fn new<S>(folder_id: S, jwt: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let uuid = uuid::Uuid::new_v4().hyphenated().to_string();
        Self {
            folder_id: folder_id.into(),
            jwt: jwt.into(),
            uuid,
            messages: vec![],
        }
    }

    fn payload(&self) -> Result<serde_json::Value> {
        let payload = Payload::new(self.model_uri(), self.messages.clone());
        Ok(serde_json::to_value(&payload)?)
    }

    pub fn jwt(&self) -> Result<String> {
        let mut jwt = String::new();
        println!("Input JWT (ex, yc iam create-token):");
        std::io::stdin().read_line(&mut jwt)?;
        Ok(jwt)
    }

    pub fn assistant<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        self.messages.push(Message {
            role: "assistant".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    pub fn user<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        self.messages.push(Message {
            role: "user".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    pub fn system<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        self.messages.push(Message {
            role: "system".into(),
            text: message.into().to_string(),
        });
        Ok(self)
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
            .add_header(b"X-Session-ID", self.uuid.clone())?
            .add_secret_header(b"authorization", auth)?
            .with_json()?
            .load_payload(self.payload()?)
    }

    pub async fn commit(&self) -> Result<serde_json::Value> {
        let rs = self.rq()?.apply().await?;
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

    #[tokio::test]
    async fn test_ask_for_yandex() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let rq = YandexGPT::new("b1gq7ngu525oc4nnpo0b", &jwt)
            .system("You are financial bot")
            .unwrap()
            .user("who are you?")
            .unwrap();
        let ans = rq.commit().await.unwrap();
        let mut req_result: RequestResult = serde_json::from_value(ans["result"].clone()).unwrap();
        let assistant = req_result.text();
        dbg!(&assistant);
        let rq = rq
            .assistant(&assistant)
            .unwrap()
            .user("What u can to do?")
            .unwrap();
        let ans = rq.commit().await.unwrap();
        let mut req_result: RequestResult = serde_json::from_value(ans["result"].clone()).unwrap();
        let assistant = req_result.text();
        dbg!(assistant);
    }
}
