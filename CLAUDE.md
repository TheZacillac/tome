# CLAUDE.md - Tome

Tome is a reference database for internet TLDs, DNS record types, and domain name terminology. It mirrors Seer's workspace structure: Rust core + CLI + Python bindings + REST API.

**Status:** Early development — data models and workspace structure in place, datasets not yet populated.

---

## Architecture

```
tome/
├── Cargo.toml              # Workspace root
├── tome-core/              # Core Rust library (all data models & query logic)
├── tome-cli/               # CLI application (clap)
├── tome-py/                # Python bindings (PyO3)
├── tome-api/               # FastAPI REST server + MCP server
└── scripts/
    └── build_tld_seed.py   # TLD data generation from IANA sources
```

### Dependency Flow

```
tome-cli ──┐
           ├──> tome-core (Rust core library)
tome-py ───┘

tome-api (Python) ──> tome-py (Python package) ──> tome-core (Rust)
```

All business logic lives in `tome-core`. Other crates/packages are thin presentation layers.

---

## Core Data Models (tome-core)

### Three Knowledge Bases

**TldDatabase** — TLD reference data:
- Core: tld name, type (Generic/CountryCode/Sponsored/Infrastructure/NewGeneric/GenericRestricted/Test), registry, WHOIS/RDAP servers
- IANA: delegation date, references, DNSSEC support (Signed/Unsigned/Unknown)
- Syntax: IDN support, min/max length
- Registration: restrictions, allowed countries, transfer requirements, contact requirements
- Periods: create/renew/transfer/redemption/grace periods (years or days)
- Features: CREATE, RENEW, TRANSFER, RESTORE, PRIVACY_PROTECT, REGISTRY_LOCK
- Privacy: WHOIS exposure, GDPR category, jurisdiction

**RecordTypeDatabase** — DNS record type definitions:
- name, type_code (IANA numeric), summary, description
- rdata_format, example (zone file format)
- rfcs, status (Active/Experimental/Obsolete/Reserved), common flag
- related record types

**GlossaryDatabase** — Domain industry terminology:
- term, abbreviation, summary, description
- category (Dns/Registration/Security/Abuse/Infrastructure/Protocol/General)
- related terms, references (RFCs)

### Query Patterns

All databases implement consistent methods:
- `lookup()` — exact match (case-insensitive, normalized)
- `search()` — partial substring match across multiple fields
- `by_type()`/`by_category()` — filter by enumerated type
- `count()`, `all()` — aggregate accessors

### Error Handling

`TomeError` enum with thiserror:
- `TldNotFound`, `RecordTypeNotFound`, `GlossaryTermNotFound`
- `InvalidQuery`, `DataError`
- `SerializationError` (from serde_json)
- `DatabaseError` (from rusqlite) — planned, not yet implemented
- `sanitized_message()` method for safe API exposure

### Output Formatting

`OutputFormatter` supports Human, Json, Yaml, Markdown. Currently only JSON is fully implemented; others fall back to JSON.

---

## CLI (tome-cli)

Binary name: `tome`

```
tome tld <query>              # Look up TLD (e.g., "com", ".uk")
tome record <query>           # Look up record type by name or code
tome glossary <query>         # Look up glossary term
tome search <query>           # Cross-database search
tome list tlds [--type TYPE]  # List TLDs, optionally filtered
tome list records [--common]  # List record types
tome list glossary [--category CATEGORY]  # List glossary terms

Global: --format human|json|yaml|markdown
```

**Current state:** Argument parsing is complete. Command handlers print placeholder messages — wiring to tome-core databases is TODO.

---

## Python Bindings (tome-py)

Built with PyO3 + Maturin. ABI3 targeting Python 3.9+.

```python
import tome

tome.tld_lookup("com")           # Returns dict or None
tome.tld_search("united")        # Returns list of dicts
tome.record_lookup("MX")         # By name or numeric code ("15")
tome.record_search("mail")       # Returns list of dicts
tome.glossary_lookup("registrar")
tome.glossary_search("DNSSEC")
```

**Key implementation details:**
- `OnceLock<T>` for lazy-initialized databases (thread-safe singletons)
- `to_py_dict()` converts serde objects via JSON serialization
- Currently databases initialized with empty Vec (no data populated)

---

## REST API (tome-api)

FastAPI application on port 8000.

```
GET /tlds/{tld}              GET /tlds/              GET /tlds/search/{query}
GET /records/{name}          GET /records/            GET /records/search/{query}
GET /glossary/{term}         GET /glossary/           GET /glossary/search/{query}
GET /health
```

**All endpoints currently return 501 Not Implemented.**

**Environment variables:**
| Variable | Default | Purpose |
|----------|---------|---------|
| `TOME_LOG_LEVEL` | `INFO` | Log level |
| `TOME_RATE_LIMIT` | `60/minute` | Rate limiting |
| `TOME_CORS_ORIGINS` | `*` | CORS origins |
| `TOME_HOST` | `0.0.0.0` | Bind address |
| `TOME_PORT` | `8000` | Port |

**MCP server** (`tome-mcp`): Stub only — prints "not yet implemented".

---

## Data Generation

`scripts/build_tld_seed.py` merges IANA root zone data with existing rich metadata:
- Input: `data/iana-tlds.txt`, `data/iana-root-db.txt`, `data/tlds.json`
- Output: enriched `data/tlds.json`
- Handles type classification, WHOIS/RDAP server detection, default entries

---

## Build & Test

```bash
# Rust
cargo build --release
cargo test
cargo clippy -- -D warnings
cargo fmt

# Python bindings
cd tome-py && maturin develop --release

# API
cd tome-api && pip install -e .
tome-api        # Start server
```

---

## Code Conventions

Follows the same conventions as Seer (see `seer/CLAUDE.md`):
- All business logic in tome-core
- PascalCase types, snake_case functions, SCREAMING_SNAKE constants
- thiserror for error handling, never unwrap() in library code
- Serde derive on all data models
- Module layout: `mod.rs` (public interface), then feature files

---

## Key Dependencies

**Rust workspace:** tokio 1, serde 1, thiserror 2, rusqlite 0.35 (bundled), clap 4, pyo3 0.24, comfy-table 7, colored 2, tracing 0.1

**Python API:** fastapi, uvicorn, pydantic, mcp, slowapi
