use anyhow::Result;
use std::error::Error;

use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObject, FunctionObjectArgs
};
use async_openai::{types::CreateChatCompletionRequestArgs, Client};
use chrono::Utc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = OpenAIConfig::new()
        // .with_api_base("http://localhost:8000/v1")
        .with_api_base("http://localhost:11434/v1")
        .with_api_key("EMPTY");
    let client = Client::with_config(config);

    let mut current_history: Vec<ChatCompletionRequestMessage> =
        vec![ChatCompletionRequestUserMessageArgs::default()
            .content("現在是幾點了?")
            // .content("寫一則部落格推銷和介紹Rust的函式庫async-openai")
            .build()?
            .into()];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gemma")
        // .max_tokens(512u32)
        .messages(current_history.clone())
        .tools([get_current_utc_datetime_tool()?])
        .tool_choice("auto")
        .build()?;

    println!("Asking question");
    let mut ret = client.chat().create(request).await?;

    loop {
        let ret_msg = ret.choices.first().unwrap().message.clone();
        println!("Get response {:?}", ret_msg);

        if let Some(tool_calls) = ret_msg.tool_calls {
            let mut ret_tool_calls: Vec<ChatCompletionMessageToolCall> = Vec::new();
            let mut ret_tool_message: Vec<ChatCompletionRequestMessage> = Vec::new();

            for tool_call in tool_calls {
                let name = tool_call.function.name.clone();
                let _args = tool_call.function.arguments.clone();
                println!("Calling func {} {}", name, _args);
                let ret = call_fn(&name, &_args);
                if let Ok(ret) = ret {
                    ret_tool_calls.push(tool_call.clone());
                    let tool_message = ChatCompletionRequestToolMessageArgs::default()
                        .content(ret.to_string())
                        .tool_call_id(tool_call.id)
                        .build()?;
                    ret_tool_message.push(tool_message.into());
                }
            }

            let assist_message = ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(ret_tool_calls)
                .build()?;

            current_history.push(assist_message.into());
            current_history.extend(ret_tool_message);

            let request = CreateChatCompletionRequestArgs::default()
                .model("gemma")
                .messages(current_history.clone())
                .tools([get_current_utc_datetime_tool()?])
                .build()?;

            println!("Send tool message");
            ret = client.chat().create(request).await?;
        } else {
            let message = ret_msg.content;
            println!("Message: {:?}", message);
            break;
        }
    }

    Ok(())
}

fn call_fn(func_name: &str, _args: &str) -> Result<serde_json::Value> {
    match func_name {
        "get_current_utc_datetime" => Ok(get_current_utc_datetime()),
        _ => Ok(json!({})),
    }
}

fn get_current_utc_datetime_tool() -> Result<ChatCompletionTool, OpenAIError> {
    ChatCompletionToolArgs::default()
        .r#type(ChatCompletionToolType::Function)
        .function(get_current_utc_datetime_func()?)
        .build()
}

fn get_current_utc_datetime_func() -> Result<FunctionObject, OpenAIError> {
    FunctionObjectArgs::default()
        .name("get_current_utc_datetime")
        .description("Get current UTC date and time.")
        .parameters(json!({
            "type": "object",
            "properties": {},
            "required": [],
        }))
        .build()
}

fn get_current_utc_datetime() -> serde_json::Value {
    let datetime = Utc::now();

    let ret = json!({
        "current_utc_date": datetime.date_naive(),
        "current_utc_time": datetime.time(),
    });

    ret
}
