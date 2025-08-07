use rqrs::prelude::*;

async fn run() -> Result<()> {
    dotenv::dotenv().ok();

    let jwt = std::env::var("YC_IAM_TOKEN")?;
    let folder_id = std::env::var("YC_IAM_FOLDER")?;
    let session_id = uuid::Uuid::new_v4().hyphenated().to_string();

    let model = Completion::new(folder_id)
        .system("You are financial bot")?
        .user("who are you?")?;
    let rs = model.run(&jwt, &session_id).await?;
    dbg!(&rs);

    let assistant = Completion::assistant_text_first(rs)?;
    let model = model.assistant(&assistant)?.user("What u can to do?")?;
    let rs = model.run(&jwt, &session_id).await?;
    dbg!(&rs);
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
