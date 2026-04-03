# Tome

**A reference database for internet TLDs, DNS record types, and domain name terminology.**

Tome provides a comprehensive, queryable knowledge base for the domain name industry with multiple interfaces: CLI, Rust library, Python library, REST API, and MCP server for AI assistants. It is the reference companion to [Seer](https://github.com/TheZacillac/seer) — where Seer diagnoses, Tome defines.

> **Note:** Tome is in active development. The TLD database is seeded with data for 300+ TLDs including registry operators, WHOIS/RDAP servers, and country mappings. Record type and glossary databases are planned but not yet populated.

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Packages Overview](#packages-overview)
- [Installation](#installation)
- [Usage](#usage)
  - [CLI](#cli)
  - [Python Library](#python-library)
  - [Rust Library](#rust-library)
  - [REST API](#rest-api)
  - [MCP Server](#mcp-server)
- [Data Coverage](#data-coverage)
- [Configuration](#configuration)
- [Development](#development)
- [Project Structure](#project-structure)
- [Technology Stack](#technology-stack)
- [License](#license)

---

## Features

| Feature | Description |
|---------|-------------|
| **TLD Database** | Comprehensive information on all internet top-level domains — type, registry, WHOIS/RDAP servers, DNSSEC status, delegation date, and restrictions |
| **DNS Record Types** | Definitions, RDATA formats, RFC references, and examples for every DNS record type |
| **Glossary** | Plain-language definitions of DNS, registration, security, and domain industry terminology |
| **Cross-Database Search** | Search across TLDs, record types, and glossary terms in a single query |
| **Multiple Output Formats** | Human-readable, JSON, YAML, and Markdown output |
| **Multiple Interfaces** | CLI, Rust library, Python library, REST API, and MCP server |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           User Interfaces                               │
├──────────────┬──────────────┬──────────────┬────────────────────────────┤
│   tome-cli   │   tome-py    │   tome-api   │         tome-api           │
│  (Terminal)  │   (Python)   │  (REST API)  │       (MCP Server)         │
└──────┬───────┴──────┬───────┴──────┬───────┴────────────┬───────────────┘
       │              │              │                    │
       │              └──────────────┼────────────────────┘
       │                             │
       ▼                             ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                            tome-core                                     │
│                     (Core Rust Library)                                   │
├────────────────┬───────────────────┬────────────────────────────────────┤
│  TLD Database  │  Record Type DB   │         Glossary DB                │
└────────────────┴───────────────────┴────────────────────────────────────┘
```

---

## Packages Overview

Tome is a monorepo containing four packages:

| Package | Type | What You Get |
|---------|------|--------------|
| **tome-core** | Rust library | Core data models and query logic — use as a dependency in your Rust code |
| **tome-cli** | Rust binary | The `tome` command-line tool — for terminal usage |
| **tome-py** | Python extension | Python library `tome` — for Python scripts and applications |
| **tome-api** | Python package | REST API server (`tome-api`) and MCP server (`tome-mcp`) |

---

## Installation

### Installing tome-cli (Binary Only)

```bash
cargo install tome-cli
```

This installs the `tome` binary to `~/.cargo/bin/`.

**Requirements:** Rust 1.70+

### Installing tome-core (Rust Library)

Add `tome-core` to your `Cargo.toml`:

```toml
[dependencies]
tome-core = "0.1"
```

### Full Installation (All Components)

```bash
# Clone the repository
git clone https://github.com/TheZacillac/tome.git
cd tome

# Install CLI to PATH
cargo install --path tome-cli

# Build and install Python bindings
cd tome-py
uv pip install maturin
maturin develop --release
cd ..

# Install REST API and MCP server
cd tome-api
uv pip install -e .
cd ..
```

**Requirements:**
- Rust 1.70+
- Python 3.9+
- [uv](https://docs.astral.sh/uv/) (recommended) or pip

---

## Usage

### CLI

```bash
# Look up a TLD
tome tld com
tome tld .uk

# Look up a DNS record type
tome record MX
tome record 28          # By type code

# Look up a glossary term
tome glossary registrar
tome glossary DNSSEC

# Search across all databases
tome search propagation

# List entries
tome list tlds --type country_code
tome list records --common
tome list glossary --category security
```

#### Output Formats

```bash
tome --format human tld com        # Human-readable (default)
tome --format json tld com         # JSON output
tome --format yaml tld com         # YAML output
tome --format markdown tld com     # Markdown output
```

### Python Library

```python
import tome

# TLD lookups
result = tome.tld_lookup("com")
results = tome.tld_search("united")

# DNS record type lookups
result = tome.record_lookup("MX")
results = tome.record_search("mail")

# Glossary lookups
result = tome.glossary_lookup("registrar")
results = tome.glossary_search("DNSSEC")
```

### Rust Library

```rust
use tome_core::{TldDatabase, RecordTypeDatabase, GlossaryDatabase};

fn main() {
    let tld_db = TldDatabase::new(/* loaded data */);

    if let Some(tld) = tld_db.lookup("com") {
        println!("Registry: {}", tld.registry);
        println!("Type: {}", tld.tld_type);
        println!("DNSSEC: {}", tld.dnssec);
    }

    let results = tld_db.search("united");
    println!("Found {} matching TLDs", results.len());
}
```

### REST API

Start the server:

```bash
tome-api
```

The API runs on `http://localhost:8000` with auto-reload enabled.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/tlds/{tld}` | GET | Look up a TLD |
| `/tlds/` | GET | List TLDs (optional `?tld_type=` filter) |
| `/tlds/search/{query}` | GET | Search TLDs |
| `/records/{name}` | GET | Look up a record type |
| `/records/` | GET | List record types (optional `?common=true`) |
| `/records/search/{query}` | GET | Search record types |
| `/glossary/{term}` | GET | Look up a glossary term |
| `/glossary/` | GET | List glossary terms (optional `?category=` filter) |
| `/glossary/search/{query}` | GET | Search glossary terms |

API documentation available at:
- **Swagger UI:** http://localhost:8000/docs
- **ReDoc:** http://localhost:8000/redoc

### MCP Server

Start the MCP server for AI assistant integration:

```bash
tome-mcp
```

#### Claude Desktop Integration

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "tome": {
      "command": "tome-mcp"
    }
  }
}
```

---

## Data Coverage

| Database | Coverage | Status |
|----------|----------|--------|
| **TLDs** | 300+ IANA-delegated TLDs with type, registry operator, WHOIS/RDAP servers, DNSSEC status, country mappings, and transfer rules | **Active** |
| **DNS Record Types** | All IANA-registered RR types with descriptions, RDATA formats, RFC references, and examples | Planned |
| **Glossary** | DNS, registration, security, abuse, infrastructure, and protocol terminology | Planned |

---

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level (`trace`, `debug`, `info`, `warn`, `error`) | — |
| `TOME_CORS_ORIGINS` | Comma-separated CORS origins for REST API | `*` (all) |
| `TOME_RATE_LIMIT` | Rate limit for REST API (requests/minute) | `60` |
| `TOME_HOST` | REST API bind address | `0.0.0.0` |
| `TOME_PORT` | REST API port | `8000` |
| `TOME_LOG_FORMAT` | Log format (`text` or `json`) | `text` |
| `TOME_LOG_LEVEL` | Log level for REST API | `INFO` |

---

## Development

### Building

```bash
# Build all Rust packages
cargo build --release

# Build Python bindings
cd tome-py && maturin develop --release

# Install API package
cd tome-api && pip install -e .
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests
cd tome-api && pytest
```

### Logging

Enable debug logging:

```bash
RUST_LOG=debug tome tld com
```

---

## Project Structure

```
tome/
├── README.md               # This file
├── Cargo.toml              # Workspace configuration
├── tome-core/              # Core Rust library (data models, SQLite, query logic)
│   ├── schema/
│   │   └── tld_schema.sql  # SQLite schema definition
│   └── src/
│       ├── lib.rs          # Module exports
│       ├── error.rs        # Error types
│       ├── db.rs           # SQLite database (TomeDb) — typed query layer
│       ├── seed.rs         # Core TLD seed data (~90 TLDs)
│       ├── seed_extended.rs # Extended seed data (~200+ ccTLDs, ~50 nTLDs)
│       ├── tld.rs          # TLD data model (in-memory)
│       ├── record_type.rs  # DNS record type data model and database
│       ├── glossary.rs     # Glossary term data model and database
│       └── output.rs       # Output formatters (human/JSON/YAML/Markdown)
│
├── tome-cli/               # CLI application
│   └── src/
│       └── main.rs         # Entry point with clap commands
│
├── tome-py/                # Python bindings (PyO3)
│   ├── pyproject.toml      # Maturin build config
│   ├── src/lib.rs          # Python module definitions
│   └── python/tome/        # Python wrapper module
│
└── tome-api/               # FastAPI REST server + MCP
    ├── pyproject.toml
    └── tome_api/
        ├── main.py         # FastAPI app
        ├── routers/        # API endpoints (tlds, records, glossary)
        └── mcp/            # MCP server
```

---

## Technology Stack

### Core (Rust)

| Dependency | Purpose |
|------------|---------|
| Serde | Serialization (JSON, YAML) |
| Rusqlite | SQLite database (bundled) |
| Thiserror | Error handling |
| Chrono | Date/time handling |

### CLI

| Dependency | Purpose |
|------------|---------|
| Clap | Command-line parsing |
| Comfy-table | Table formatting |
| Colored | Terminal colors |

### Python

| Dependency | Purpose |
|------------|---------|
| PyO3 | Rust/Python bindings |
| FastAPI | REST API framework |
| Pydantic | Data validation |
| MCP | Model Context Protocol |

---

## License

MIT License

Copyright (c) 2026 Zac Roach

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
