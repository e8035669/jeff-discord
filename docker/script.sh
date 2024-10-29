# docker run -it --rm -v ${PWD}/huggingface:/root/.cache/huggingface -p 8000:8000 --ipc=host vllm-cpu-env --model "yentinglin/Llama-3-Taiwan-8B-Instruct"

# docker run -it --rm --gpus all -v ${PWD}/huggingface:/root/.cache/huggingface -p 8000:80 --ipc host ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.0 -i plain -m yentinglin/Llama-3-Taiwan-8B-Instruct -a llama

# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.0 gguf -t /data/Llama-3-Taiwan-8B-Instruct -m /data/Llama-3-Taiwan-8B-Instruct -f Llama-3-Taiwan-8B-Instruct-rc2-Q2_K.gguf

# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host vllm-openvino-env

# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host --init --entrypoint bash ghcr.io/ggerganov/llama.cpp:full

# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host --init ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.1 gguf -t /data/gemma-2-2b-it-GGUF -m /data/gemma-2-2b-it-GGUF -f ggml-model-Q8_0.gguf

# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host --init ghcr.io/ericlbuehler/mistral.rs:cpu-0.3.1 -i  plain -m EricB/gemma-2-2b-it-UQFF --from-uqff gemma2-2b-instruct-q8_0.uqff

docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host --init --gpus all ghcr.io/e8035669/llama.cpp:full-cuda -s -m /data/gemma-2-2b-it-GGUF/ggml-model-Q8_0.gguf --host 0.0.0.0 --port 80 --n-gpu-layers 25 -t 8
# docker run -it --rm -v ${PWD}/data:/data -p 8000:80 --ipc host --init ghcr.io/ggerganov/llama.cpp:full -s -m /data/gemma-2-2b-it-GGUF/ggml-model-Q4_K.gguf --host 0.0.0.0 --port 80



