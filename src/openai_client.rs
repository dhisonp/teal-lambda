use async_openai::{types::CreateCompletionRequestArgs, Client};

pub(crate) async fn ask(prompt: &str) -> String {
    let client = Client::new();

    let req = CreateCompletionRequestArgs::default()
        .model("gpt-3.5-turbo-instruct")
        .prompt(prompt)
        .max_tokens(40_u32)
        .build()
        .unwrap(); // TODO: Handle errors here

    let res = client.completions().create(req).await.unwrap();
    return res.choices.first().unwrap().text.to_string();
}
