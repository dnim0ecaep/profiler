# Storage Architecture

## Overview

The profiler now uses a **folder-based storage system** with human-readable TOML and JSON files instead of a SQLite database. This makes profiles easily editable, portable, and git-friendly.

## Directory Structure

```
./data/
├── profiles/
│   ├── john-doe/
│   │   ├── profile.toml        # Basic metadata (name, ID, dates, tags)
│   │   ├── traits.toml         # DISC personality traits and scores
│   │   ├── preferences.toml    # Communication preferences
│   │   ├── motivators.toml     # What motivates this person
│   │   ├── stress.toml         # Stress triggers
│   │   ├── analysis.toml       # Strengths and blind spots
│   │   ├── evidence.json       # Supporting evidence snippets
│   │   └── sources/            # User can manually add files here
│   │       ├── email-samples.txt
│   │       ├── chat-logs.md
│   │       └── notes.txt
│   └── jane-smith/
│       ├── profile.toml
│       ├── traits.toml
│       └── sources/
└── logs/
    ├── llm-requests.jsonl      # LLM request logs (one per line)
    ├── draft-analyses/         # Message coaching analyses
    │   └── analysis-123.json
    └── compatibility-reports/  # Profile comparison reports
        └── report-456.json
```

## Profile Folder Structure

Each profile is stored in its own folder named after the profile ID (slugified name). The profile is split across multiple TOML files for easy editing:

### profile.toml
```toml
id = "john-doe"
name = "John Doe"
profile_type = "TextInference"
confidence = 0.85
tags = ["colleague", "engineering"]
created_at = "2026-02-28T08:00:00Z"
updated_at = "2026-02-28T08:30:00Z"
```

### traits.toml
```toml
primary_style = "Dominance"
secondary_style = "Conscientiousness"
directness = 0.8
pace = 0.7
people_vs_task = 0.4
detail_orientation = 0.6
risk_tolerance = 0.7
formality = 0.5
```

### preferences.toml
```toml
preferred_tone = ["direct", "professional"]
message_length = "Brief"
response_urgency = "High"
meeting_style = ["structured", "agenda-driven"]
do_list = ["Be direct", "Lead with conclusions"]
dont_list = ["Don't be vague", "Don't waste time with small talk"]
```

### sources/ Directory

The `sources/` folder is where **you can manually add files** for re-analysis:

- Drop in `.txt`, `.md`, `.json` files
- Email archives
- Chat transcripts
- Meeting notes
- Writing samples

The app can scan these files and re-run inference to update the profile.

## Benefits

✅ **Human-readable** - Edit profiles with any text editor  
✅ **Git-friendly** - Track profile changes over time  
✅ **Portable** - Just copy folders between machines  
✅ **Extensible** - Easy to add new fields  
✅ **Manual editing** - Users can tweak profiles directly  
✅ **Source tracking** - Keep original materials alongside profiles  

## Re-Analysis Feature

When you add new files to a profile's `sources/` folder:

1. Open the profile in the app
2. Press **R** to trigger re-analysis (future feature)
3. The app will scan all source files
4. Re-run AI inference on combined text
5. Update the profile with new insights

## Migration from SQLite

The old SQLite database is no longer used. Start fresh by:

1. Delete `./data/profiler.db` (if it exists)
2. Create new profiles through the app
3. They will be stored in the new folder structure

## Manual Editing

You can edit profile TOML files directly:

```bash
# Edit a profile's traits
nano ./data/profiles/john-doe/traits.toml

# Add source documents
cp ~/emails.txt ./data/profiles/john-doe/sources/

# View all profiles
ls ./data/profiles/
```

## Backup and Sync

To backup all profiles:

```bash
# Backup
tar -czf profiles-backup.tar.gz ./data/profiles/

# Restore
tar -xzf profiles-backup.tar.gz

# Sync with git
cd data/profiles
git init
git add .
git commit -m "Profile snapshot"
```

## Technical Details

- **Storage Layer**: `src/storage.rs` - Handles all file I/O
- **Format**: TOML for structured data, JSON for complex structures
- **Profile ID**: Slugified name (e.g., "John Doe" → "john-doe")
- **Logs**: Append-only JSONL for LLM request logs
- **Concurrency**: File-based locking not implemented (single-user app)

## Future Enhancements

- [ ] Re-analysis button in ProfileView screen
- [ ] Auto-watch sources/ folder for changes
- [ ] Export/import profiles as ZIP files
- [ ] Profile templates
- [ ] Bulk operations on profiles
- [ ] Source file metadata tracking
