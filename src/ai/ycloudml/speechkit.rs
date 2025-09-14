use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Result;

pub static URI: &str = "/speech/v1/stt:recognize";
pub static URL: &str = "https://stt.api.cloud.yandex.net";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
    file: PathBuf,
    folder_id: String,
    lang: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub result: String,
}

impl Payload {
    pub fn new<S>(folder_id: S, lang: S) -> Self
    where
        S: Into<String>,
    {
        Payload {
            file: PathBuf::new(),
            folder_id: folder_id.into(),
            lang: lang.into(),
        }
    }

    pub fn file<P>(mut self, file: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        self.file = file.into();
        Ok(self)
    }

    pub async fn run<S>(&self, jwt: S) -> Result<Response>
    where
        S: Into<String>,
    {
        let client = reqwest::Client::new();
        let file = std::fs::read(&self.file)?;
        let rs: Response = client
            .post(format!("{URL}{URI}"))
            .bearer_auth(jwt.into().trim())
            .body(file)
            .query(&[
                ("folderId", self.folder_id.clone()),
                ("lang", self.lang.clone()),
            ])
            .send()
            .await?
            .json()
            .await?;
        Ok(rs)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn test_speech() {
        dotenv::dotenv().ok();

        let jwt = std::env::var("YC_IAM_TOKEN").unwrap();
        let folder_id = std::env::var("YC_IAM_FOLDER").unwrap();

        let payload = speechkit::Payload::new(folder_id.as_str(), "ru-RU")
            .file("/tmp/speech.ogg")
            .unwrap();
        let _ = payload.run(&jwt).await.unwrap().result;
    }
}
