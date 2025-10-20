use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionFunctionCall, ChatCompletionFunctionsArgs,
        ChatCompletionRequestFunctionMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // This should come from env var outside the program
    std::env::set_var("RUST_LOG", "warn");

    // Setup tracing subscriber so that library can log the rate limited message
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let config = OpenAIConfig::new()
        // .with_api_base("http://localhost:8000/v1")
        .with_api_base("http://localhost:11434/v1")
        .with_api_key("EMPTY");
    let client = Client::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("gemma")
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content("What's the current weather like in Boston?")
            .build()?
            .into()])
        .functions([ChatCompletionFunctionsArgs::default()
            .name("get_current_weather")
            .description("Get the current weather in a given location")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA",
                    },
                    "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
                },
                "required": ["location"],
            }))
            .build()?])
        .function_call(ChatCompletionFunctionCall::Function {
            name: "get_current_weather".to_string(),
        })
        .build()?;

    let response_message = client
        .chat()
        .create(request)
        .await?
        .choices
        .first()
        .unwrap()
        .message
        .clone();

    if let Some(tool_calls) = response_message.tool_calls {
        let mut available_functions: HashMap<&str, fn(&str, &str) -> serde_json::Value> =
            HashMap::new();
        available_functions.insert("get_current_weather", get_current_weather);

        let mut function_messages = Vec::new();

        for tool_call in tool_calls {
            let function_call = tool_call.function.clone();
            let function_name = function_call.name;
            let function_args: serde_json::Value = function_call.arguments.parse().unwrap();

            let location = function_args["location"].as_str().unwrap();
            let unit = "fahrenheit";
            let function = available_functions.get(function_name.as_str()).unwrap();
            let function_response = function(location, unit);

            let function_message = ChatCompletionRequestFunctionMessageArgs::default()
                .content(function_response.to_string())
                .name(function_name)
                .build()?;
            function_messages.push(function_message);
        }

        let mut message = vec![ChatCompletionRequestUserMessageArgs::default()
            .content("What's the weather like in Boston?")
            .build()?
            .into()];
        message.extend(function_messages.into_iter().map(|fm| fm.into()));

        println!("{}", serde_json::to_string(&message).unwrap());

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("gemma")
            .messages(message)
            .build()?;

        let response = client.chat().create(request).await?;

        println!("\nResponse:\n");
        for choice in response.choices {
            println!(
                "{}: Role: {}  Content: {:?}",
                choice.index, choice.message.role, choice.message.content
            );
        }
    } else {
        println!("Response: {:?}", response_message);
    }

    Ok(())
}

fn get_current_weather(location: &str, unit: &str) -> serde_json::Value {
    let weather_info = json!({
        "location": location,
        "temperature": "72",
        "unit": unit,
        "forecast": ["sunny", "windy"]
    });

    weather_info
}
