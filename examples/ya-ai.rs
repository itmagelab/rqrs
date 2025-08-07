use rqrs::prelude::*;

async fn run() -> Result<()> {
    let mut folder_id = String::new();
    println!("Input YAC folder ID (ex, b1gq7ngu525oc4nnpo0b):");
    std::io::stdin().read_line(&mut folder_id)?;
    let model_uri = format!("gpt://{}/yandexgpt", folder_id.trim());

    let system = "I am a bot";
    let prompt = "Who i am?";
    let payload = serde_json::json!({
        "modelUri": model_uri,
        "completionOptions": {
        "stream": false,
        "temperature": 0.6,
        "maxTokens": "2000",
        "reasoningOptions": {
            "mode": "DISABLED"
        }
        },
        "messages": [
            {"role": "system", "text": system},
            {"role": "user", "text": prompt},
        ],
    });
    let mut jwt = String::new();
    println!("Input JWT (ex, yc iam create-token):");
    std::io::stdin().read_line(&mut jwt)?;
    let auth = format!("Bearer {}", jwt.trim());

    let uuid = uuid::Uuid::new_v4().hyphenated().to_string();
    let rq = Rq::from_static("https://llm.api.cloud.yandex.net")?
        .uri("/foundationModels/v1/completion")
        .method("POST")?
        .add_header(b"x-data-logging-enabled", "false")?
        .add_header(b"X-Session-ID", &uuid)?
        .add_secret_header(b"authorization", &auth)?
        .with_json()?
        .load_payload(payload)?;
    let rs = rq.apply().await?;
    dbg!(&rs);
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
