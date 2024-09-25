use std::io::{self, BufRead, Write};

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

    let stdout = io::stdout();
    let stdin = io::stdin();

    let mut history: Vec<ChatCompletionMessage> = Vec::new();
    history.push(ChatCompletionMessage {
        role: MessageRole::system,
        content: Content::Text(
            "You are a useful assistant, you are familar in cooking, making food".to_string(),
        ),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });

    loop {
        print!(">>> ");
        stdout.lock().flush()?;
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        if input.trim().is_empty() {
            continue;
        }

        let msg = ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(input.clone()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        };

        history.push(msg);

        let req = ChatCompletionRequest::new("mistral".to_string(), history.clone());
        let result = client.chat_completion(req).await?;

        for i in &result.choices {
            let role = i.message.role.clone();
            let content = i.message.content.clone().unwrap_or_default();
            println!("{:?}: {}", role, content);
        }

        for i in &result.choices {
            let resp_msg = &i.message;
            let resp_msg = ChatCompletionMessage {
                role: resp_msg.role.clone(),
                content: Content::Text(resp_msg.content.clone().unwrap_or_default()),
                name: resp_msg.name.clone(),
                tool_call_id: None,
                tool_calls: resp_msg.tool_calls.clone(),
            };
            history.push(resp_msg);
        }
    }

    // Ok(())
}
