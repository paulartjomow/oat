use seahorse::Command;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tokio;

pub fn generate_command() -> Command {
    Command::new("generate")
        .usage("oat generate [subcommand]")
        .command(dalle_command())
}

fn dalle_command() -> Command {
    Command::new("dalle")
        .usage(r#"oat generate dalle "[prompt]""#)
        .action(|c| {
            let prompt: String = c.args.join(" ");
            tokio::spawn(async move {
                dalle_action(prompt).await;
            });
        })
}

#[derive(Serialize)]
struct DalleRequest {
    model: String,
    prompt: String,
    n: u32,
    size: String,
}

#[derive(Deserialize)]
struct DalleResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    url: String,
}

async fn dalle_action(prompt: String) {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let client = Client::new();
    let request_body = DalleRequest {
        model: "dall-e-3".to_string(),
        prompt: prompt.clone(),
        n: 1,
        size: "1024x1024".to_string(),
    };

    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    if response.status().is_success() {
        let dalle_response: DalleResponse = response.json().await.expect("Failed to parse response");
        if let Some(image_data) = dalle_response.data.first() {
            println!("{}", image_data.url);
        } else {
            eprintln!("No image data found in the response");
        }
    } else {
        eprintln!("Failed to generate image: {}", response.status());
    }
} 