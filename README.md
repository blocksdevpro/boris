<div align="center">

# 🤖 Boris

**A locally-running, offline-first voice assistant with a big personality.**

Boris is a Rust learning project — a voice assistant that listens for a wake word, transcribes your speech with Whisper, sends it to an LLM, and speaks back using a local TTS model. No cloud. No telemetry. Just Boris being Boris.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![OpenRouter](https://img.shields.io/badge/LLM-OpenRouter-blue)](https://openrouter.ai)

</div>

---

## What is Boris?

Boris is a 24-year-old AI voice assistant. He's German, enthusiastic, overconfident, and hilariously dumb.

> *"Ja ja, I totally know what I'm doing, broda. The CPU is overheating because... uh... the electrons are tired. Not my fault."*

Built as a personal Rust learning project to explore:
- Real-time audio capture and processing
- On-device wake word detection
- Whisper speech-to-text transcription
- OpenAI-compatible LLM APIs (via OpenRouter)
- Local neural TTS with Piper

---

## Architecture

```
Microphone
    │
    ▼
AudioStream  (cpal — raw PCM frames)
    │
    ▼
AudioAdapter  (resampler → sliding buffer)
    │
    ├──[idle]──▶  WakeWord  (livekit-wakeword ONNX + EMA smoothing)
    │                │
    │           [detected]
    │                │
    └──[recording]──▶  VAD  (earshot — voice activity detection)
                         │
                    [silence]
                         │
                         ▼
                     Whisper  (whisper-rs — local STT)
                         │
                         ▼
                     OpenAI  (reqwest — OpenRouter / any OpenAI-compat API)
                         │
                         ▼
                       TTS  (piper-rs — local neural voice)
                         │
                         ▼
                     Playback  (cpal — speaker output)
```

---

## Prerequisites

| Requirement | Notes |
|---|---|
| [Rust](https://rustup.rs/) (stable 2024) | Install via `rustup` |
| LLVM / Clang | Required by `whisper-rs` for C++ FFI bindings |
| ONNX Runtime | Required by `livekit-wakeword` — see below |
| Model files | See **Downloading Models** below |

### Windows (LLVM)
```powershell
winget install LLVM.LLVM
```

### macOS (LLVM)
```bash
brew install llvm
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

---

## Downloading Models

Boris needs three sets of model files placed in the `models/` directory. They are not committed to this repository due to size.

### 1. Wake Word Model (`models/livekit/boris.onnx`)

This is a custom ONNX model trained on the wake word **"Boris"**.
It is bundled with this repo as it is small (~160 KB).
If it's missing, you can retrain one using [openWakeWord](https://github.com/dscripka/openWakeWord) or [livekit/rust-sdks](https://github.com/livekit/rust-sdks).

### 2. Whisper Speech-to-Text (`models/whisper/`)

Download from [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp/tree/main):

```bash
# Tiny English model (~75 MB) — fast, good enough for commands
curl -L -o models/whisper/ggml-tiny.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin
```

Other available sizes: `base`, `small`, `medium`, `large` — update `WHISPER_MODEL_PATH` in `src/constants.rs` to match.

### 3. Piper TTS Voice (`models/piper/nmale/`)

Download from [rhasspy/piper-voices](https://huggingface.co/rhasspy/piper-voices):

```bash
# English male voice (~60 MB)
curl -L -o models/piper/nmale/nmale.onnx \
  https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/ryan/high/en_US-ryan-high.onnx

curl -L -o models/piper/nmale/nmale.onnx.json \
  https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/ryan/high/en_US-ryan-high.onnx.json
```

> Any Piper-compatible voice works — just update the paths in `src/constants.rs`.

---

## Setup

**1. Clone the repo**
```bash
git clone https://github.com/blocksdevpro/boris
cd boris
```

**2. Download models** (see above)

**3. Configure environment**
```bash
cp .env.example .env
```
Then edit `.env` and fill in your API key.

**4. Get an API key**

Boris uses [OpenRouter](https://openrouter.ai) by default — it's free to sign up and gives you access to many models including Gemini and GPT-4o.

---

## Running

```bash
cargo run --release
```

Boris will boot, load all models, and then say **"listening..."** — say **"Boris"** to wake him up.

---

## Environment Variables

All config is via environment variables (or a `.env` file).

| Variable | Required | Default | Description |
|---|---|---|---|
| `BORIS_API_KEY` | ✅ Yes | — | Your OpenRouter API key |
| `BORIS_MODEL` | No | `google/gemini-3-flash-preview` | LLM model slug — any OpenRouter model works |
| `BORIS_BASE_URL` | No | `https://openrouter.ai/api/v1` | API base URL — use for local models (Ollama, LM Studio) |

> **Local models**: Set `BORIS_BASE_URL=http://localhost:11434/v1` and `BORIS_MODEL=llama3` for fully offline operation.

---

## Project Structure

```
boris/
├── src/
│   ├── main.rs              # Entry point — wires up audio pipeline
│   ├── config.rs            # Env-var config loader
│   ├── constants.rs         # Model paths, audio/VAD/wakeword tuning params
│   ├── logger.rs            # env_logger setup
│   ├── utils.rs             # f32↔i16 helpers, WAV writer
│   ├── audio/
│   │   ├── adapter.rs       # Resampler + buffer + event dispatch
│   │   ├── boris.rs         # Main state machine (Idle → Listening → Recording)
│   │   ├── buffer.rs        # SlidingBuffer and RecordBuffer
│   │   ├── filters.rs       # High-pass filter, pre-emphasis, RMS normalize
│   │   ├── playback.rs      # cpal output stream
│   │   ├── resampler.rs     # rubato-based audio resampler
│   │   ├── stream.rs        # cpal input stream
│   │   └── whisper.rs       # whisper-rs transcription wrapper
│   └── services/
│       ├── openai.rs        # OpenAI-compatible HTTP client
│       └── tts.rs           # piper-rs TTS wrapper
├── models/                  # Model files (not committed — see above)
│   ├── livekit/             # boris.onnx — wake word model
│   ├── whisper/             # ggml-*.bin — Whisper STT model
│   └── piper/nmale/         # nmale.onnx + .json — TTS voice
├── .env.example             # Copy to .env and fill in your key
├── Cargo.toml
└── LICENSE
```

---

## Tuning

Boris is designed to be easily tunable without recompiling by editing `src/constants.rs`:

| Constant | Purpose | Default |
|---|---|---|
| `WAKEWORD_THRESHOLD` | How confident the model must be to trigger | `0.2` |
| `VAD_SPEECH_THRESHOLD` | Min confidence to mark audio as speech | `0.65` |
| `VAD_SILENCE_THRESHOLD` | Max confidence to count as silence | `0.3` |
| `VAD_SILENCE_DURATION` | How long silence must last before stopping recording | `1000ms` |
| `WAKEWORD_INTERVAL` | How often wake word inference runs | `80ms` |

The EMA (Exponential Moving Average) smoothing in `boris.rs` accumulates weak per-frame wake word scores — useful for far-field / quiet microphones where a single frame never crosses the threshold alone.

---

## What I Learned

This project was built to learn Rust. Key things explored:

- **Ownership & borrowing** — threading audio data across multiple owned structs without `Arc<Mutex<>>` everywhere, using channels (`mpsc`) instead
- **FFI** — calling C/C++ libraries (`whisper-rs`, `piper-rs`) safely from Rust
- **Real-time audio** — working with `cpal` for zero-latency streaming, understanding buffer sizes and callback timing
- **State machines** — modeling `Idle → Listening → Recording` transitions cleanly
- **Channel-based concurrency** — the `BorisEvent` enum as a message bus between the audio adapter thread and the main processing loop

---

## Roadmap

- [ ] GUI process context — let Boris see what apps are running
- [ ] Screenshot capture — multimodal prompts with screen context
- [ ] Conversation memory — multi-turn dialogue
- [ ] Custom wake word training guide
- [ ] Linux / macOS testing

---

## License

[MIT](./LICENSE) — do whatever you want, just don't blame Boris when he gives you wrong advice.
