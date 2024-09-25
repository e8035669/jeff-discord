
from openai import OpenAI
# Set OpenAI's API key and API base to use vLLM's API server.
openai_api_key = "EMPTY"
openai_api_base = "http://localhost:8000/v1"

client = OpenAI(
    api_key=openai_api_key,
    base_url=openai_api_base,
)

chat_response = client.chat.completions.create(
    model="mistral",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "說個笑話吧"},
    ]
)
print("Chat response:", chat_response)
print(chat_response.choices[0].message.content)
