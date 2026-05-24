# Voice Assistant

An offline voice assistant written in Rust. Listens continuously for a wake phrase, then transcribes and routes spoken commands to action handlers — currently creating tasks in [Vikunja](https://vikunja.io/).

## How It Works

1. Audio is captured from the default microphone at 16 kHz mono via [cpal](https://github.com/RustAudio/cpal).
2. Each audio chunk is fed to a local [Vosk](https://alphacephei.com/vosk/) speech recognition model, which emits partial and finalized transcripts.
3. The dispatcher listens for the wake phrase **"popcorn"**. Once heard, it enters an active state and passes subsequent finalized speech to command handlers.
4. The first handler that recognizes the utterance acts on it. Unrecognized commands are printed to stdout.
5. After 5 seconds of silence the dispatcher returns to idle and waits for the wake phrase again.

## Prerequisites

- Rust (edition 2024)
- A [Vosk model](https://alphacephei.com/vosk/models) — the small English model (`vosk-model-small-en-us`) works well
- A running [Vikunja](https://vikunja.io/) instance with an API token

## Setup

**1. Download a Vosk model**

```bash
wget https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.22.zip
unzip vosk-model-small-en-us-0.22.zip -d model
```

The extracted directory should be named `model/` in the project root, or pass its path as the first argument at runtime.

**2. Configure environment variables**

Create a `.env` file in the project root:

```env
VIKUNJA_URL=https://your-vikunja-instance.example.com
VIKUNJA_TOKEN=your_api_token_here
VIKUNJA_PROJECT_ID=1
```

**3. Build and run**

```bash
cargo run --release
# or with a custom model path:
cargo run --release -- /path/to/vosk-model
```

## Supported Commands

### Reminders

Say **"popcorn"** to activate, then speak a reminder. A schedule (period and/or time) is required.

| Pattern | Example |
|---|---|
| `remind me to X every day [at TIME]` | *"remind me to take my meds every day at 8 AM"* |
| `remind me to X every WEEKDAY [at TIME]` | *"remind me to call mum every Sunday"* |
| `remind me to X on WEEKDAY [at TIME]` | *"remind me to submit the report on Friday at noon"* |
| `remind me to X on the Nth of the month [at TIME]` | *"remind me to pay rent on the 1st of the month"* |

Times are understood as `at HH AM/PM`, `at HH:MM AM/PM`, `at noon`, or `at midnight`.

Numbers in transcripts are normalised automatically — saying *"remind me to exercise on the third of the month"* works the same as saying *"3rd"*.

## Project Structure

```
src/
  main.rs              Entry point — wires audio, recognizer, dispatcher, and handlers
  audio.rs             Microphone capture (cpal)
  recognizer.rs        Vosk speech-to-text wrapper
  dispatcher.rs        Wake-phrase detection and command routing state machine
  tokens/              Transcript parsers (period, time, remind trigger, normalizer)
  commands/            Command handlers (remind, print fallback)
  tasks/               TaskClient trait and Vikunja HTTP client
```

## Dependencies

| Crate | Purpose |
|---|---|
| `vosk` | Offline speech recognition |
| `cpal` | Cross-platform audio capture |
| `tokio` | Async runtime |
| `reqwest` | HTTP client for Vikunja API |
| `chrono` | Date/time arithmetic |
| `regex` | Transcript pattern matching |
| `dotenvy` | `.env` file loading |
