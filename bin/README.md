# Bundled local LLM

`laverna` ships with a local llama.cpp engine so the **L.ai** personality
agent (`laverna_companion` MCP tool, `--features llm`) runs **offline, on
device** — no network, no API key, no external trust.

## What's here

- `bin/llama/llama` — prebuilt llama.cpp (b10034, aarch64) multicall binary +
  its shared libraries. Auto-detected by the engine; no install needed.
- `bin/models/` — **empty by default**. Drop a single `.gguf` file here.

## Add a model

The engine needs a GGUF weights file to actually *speak*. Any instruct-tuned
GGUF works; for a phone, a tiny model (~50–100 MB, e.g. Qwen2.5-0.5B Q4) is
plenty for short, practical answers. The quickest path is
`./scripts/get-model.sh`, which downloads a pinned default into `bin/models/`.

```bash
# example (run where you have Hugging Face access, then move the file here)
curl -L -o bin/models/model.gguf \
  https://huggingface.co/<owner>/<model>/resolve/main/<model>-q4_k_m.gguf

# or just copy any .gguf you already have:
cp ~/Downloads/some-model.Q4_K_M.gguf bin/models/
```

The first `*.gguf` found in `bin/models/` is used automatically.

## Overrides (env)

| Var | Default | Purpose |
|-----|---------|---------|
| `LAVERNA_LLAMA_BIN` | `bin/llama/llama` | Path to the llama binary |
| `LAVERNA_LLAMA_MODEL` | first `bin/models/*.gguf` | Path to the GGUF weights |

Without a model present, the companion tool returns the verified engine
result directly (plain text) and a short note telling you to add a `.gguf` —
it never fabricates.
