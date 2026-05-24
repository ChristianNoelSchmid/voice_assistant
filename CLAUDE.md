# Voice Assistant

Rust voice assistant that listens for a wake phrase ("popcorn"), then routes spoken commands to handlers. Uses Vosk for offline speech recognition and Vikunja as the task backend.

## Build & Run

```bash
cargo run -- model/   # model/ is the Vosk model directory (default if omitted)
cargo check           # fast type-check without linking
cargo build           # full build
```

Requires a `.env` file (or environment variables):

| Variable | Description |
|---|---|
| `VIKUNJA_URL` | Base URL of your Vikunja instance |
| `VIKUNJA_TOKEN` | Vikunja API token |
| `VIKUNJA_PROJECT_ID` | Numeric ID of the project to create tasks in |

## Architecture

```
audio.rs          Captures 16 kHz mono i16 PCM from the default input device (cpal)
recognizer.rs     Wraps Vosk; emits Partial / Finalized RecognitionEvents
dispatcher.rs     State machine: Idle → Active on wake phrase; routes text to handlers
tokens/
  normalize.rs    Converts spoken numbers/ordinals to digits; collapses Vosk's "a m" → "am"
  period.rs       Parses scheduling periods ("every Monday", "on the 3rd of the month")
  time.rs         Parses clock times ("at 9:30 PM", "at noon")
  remind.rs       Detects the trigger word "remind"
commands/
  mod.rs          CommandHandler (typed) + DynCommandHandler (object-safe) traits
  remind.rs       Handles "remind me to X [period] [time]" → creates a Vikunja task
  print.rs        Fallback handler — prints any unmatched command (always last)
tasks/
  mod.rs          TaskClient trait + TaskClientError
  vikunja.rs      HTTP client for the Vikunja REST API
```

## Key Conventions

- **Token byte ranges**: `Token::parse` returns the matched value *and* its byte range in the input. Callers collect these ranges to subtract consumed spans when reconstructing the remaining content text.
- **Handler ordering**: handlers are tried in order; first match wins. `PrintHandler` always matches, so it must be last in the list.
- **Normalization before parsing**: all transcript text is passed through `tokens::normalize` before any token or command parsing. Downstream code can assume digits for numbers and "am"/"pm" without spaces.
- **Wake phrase on partials**: the dispatcher checks for the wake phrase on `Partial` events so activation happens as soon as the word is spoken, before Vosk finalizes the utterance.
- **Vikunja task creation uses PUT**, not POST.
- **Monthly recurrence encoding**: `repeat_mode=1` tells Vikunja to treat `repeat_after` as a month count rather than seconds.
