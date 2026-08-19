# BUDBUDDHI (Direct Human Intent)

> A token-minimal AI coding engine built in Rust. BUDBUDDHI translates raw human intent into precise code modifications using a hybrid local/cloud inference architecture with strict token budgets, security sandboxing, and automatic verification.

[![CI](https://github.com/InfidelRahul/buddhi-dev/actions/workflows/ci.yml/badge.svg)](https://github.com/InfidelRahul/buddhi-dev/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

---

## Table of Contents

- [Philosophy](#philosophy)
- [Architecture](#architecture)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Model Support](#model-support)
- [Development](#development)
- [Project Structure](#project-structure)
- [Roadmap](#roadmap)
- [License](#license)

---

## Philosophy

BUDBUDDHI is built on three core principles:

1. **Token Minimalism** — Every prompt, context window, and cloud call is budget-constrained. No wasted tokens.
2. **Security First** — All file operations pass through `PathGuard`. All patches go through `PatchSafety` with dry-run validation.
3. **Hybrid Intelligence** — A local GGUF model handles intent parsing and prompt enhancement. The cloud model handles heavy code generation. This keeps costs low and privacy high.

---

## Architecture

```
+---------------------------------------------------------------------+
|                         buddhi-cli (Entry Point)                        |
+---------------------------------------------------------------------+
|                              buddhi-engine                              |
|              (Agent Loop - Prompt Builder - Orchestration)           |
+------------------+------------------+-------------------------------+
|   buddhi-brain      |    buddhi-llm       |         buddhi-tools             |
| (Local Intent    | (Cloud LLM       |  (expand - get_snippet -      |
|  Optimization)   |  Streaming)      |   replace - patch safety)     |
+------------------+------------------+-------------------------------+
|                          buddhi-inference                               |
|              +------------------+------------------+                 |
|              |   GGUF Engine    | Safetensors      |                 |
|              |  (llama-cpp-2)   | Engine           |                 |
|              |  CPU - Q4_K_M    | (candle-core)    |                 |
|              +------------------+------------------+                 |
+---------------------------------------------------------------------+
|  buddhi-token  |  buddhi-security  |  buddhi-context  |  buddhi-verify          |
|  (Budgets)  |  (PathGuard)   |  (Tree-sitter)|  (Error Compressor)  |
+---------------------------------------------------------------------+
|              buddhi-core - buddhi-config - buddhi-rules - buddhi-memory          |
+---------------------------------------------------------------------+
```

---

## Features

### Hybrid Inference
- **Local Brain**: Quantized GGUF models (via `llama-cpp-2`) for instant intent parsing, prompt enhancement, and log analysis
- **Cloud Brain**: OpenAI/Anthropic streaming with JSON tool-call interception
- **Dual Engine**: Automatic routing between `.gguf` and `.safetensors` models

### Security & Safety
- **PathGuard**: Prevents path traversal, hidden file access, and out-of-bounds writes
- **PatchSafety**: Dry-run mode, unified diff generation, and AST validation before applying changes
- **Secret Scanning**: Detects API keys and credentials in generated code

### Token Management
- **Strict Budgets**: Per-turn and per-task token limits with real-time enforcement
- **Stream Tracking**: Budget validation during streaming responses
- **Context Caching**: Efficient snippet storage to avoid re-transmission

### Developer Tools
- **expand**: Retrieve full file contents from partial references
- **get_snippet**: Extract specific line ranges or symbol definitions
- **replace**: Apply targeted code modifications with diff verification

### Verification Pipeline
- **Auto-Compile**: Runs `cargo check` / `clippy` after every patch
- **Error Compression**: Transforms verbose compiler errors into token-minimal diagnostics
- **Test Runner**: Validates changes don't break existing tests

### Rules & Memory
- **Global Rules**: Universal constraints (no `unwrap()`, JSON-only output)
- **Project Rules**: Per-repository YAML rules with AST node matching
- **Lesson Storage**: Persistent memory of human corrections and style preferences

---

## Prerequisites

| Requirement | Version | Purpose |
|---|---|---|
| **Rust** | 1.75+ | Core language and build system |
| **C/C++ Compiler** | GCC 12+ / Clang 15+ | Required for `llama-cpp-2` (GGUF engine) |
| **Make** | 4.0+ | Build automation |
| **Git** | 2.30+ | Source control |

### Linux (Debian/Kali/Ubuntu)
```bash
sudo apt update && sudo apt install -y build-essential git make
```

### macOS
```bash
xcode-select --install
```

### Windows
```bash
# Install Visual Studio Build Tools with C++ workload
# Install Rust via rustup
winget install Microsoft.VisualStudio.2022.BuildTools
```

---

## Installation

```bash
# Clone the repository
git clone https://github.com/InfidelRahul/buddhi-dev.git
cd buddhi-dev

# Build in release mode (recommended)
make release

# Or build in debug mode for development
make build

# Verify installation
cargo run -p buddhi-cli -- --help
```

---

## Configuration

BUDBUDDHI uses a YAML configuration file. Copy the example and customize:

```bash
cp config.example.yaml config.yaml
```

### Configuration Structure

```yaml
# Local Brain (GGUF model for intent parsing)
local_brain:
  enabled: true
  model_path: "models/qwen2.5-coder-1.5b-q4_k_m.gguf"
  tokenizer_path: "models/tokenizer.json"
  max_output_tokens: 120
  timeout_ms: 800

# Cloud Brain (heavy code generation)
cloud:
  provider: "openai"
  model: "gpt-4o"
  api_key_env: "OPENAI_API_KEY"  # Environment variable name
  max_output_tokens: 1024

# Token Budgets
budget:
  max_tokens_per_turn: 4096
  max_total_tokens_per_task: 16384

# Security
security:
  sandbox_enabled: true
  secret_scanning_enabled: true
```

### Environment Variables

```bash
export OPENAI_API_KEY="sk-your-key-here"
export ANTHROPIC_API_KEY="sk-ant-your-key-here"  # Optional
```

---

## Usage

### Basic Task Execution

```bash
# Run a coding task
cargo run -p buddhi-cli -- --task "Fix the null pointer error in src/parser.rs"

# Run with custom config
cargo run -p buddhi-cli -- --config custom.yaml --task "Add unit tests for the tokenizer"
```

### Model Loading

BUDBUDDHI automatically detects model format:

```bash
# GGUF model (fast CPU inference, quantized)
model_path: "models/qwen2.5-coder-1.5b-q4_k_m.gguf"

# Safetensors directory (full precision, HuggingFace standard)
model_path: "models/llama-3-8b-instruct/"
```

### Supported Task Types

| Task Type | Description | Example |
|---|---|---|
| `BugFix` | Fix compilation/runtime errors | "Fix the borrow checker error" |
| `Feature` | Implement new functionality | "Add retry logic to the API client" |
| `Refactor` | Restructure existing code | "Extract this into a trait" |
| `Test` | Add or modify tests | "Write integration tests for the parser" |
| `Unknown` | Fallback heuristic routing | — |

---

## Model Support

### GGUF Models (Recommended for Local Brain)

| Model | Size | Use Case |
|---|---|---|
| Qwen2.5-Coder-1.5B-Q4_K_M | ~1GB | Prompt enhancement, log analysis |
| Llama-3.2-3B-Q4_K_M | ~2GB | Code understanding, error detection |
| Mistral-7B-Q4_K_M | ~4GB | Complex refactoring suggestions |

### Safetensors Models (Full Precision)

| Model | Size | Use Case |
|---|---|---|
| Llama-3-8B-Instruct | ~16GB | Production code generation |
| Qwen2.5-Coder-7B | ~14GB | Multi-file refactoring |
| CodeLlama-13B | ~26GB | Enterprise codebases |

### Downloading Models

```bash
# Using huggingface-cli
pip install huggingface-hub
huggingface-cli download Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF \
  qwen2.5-coder-1.5b-instruct-q4_k_m.gguf \
  --local-dir models/

# Or download directly from HuggingFace Hub
# https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF
```

---

## Development

### Build Commands

```bash
make build      # Debug build
make release    # Release build (optimized, LTO, stripped)
make test       # Run all tests
make lint       # Run clippy with strict warnings
make fmt        # Format code
make bench      # Run benchmarks
make clean      # Clean build artifacts
```

### Adding a New Tool

1. Create `crates/buddhi-tools/src/my_tool.rs`
2. Implement the `Tool` trait
3. Register in `crates/buddhi-tools/src/registry.rs`
4. Add tests in `crates/buddhi-tools/tests/`

### Adding a New Engine

1. Create `crates/buddhi-inference/src/my_engine.rs`
2. Implement the `InferenceEngine` trait
3. Add routing logic in `crates/buddhi-inference/src/loader.rs`

### Running Specific Tests

```bash
# Test a specific crate
cargo test -p buddhi-inference

# Test with output
cargo test -p buddhi-security -- --nocapture

# Run benchmarks
cargo bench -p buddhi-token
```

---

## Project Structure

```
buddhi-dev/
├── Cargo.toml                    # Workspace root
├── Makefile                      # Build automation
├── config.example.yaml           # Configuration template
├── .buddhi/
│   └── rules.example.yaml        # Project rules template
├── .github/
│   └── workflows/
│       ├── ci.yml                # Continuous Integration
│       └── release.yml           # Release automation
├── crates/
│   ├── buddhi-core/                 # Domain models, errors, session
│   ├── buddhi-config/               # YAML config loading
│   ├── buddhi-cli/                  # Command-line interface
│   ├── buddhi-testsupport/          # Test utilities
│   ├── buddhi-heuristics/           # Task parsing & detection
│   ├── buddhi-brain/                # Local intent optimization
│   ├── buddhi-token/                # Token budgets & counting
│   ├── buddhi-llm/                  # Cloud LLM providers
│   ├── buddhi-tools/                # Code manipulation tools
│   ├── buddhi-security/             # PathGuard & sandboxing
│   ├── buddhi-context/              # Tree-sitter AST parsing
│   ├── buddhi-verify/               # Compilation & test verification
│   ├── buddhi-rules/                # Global & project rules
│   ├── buddhi-memory/               # Persistent lesson storage
│   ├── buddhi-engine/               # Agent loop orchestration
│   └── buddhi-inference/            # Dual engine (GGUF + Safetensors)
└── models/                       # Downloaded model files (gitignored)
```

---

## Roadmap

### v0.2.0 (Current)
- [x] Dual Engine Architecture (GGUF + Safetensors)
- [x] Token budget enforcement
- [x] PathGuard security layer
- [x] Patch safety with dry-run
- [ ] Dynamic model configuration (`config.json` parsing)
- [ ] Close the agent loop (tool execution → verification → retry)
- [ ] GPU acceleration (CUDA/Metal)

### v0.3.0 (Planned)
- [ ] Multi-language Tree-sitter support (Python, TypeScript, Go)
- [ ] Real-time token streaming to terminal with ANSI coloring
- [ ] Context window management with automatic truncation
- [ ] Cloud provider abstraction (Anthropic, Google, Azure)

### v0.4.0 (Future)
- [ ] Plugin system for custom tools
- [ ] IDE integration (LSP server)
- [ ] Team memory sharing
- [ ] Model fine-tuning pipeline

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Ensure all tests pass (`make test`)
4. Ensure clippy is clean (`make lint`)
5. Commit with conventional messages (`feat:`, `fix:`, `docs:`)
6. Push and open a Pull Request

### Commit Convention

```
feat(phase-N): description
fix(phase-N): description
docs(phase-N): description
test(phase-N): description
refactor(phase-N): description
```

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) — GGUF inference backend
- [candle](https://github.com/huggingface/candle) — Pure Rust ML framework
- [tree-sitter](https://github.com/tree-sitter/tree-sitter) — Incremental AST parsing
- [HuggingFace](https://huggingface.co) — Model hosting and tokenizer ecosystem

---

**Built with ❤️ by the BUDBUDDHI community. Token-efficient. Security-first. Rust-native.**
