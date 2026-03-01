# Profiler

A local-first personality intelligence and communication coaching terminal application.

## Overview

Profiler is a privacy-first TUI (Terminal User Interface) application that helps you:

- **Create personality profiles** from text samples, writing, or notes
- **Analyze draft messages** for tone, directness, and compatibility
- **Get communication recommendations** tailored to specific people
- **Compare profiles** to identify alignment and friction points
- **Rewrite messages** in different styles (concise, warm, executive, etc.)

All processing happens locally using Ollama - no cloud APIs, no data leaves your machine.

## Features

### Profile Creation
- Infer communication patterns from writing samples
- DISC-style personality dimensions (Dominance, Influence, Steadiness, Conscientiousness)
- Directness, pace, formality, risk tolerance analysis
- Motivators and stress triggers identification
- Evidence-based insights with confidence scores

### Message Coaching
- Analyze draft emails/messages against target profiles
- Score clarity, tone fit, directness, detail level, warmth
- Identify risky phrases that may not land well
- Get rewrite suggestions in multiple styles
- Explain why changes improve communication

### Profile Comparison
- Compatibility scoring between two people
- Alignment areas and friction points
- Tactical collaboration recommendations
- Meeting strategy suggestions

### Rule-Based Fallback
- Works without LLM when Ollama is unavailable
- Pattern-based analysis of text characteristics
- Reduced confidence but still useful insights

## Requirements

- **Rust** 1.70+ (for building)
- **Ollama** running locally
- Recommended models:
  - Chat: `qwen2.5:7b`, `gemma2:9b`, or `llama3.2:3b`
  - Embeddings: `nomic-embed-text` or `mxbai-embed-large`

## Installation

### 1. Install Ollama

```bash
# Linux/macOS
curl -fsSL https://ollama.com/install.sh | sh

# Or download from https://ollama.com
```

### 2. Pull Required Models

```bash
# Chat model (choose one)
ollama pull qwen2.5:7b
# or
ollama pull gemma2:9b

# Embedding model
ollama pull nomic-embed-text
```

### 3. Build Profiler

```bash
cd profiler
cargo build --release
```

### 4. Install (Optional)

```bash
cargo install --path .
```

## Configuration

On first run, Profiler creates a default config at `~/.config/profiler/config.toml`.

See `config.example.toml` for all options.

Key settings:
- `ollama_host`: Ollama server URL (default: `http://localhost:11434`)
- `chat_model`: Model for inference (default: `qwen2.5:7b`)
- `embedding_model`: Model for embeddings (default: `nomic-embed-text`)
- `database_path`: SQLite database location
- `privacy.store_evidence`: Whether to save evidence snippets
- `privacy.store_draft_text`: Whether to save full draft text

## Usage

### Starting the App

```bash
# If installed
profiler

# Or run directly
cargo run --release
```

### Navigation

The app is keyboard-driven:

**Global Shortcuts:**
- `Ctrl+Q` - Quit application
- `ESC` - Return to dashboard
- Arrow keys - Navigate lists

**Dashboard:**
- `N` - Create new profile
- `C` - Coach a message
- `P` - View saved profiles
- `M` - Compare profiles
- `S` - Settings
- `L` - View logs
- `Q` - Quit

**Profile Creation:**
1. Select profile type (Text Inference recommended)
2. Enter profile name
3. Paste text samples (emails, messages, writing)
4. `Ctrl+S` - Process and create profile

**Message Coaching:**
1. Select target profile
2. Paste your draft message
3. `Ctrl+S` - Analyze
4. Review scores and risky phrases
5. `R` - Request rewrites

## Architecture

```
profiler/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Module exports
│   ├── app.rs               # Main app logic & event loop
│   ├── config.rs            # Configuration management
│   ├── models.rs            # Core data structures
│   ├── storage.rs           # SQLite persistence
│   ├── ollama.rs            # Ollama HTTP client
│   ├── inference/
│   │   ├── mod.rs
│   │   ├── pipeline.rs      # Inference orchestration
│   │   ├── prompts.rs       # LLM prompts
│   │   └── fallback.rs      # Rule-based analysis
│   └── ui/
│       ├── mod.rs
│       ├── terminal.rs      # Terminal setup/teardown
│       ├── components.rs    # Reusable UI components
│       └── screens/         # All TUI screens
│           ├── dashboard.rs
│           ├── create_profile.rs
│           ├── profile_view.rs
│           ├── coach.rs
│           ├── compare.rs
│           ├── saved_profiles.rs
│           ├── settings.rs
│           └── logs.rs
├── Cargo.toml
├── config.example.toml
└── README.md
```

## Data Storage

All data is stored locally in SQLite:

- **Profiles**: Complete personality profiles with evidence
- **Analyses**: Draft analysis results and rewrites
- **Compatibility Reports**: Profile comparison results
- **Logs**: LLM request history for debugging

Default location: `~/.local/share/profiler/profiler.db`

## Privacy

Profiler is designed with privacy as a core principle:

- ✅ All processing happens locally via Ollama
- ✅ No cloud APIs or external network calls
- ✅ Data never leaves your machine
- ✅ Configurable evidence/draft storage
- ✅ SQLite database under your control

## Example Workflow

### 1. Create a Profile for a Colleague

```
1. Press 'N' on dashboard
2. Select "Text Inference"
3. Enter name: "Sarah (PM)"
4. Paste recent emails/Slack messages from Sarah
5. Ctrl+S to process
6. Review profile: DISC style, traits, do/don't lists
```

### 2. Coach a Message to Sarah

```
1. Press 'C' on dashboard
2. Select Sarah's profile
3. Paste your draft email
4. Ctrl+S to analyze
5. Review scores (80% tone fit, 65% directness fit)
6. Check risky phrases
7. Press 'R' for rewrite suggestions
8. Choose "Warm" variant
```

### 3. Compare Two Team Members

```
1. Press 'M' on dashboard
2. Select two profiles
3. View compatibility score
4. Read alignment areas (shared values)
5. Review friction points + mitigations
6. Apply collaboration recommendations
```

## Troubleshooting

### "Ollama connection failed"

- Ensure Ollama is running: `ollama serve`
- Check host config: `ollama_host` in config.toml
- Verify models are pulled: `ollama list`

### "Failed to parse LLM response"

- LLM may not support JSON mode
- Try a different model (qwen2.5:7b recommended)
- Check logs screen (`L`) for error details
- Fallback mode will activate automatically

### Slow inference

- Use smaller models: `llama3.2:3b`
- Enable GPU acceleration in Ollama
- Reduce draft/text length

### Database errors

- Check permissions on `~/.local/share/profiler/`
- Delete and recreate: `rm ~/.local/share/profiler/profiler.db`

## Development

### Running Tests

```bash
cargo test
```

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run
```

Logs are written to `~/.local/share/profiler/logs/profiler.log`

### Building for Release

```bash
cargo build --release
# Binary at: target/release/profiler
```

## Future Improvements

Potential enhancements:

- **Self-assessment questionnaire** for building your own profile
- **File import** for analyzing documents (.txt, .md, .pdf)
- **Semantic search** using embeddings (find similar profiles)
- **Export** profiles/analyses to Markdown/JSON
- **Tags and filtering** for profile organization
- **Meeting prep mode** - prepare for specific interactions
- **Relationship tracking** - how communication changes over time
- **Custom DISC models** - configure personality frameworks
- **Batch analysis** - process multiple messages at once

## Use Cases

**Sales & Outreach:**
- Analyze prospect communication style
- Tailor pitch to their preferences
- Optimize cold email effectiveness

**Team Management:**
- Understand direct reports better
- Adapt leadership style per person
- Reduce team friction

**Collaboration:**
- Bridge communication gaps
- Prepare for difficult conversations
- Navigate diverse work styles

**Personal Development:**
- Understand your own communication patterns
- Identify blind spots
- Practice adapting your style

## Contributing

This is a personal project, but feedback and suggestions are welcome!

## License

MIT License - see LICENSE file for details.

## Credits

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Ollama](https://ollama.com) - Local LLM runtime
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [SQLite](https://www.sqlite.org/) via [rusqlite](https://github.com/rusqlite/rusqlite)

Inspired by personality frameworks like DISC, but adapted for practical communication coaching.
