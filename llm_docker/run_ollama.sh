#!/bin/bash

docker run --rm -e OLLAMA_DEBUG="1" -v ${PWD}/ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama

