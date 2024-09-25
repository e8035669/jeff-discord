# docker run -it --rm -v ${PWD}/huggingface:/root/.cache/huggingface -p 8000:8000 --ipc=host vllm-cpu-env --model "yentinglin/Llama-3-Taiwan-8B-Instruct"

# docker run -it --rm --gpus all -v ${PWD}/huggingface:/root/.cache/huggingface -p 8000:80 --ipc host ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.0 -i plain -m yentinglin/Llama-3-Taiwan-8B-Instruct -a llama

docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.0 gguf -t /data/Llama-3-Taiwan-8B-Instruct -m /data/Llama-3-Taiwan-8B-Instruct -f Llama-3-Taiwan-8B-Instruct-rc2-Q4_K.gguf
