#!/bin/bash

docker run --rm -p 3000:8080 \
    -e WEBUI_AUTH=false \
    -e OLLAMA_BASE_URL=http://192.168.17.20:11434 \
    -v ${PWD}/open-webui:/app/backend/data \
    ghcr.io/open-webui/open-webui:main
#
# docker run --rm -p 3000:8080 \
#     -e WEBUI_AUTH=false \
#     -e OPENAI_API_BASE_URL=http://192.168.17.20:8000/v1 \
#     -v ${PWD}/open-webui:/app/backend/data \
#     ghcr.io/open-webui/open-webui:main
