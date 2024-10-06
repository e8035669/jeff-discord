use std::error::Error;
use std::io::{stdout, Write};

use async_openai::config::OpenAIConfig;
use async_openai::types::ChatCompletionRequestUserMessageArgs;
use async_openai::{types::CreateChatCompletionRequestArgs, Client};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = OpenAIConfig::new()
        .with_api_base("http://localhost:8000/v1")
        .with_api_key("EMPTY");
    let client = Client::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        // .max_tokens(512u32)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content("Write a marketing blog praising and introducing Rust library async-openai in Chinese")
            // .content("寫一則部落格推銷和介紹Rust的函式庫async-openai")
            .build()?
            .into()])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    // From Rust docs on print: https://doc.rust-lang.org/std/macro.print.html
    //
    //  Note that stdout is frequently line-buffered by default so it may be necessary
    //  to use io::stdout().flush() to ensure the output is emitted immediately.
    //
    //  The print! macro will lock the standard output on each call.
    //  If you call print! within a hot loop, this behavior may be the bottleneck of the loop.
    //  To avoid this, lock stdout with io::stdout().lock():

    let mut lock = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(err) => {
                println!("");
                println!("error: {err}");
                break;
            }
        }
        stdout().flush()?;
    }

    Ok(())
}
