use std::env;
use std::io::{Read};
use std::fs::File;
use reqwest::Client;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let server_url = env::var("SERVER_URL").expect("SERVER_URL must be set");

    let mut file = File::open("input.json")?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let client = Client::new();
    let res = client.post(&server_url)
        .body(input)
        .send()
        .await?;

    println!("Response: {:?}", res.text().await?);
    Ok(())
}