use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs
    },
    Client,
};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static TOOL_USE_TEMPLATE: &str = r#"Available Tools: {{TOOLS}}
Return an empty string if no tools match the query. If a function tool matches, construct and return a JSON object in the format {"name": "functionName", "parameters": {"requiredFunctionParamKey": "requiredFunctionParamValue"}} using the appropriate tool and its parameters. Only return the object and limit the response to the JSON object without additional text."#;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
struct Tool {
    name: String,
    description: String,
    parameters: Parameters,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
struct Parameters {
    #[serde(rename = "type")]
    kind: String,
    properties: HashMap<String, Property>,
    required: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
struct Property {
    #[serde(rename = "type")]
    kind: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
struct ToolRequestFunction {
    name: String,
    parameters: HashMap<String, String>,
}

fn tool_calculator() -> Tool {
    Tool {
        name: "calculator".to_owned(),
        description: "Calculate the result of an equation".to_owned(),
        parameters: Parameters {
            kind: "object".to_owned(),
            properties: HashMap::from([(
                "equation".to_owned(),
                Property {
                    kind: "string".to_owned(),
                    description: "The equation to calculate.".to_owned(),
                },
            )]),
            required: vec!["equation".to_owned()],
        },
    }
}

fn tool_get_current_time() -> Tool {
    Tool {
        name: "get_current_time".to_owned(),
        description: "Get the current time in a more human-readable format.".to_owned(),
        parameters: Parameters {
            kind: "object".to_owned(),
            properties: HashMap::new(),
            required: Vec::new(),
        },
    }
}

fn tool_get_current_weather() -> Tool {
    Tool {
        name: "get_current_weather".to_owned(),
        description: "Get the current weather for a given city.".to_owned(),
        parameters: Parameters {
            kind: "object".to_owned(),
            properties: HashMap::from([(
                "city".to_owned(),
                Property {
                    kind: "string".to_owned(),
                    description: "The name of the city to get the weather for.".to_owned(),
                },
            )]),
            required: vec!["city".to_owned()],
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // This should come from env var outside the program
    std::env::set_var("RUST_LOG", "warn");

    // Setup tracing subscriber so that library can log the rate limited message
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let mut env = Environment::new();
    env.add_template_owned("hello", "Hello {{name}}!")?;
    env.add_template_owned("tool_use_template", TOOL_USE_TEMPLATE)?;

    let tools = vec![
        tool_calculator(),
        tool_get_current_time(),
        tool_get_current_weather(),
    ];

    let config = OpenAIConfig::new()
        .with_api_base("http://localhost:11434/v1")
        .with_api_key("EMPTY");
    let client = Client::with_config(config);

    let history = "USER: \"\"\"現在時間幾點\"\"\"";
    let query = format!("Query: History:\n{}\nQuery: 現在時間幾點", history);

    let tmpl = env.get_template("tool_use_template")?;

    let messages: Vec<_> = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(tmpl.render(context! {TOOLS => tools})?)
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(query)
            .build()?
            .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("gemma")
        .messages(messages.clone())
        .build()?;

    println!("Requests:");
    println!("{}", serde_json::to_string_pretty(&messages)?);

    let resp = client.chat().create(request).await?;
    let msg = resp.choices.first().unwrap().message.content.clone();
    if let Some(msg) = msg {
        println!("Response: \n{}\n", msg);
        let tool_request: ToolRequestFunction = serde_json::from_str(&msg)?;
        println!("Tool request: {:?}", tool_request);
    } else {
        println!("No response...");
    }

    Ok(())
}
