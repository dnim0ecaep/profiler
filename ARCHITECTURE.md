# Architecture Overview

## Design Principles

1. **Separation of Concerns**: Domain logic, storage, inference, and UI are clearly separated
2. **Privacy-First**: All processing happens locally, no external dependencies
3. **Fallback Gracefully**: Rule-based analysis when LLM is unavailable
4. **Testable**: Pure functions where possible, dependency injection for I/O
5. **Type Safety**: Rich domain models prevent invalid states

## Module Structure

### Core Modules

#### `models.rs`
Defines all domain entities:
- `PersonProfile` - Complete personality profile with traits and preferences
- `TraitScores` - DISC dimensions and communication metrics
- `DraftAnalysis` - Message scoring and feedback
- `CompatibilityReport` - Profile comparison results
- `EvidenceSnippet` - Supporting evidence for inferences

All models are:
- Serializable for persistence
- Cloneable for thread safety
- Documented with field semantics

#### `config.rs`
Configuration management:
- Loads from `~/.config/profiler/config.toml`
- Falls back to sensible defaults
- Validates settings on load
- Provides save() for persistence

#### `storage.rs`
SQLite persistence layer:
- Profile CRUD operations
- Draft analysis history
- Compatibility report caching
- LLM request logging
- Tag-based search

Abstracts all SQL details behind a clean API.

### Inference System

#### `inference/prompts.rs`
LLM prompt templates:
- System prompts for each task type
- User prompt formatters
- JSON schema definitions
- Examples and instructions

Prompts are designed for:
- Reliability (structured output)
- Clarity (explicit instructions)
- Flexibility (model-agnostic)

#### `inference/fallback.rs`
Rule-based analysis:
- Text metric extraction (sentence length, hedging, urgency)
- Heuristic trait scoring
- Pattern matching for communication style
- Lower confidence but functional without LLM

Provides baseline functionality when:
- Ollama is offline
- Model doesn't support JSON mode
- LLM response is malformed

#### `inference/pipeline.rs`
Orchestration layer:
- Profile inference from text
- Draft analysis and scoring
- Message rewriting
- Profile comparison
- Error handling with fallback
- LLM request logging

Coordinates between Ollama client, storage, and fallback analyzer.

### Ollama Integration

#### `ollama.rs`
HTTP client for Ollama:
- Chat completions with JSON mode
- Embeddings (future: semantic search)
- Model listing
- Connection health checks
- Timeout handling

Abstracts Ollama's REST API behind typed Rust interface.

### User Interface

#### `ui/terminal.rs`
Terminal lifecycle:
- Initialize raw mode
- Setup alternate screen
- Restore on exit
- Error handling

#### `ui/components.rs`
Reusable UI widgets:
- Trait bars (gauges for 0-1 scores)
- Score coloring (green/yellow/red)
- Help bar formatting

#### `ui/screens/`
Individual screens:
- `dashboard.rs` - Home screen with recent profiles
- `create_profile.rs` - Multi-step profile creation
- `profile_view.rs` - Detailed profile display
- `coach.rs` - Draft analysis and rewriting
- `compare.rs` - Profile compatibility
- `saved_profiles.rs` - Profile list and search
- `settings.rs` - Configuration editing
- `logs.rs` - LLM request history

Each screen is a pure render function taking state as input.

#### `app.rs`
Application state machine:
- Event loop (keyboard input)
- Screen navigation
- State management
- Async operations (inference)
- Data loading/refreshing

Owns all mutable state and coordinates screen transitions.

## Data Flow

### Profile Creation Flow

```
User Input (text) 
  → App::handle_create_profile_keys() 
  → InferencePipeline::infer_profile_from_text()
  → Try: LLM inference
  → Fallback: Rule-based analysis
  → Storage::save_profile()
  → App::load_profiles()
  → Navigate to ProfileView screen
```

### Message Coaching Flow

```
User Input (draft + target profile)
  → App::handle_coach_keys()
  → InferencePipeline::analyze_draft()
  → Ollama::chat() with analysis prompt
  → Parse JSON response
  → DraftAnalysis model
  → Render scores and risky phrases
  → User requests rewrite
  → InferencePipeline::rewrite_draft()
  → Display variants
```

### Profile Comparison Flow

```
Select two profiles
  → App navigates to Compare screen
  → InferencePipeline::compare_profiles()
  → Ollama::chat() with comparison prompt
  → CompatibilityReport model
  → Storage::save_compatibility_report()
  → Render alignment/friction/recommendations
```

## Error Handling

### Strategy
1. **Results, not panics**: All fallible operations return `Result<T, E>`
2. **Context enrichment**: Use `anyhow::Context` to add error context
3. **Graceful degradation**: Fall back to rule-based analysis
4. **User-friendly messages**: Map technical errors to helpful guidance
5. **Logging**: Trace errors for debugging

### Error Types
- **Storage errors**: Database connection, SQL errors
- **Ollama errors**: Connection failed, model not found, malformed response
- **Parsing errors**: JSON deserialization, invalid data
- **IO errors**: Config read/write, log file creation

All errors bubble up to the event loop and are displayed to the user.

## Async Architecture

Uses Tokio for async runtime:
- Main event loop is async
- Ollama HTTP calls are async
- UI rendering is synchronous (Ratatui requirement)

Async work happens in:
- `InferencePipeline` methods (I/O bound)
- `OllamaClient` HTTP requests
- `App::process_profile_creation()`

## Testing Strategy

### Unit Tests
- `models.rs`: Domain logic, serialization
- `config.rs`: Default values, save/load
- `inference/fallback.rs`: Text metrics, trait scoring
- `storage.rs`: CRUD operations

### Integration Tests
- Profile creation end-to-end
- Storage + models integration
- Fallback analyzer on sample text

### Manual Testing
- TUI screens (visual)
- Keyboard navigation
- Ollama integration
- Long-running operations

## Performance Considerations

### Database
- Indexed queries (name, created_at, tags)
- Prepared statements
- Batch operations where possible

### LLM Inference
- Timeout after 120s
- Model selection (smaller = faster)
- JSON mode reduces parsing overhead

### UI Rendering
- 100ms poll interval (responsive)
- Efficient layout calculations
- Minimal redraws

## Security & Privacy

### Data Storage
- Local SQLite only
- No encryption (user's filesystem security)
- Configurable evidence retention

### Network
- Only connects to local Ollama
- No external APIs
- No telemetry or analytics

### Input Sanitization
- SQL: Parameterized queries
- JSON: Strict deserialization
- No code execution from user input

## Future Extensibility

### Easy Additions
- New screens (add to `ui/screens/`)
- New inference tasks (add prompts + pipeline methods)
- New storage queries (add to `storage.rs`)
- New keyboard shortcuts (add to `app.rs` handlers)

### Harder Changes
- Different backend (replace `ollama.rs`)
- Different UI framework (replace `ui/` module)
- Different storage (replace `storage.rs`)

The modular design makes these changes localized.

## Dependencies Rationale

- **ratatui**: Best-in-class TUI framework for Rust
- **crossterm**: Cross-platform terminal manipulation
- **tokio**: De facto standard for async Rust
- **reqwest**: Ergonomic HTTP client
- **rusqlite**: Embedded SQLite, no external dependencies
- **serde**: Serialization ecosystem
- **anyhow/thiserror**: Modern error handling
- **tracing**: Structured logging

All dependencies are:
- Well-maintained
- Widely used
- Pure Rust (no C dependencies except SQLite)
- Compatible licenses (MIT/Apache-2.0)
