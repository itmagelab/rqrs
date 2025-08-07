use rqrs::prelude::*;

async fn run() -> Result<()> {
    dotenv::dotenv().ok();

    let jwt = std::env::var("YC_IAM_TOKEN")?;
    let folder_id = std::env::var("YC_IAM_FOLDER")?;
    let session_id = uuid::Uuid::new_v4().hyphenated().to_string();

    let payload = complition::Payload::new(folder_id)
        .system("You are financial bot")?
        .user("who are you?")?;
    let rs = payload.run(&jwt, &session_id).await.unwrap();
    dbg!(&rs);

    let assistant = complition::Payload::assistant_text_first(rs)?;
    let payload = payload.assistant(&assistant)?.user("What u can to do?")?;
    let rs = payload.run(&jwt, &session_id).await.unwrap();

    dbg!(&rs);
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
