use serde::{Serialize,Deserialize};

#[derive(Serialize,Debug)]
struct SummaryRequest {
    model: String,
    temperature: usize,
    max_tokens: usize,
    prompt: String,
}

#[derive(Deserialize,Debug)]
struct SummaryResponse {
    id: String,
    object: String,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Deserialize,Debug)]
pub struct Choice {
    text: String,
    index: usize,
    finish_reason: String,
}

async pub fn try_completion() -> eyre::Result<(), reqwest::Error> {
    let api_token = "";
    let req = SummaryRequest {
        model: "text-davinci-003".to_string(),
        temperature: 0,
        max_tokens: 10,
        prompt: "say this is a test".to_string(),
    };
    let client = Client::new();
    let resp = client
        .post("https://api.openai.com/v1/completions")
        .json(&req)
        .bearer_auth(api_token)
        .send()
        .await?
        .json::<SummaryResponse>()
        .await?;
    println!("{:?}", resp);
    Ok(())
}
