use rqrs::prelude::*;

async fn run() -> Result<()> {
    let rq = Rq::from_static("https://reqres.in")?
        .uri("/api/users")
        .method("GET")?
        .add_secret_header(b"x-api-key", "reqres-free-v1")?
        .add_header(b"Content-Type", "application/json")?
        .add_params(vec![("page", "2")]);
    let rs = rq.apply().await?;
    let json = serde_json::to_string_pretty(&rs.data)?;
    println!("{json}");
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
