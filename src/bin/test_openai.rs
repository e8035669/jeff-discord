use anyhow::Result;
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole},
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = OpenAIClient::new_with_endpoint(
        String::from("http://localhost:8000/v1"),
        String::from("EMPTY"),
    );

    let req = ChatCompletionRequest::new(
        "mistral".to_string(),
        vec![ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text("What is bitcoin?".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
    );

    let result = client.chat_completion(req).await?;

    println!("{:?}", result);

    for i in &result.choices {
        let role = i.message.role.clone();
        let content = i.message.content.clone().unwrap_or_default();
        println!("{:?}: {}", role, content);
    }

    Ok(())
}
