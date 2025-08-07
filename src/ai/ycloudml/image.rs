//!```
//!use rqrs::prelude::*;
//!
//!#[tokio::test]
//!async fn run() {
//!    dotenv::dotenv().ok();
//!
//!    let jwt = std::env::var("YC_IAM_TOKEN")?;
//!    let folder_id = std::env::var("YC_IAM_FOLDER")?;
//!
//!    let rs = image::Payload::new(folder_id)
//!        .text("Generate an image of a grandfather learning Rust while drinking strong coffee")?
//!        .aspect_ratio(16, 9)
//!        .seed(0)
//!        .run(&jwt)
//!        .await?;
//!    let file = std::fs::File::create("/tmp/test.jpg")?;
//!    image::Payload::image(rs, file, jwt).await?;
//!}
//!```
use base64::prelude::*;
use std::{fs::File, io::Write};

use crate::{
    Result,
    api::{Rq, Rs},
};
use serde::{Deserialize, Serialize};

use super::{Model, URL};

pub static URI: &str = "/foundationModels/v1/imageGenerationAsync";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    #[serde(rename = "modelUri")]
    pub model_uri: String,
    #[serde(rename = "generationOptions")]
    pub generation_options: serde_json::Value,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    text: String,
    weight: i8,
}

#[derive(Deserialize, Serialize)]
pub struct Response {
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
    created_by: Option<String>,
    description: String,
    done: bool,
    id: String,
    metadata: Option<String>,
    #[serde(rename = "modifiedAt")]
    modified_at: Option<String>,
    response: Option<Object>,
}

#[derive(Deserialize, Serialize)]
pub struct Object {
    #[serde(rename = "@type")]
    r#type: String,
    image: String,
    #[serde(rename = "modelVersion")]
    model_version: String,
}

impl Default for Payload {
    fn default() -> Self {
        Payload {
            model_uri: String::new(),
            generation_options: serde_json::json!({
                "seed": "1863",
                "aspectRatio": {
                  "widthRatio": "16",
                  "heightRatio": "9"
                },
            }),
            messages: Vec::new(),
        }
    }
}

impl From<Payload> for Model {
    fn from(value: Payload) -> Self {
        Model::Image {
            payload: value,
            uri: URI.into(),
            method: "POST".into(),
        }
    }
}

impl Payload {
    pub fn new(folder_id: impl Into<String>) -> Self {
        Payload {
            model_uri: format!("art://{}/yandex-art/latest", folder_id.into().trim()),
            ..Default::default()
        }
    }

    pub fn seed(mut self, seed: u32) -> Self {
        self.generation_options["seed"] = serde_json::json!(seed);
        self
    }

    pub fn aspect_ratio(mut self, width_ratio: u32, height_ratio: u32) -> Self {
        self.generation_options["aspectRatio"] = serde_json::json!({
                  "widthRatio": width_ratio,
                  "heightRatio": height_ratio
        });
        self
    }

    pub fn text<S>(mut self, text: S) -> Result<Self>
    where
        S: Into<String>,
        Self: Sized,
    {
        self.messages.push(Message {
            text: text.into(),
            weight: 100,
        });
        Ok(self)
    }

    pub async fn image<S>(rs: Rs, mut file: File, jwt: S) -> Result<()>
    where
        S: Into<String>,
    {
        let rs: Response = serde_json::from_value(rs.data)?;

        let auth = format!("Bearer {}", jwt.into().trim());

        let rq = Rq::from_static(URL)?
            .uri(format!("/operations/{}", rs.id))
            .method("GET")?
            .add_secret_header(b"authorization", &auth)?
            .with_json()?;

        let max_attempts = 10;

        for _ in 1..=max_attempts {
            let rs = rq.apply().await?;
            let rs: Response = serde_json::from_value(rs.data)?;
            if rs.done {
                if let Some(response) = rs.response {
                    let buf = BASE64_STANDARD.decode(response.image)?;
                    file.write_all(&buf)?;
                    return Ok(());
                };
            };

            tracing::debug!("Waiting for image generation...");
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        }
        Err(anyhow::anyhow!(
            "Image generation did not complete after {} attempts",
            max_attempts
        ))
    }

    pub async fn run<S>(&self, jwt: S) -> Result<Rs>
    where
        S: Into<String>,
    {
        let auth = format!("Bearer {}", jwt.into().trim());
        let model: Model = self.clone().into();
        let uri = model.uri();
        let method = model.method();
        let payload = model.try_into()?;
        Rq::from_static(URL)?
            .uri(uri)
            .method(method)?
            .add_secret_header(b"authorization", auth)?
            .with_json()?
            .load_payload(payload)?
            .apply()
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_image() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let folder_id = std::env::var("YC_IAM_FOLDER").unwrap();

        let rs = image::Payload::new(folder_id)
            .text("Generate an image of a grandfather learning Rust while drinking strong coffee")
            .unwrap()
            .aspect_ratio(16, 9)
            .seed(0)
            .run(&jwt)
            .await
            .unwrap();
        let file = std::fs::File::create("/tmp/test.jpg").unwrap();
        image::Payload::image(rs, file, jwt).await.unwrap();
    }
}
