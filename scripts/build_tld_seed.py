#!/usr/bin/env python3
"""
Build comprehensive TLD seed data by merging:
1. IANA root zone database (authoritative list of all TLDs with type + registry)
2. Existing rich metadata from data/tlds.json (Realtime Register source)

Output: data/tlds.json with all IANA TLDs, enriched where we have metadata.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IANA_FILE = ROOT / "data" / "iana-tlds.txt"
EXISTING_FILE = ROOT / "data" / "tlds.json"
IANA_DB_FILE = ROOT / "data" / "iana-root-db.txt"
OUTPUT_FILE = ROOT / "data" / "tlds.json"

# Map IANA type strings to our enum values
TYPE_MAP = {
    "generic": "generic",
    "country-code": "country_code",
    "sponsored": "sponsored",
    "infrastructure": "infrastructure",
    "test": "test",
    "generic-restricted": "generic_restricted",
}

# Known WHOIS servers for TLDs not in our existing data
WHOIS_SERVERS = {
    "com": "whois.verisign-grs.com",
    "net": "whois.verisign-grs.com",
    "org": "whois.pir.org",
    "edu": "whois.educause.edu",
    "gov": "whois.dotgov.gov",
    "int": "whois.iana.org",
    "arpa": "whois.iana.org",
}

# TLDs known to have no WHOIS service
NO_WHOIS_TLDS = {"mil"}

# Well-known new gTLD registries and their WHOIS/RDAP patterns
REGISTRY_WHOIS = {
    "Binky Moon, LLC": ("whois.nic.{tld}", "https://rdap.donuts.co/rdap/"),
    "Dog Beach, LLC": ("whois.nic.{tld}", "https://rdap.donuts.co/rdap/"),
    "Charleston Road Registry Inc.": ("whois.nic.google", "https://rdap.nic.google/"),
    "Amazon Registry Services, Inc.": ("whois.nic.{tld}", "https://rdap.nominet.uk/{tld}/"),
    "Identity Digital Limited": ("whois.nic.{tld}", "https://rdap.donuts.co/rdap/"),
    "XYZ.COM LLC": ("whois.nic.{tld}", "https://rdap.centralnic.com/{tld}/"),
    "Radix Technologies Inc.": ("whois.nic.{tld}", "https://rdap.centralnic.com/{tld}/"),
    "Registry Services, LLC": ("whois.nic.{tld}", None),
    "Minds + Machines Group Limited": ("whois.nic.{tld}", None),
    "Afilias, Inc.": ("whois.nic.{tld}", "https://rdap.afilias.net/rdap/{tld}/"),
    "Internet Corporation for Assigned Names and Numbers": ("whois.iana.org", None),
}


def make_default_tld(tld: str, tld_type: str, registry: str) -> dict:
    """Create a TLD entry with sensible defaults."""
    is_cctld = tld_type == "country_code"
    is_generic = tld_type in ("generic", "new_generic")
    is_infra = tld_type == "infrastructure"

    # Classify new gTLDs vs legacy gTLDs
    legacy_gtlds = {"com", "net", "org", "int", "edu", "gov", "mil"}
    is_new_gtld = is_generic and tld not in legacy_gtlds and len(tld) > 2

    actual_type = tld_type
    if is_new_gtld and tld_type == "generic":
        actual_type = "new_generic"

    # Determine WHOIS server
    whois = WHOIS_SERVERS.get(tld)
    rdap = None
    if tld in NO_WHOIS_TLDS:
        whois = None
    elif whois is None and registry in REGISTRY_WHOIS:
        ws, rd = REGISTRY_WHOIS[registry]
        if ws:
            whois = ws.format(tld=tld)
        if rd:
            rdap = rd.format(tld=tld)
    if whois is None and tld not in NO_WHOIS_TLDS and (is_new_gtld or is_cctld):
        whois = f"whois.nic.{tld}"

    entry = {
        "tld": tld,
        "tld_type": actual_type,
        "name": tld.upper() if is_cctld else tld.capitalize() if len(tld) <= 4 else tld.replace("-", " ").title(),
        "registry": registry,
        "whois_server": whois,
        "rdap_url": rdap,
        "dnssec": "signed" if is_infra else "unknown",
        "idn_support": False,
        "references": [],
        "allowed_countries": [],
        "create_periods": [1] if is_cctld else [1, 2, 3, 5, 10] if is_new_gtld else [1, 2, 3, 5, 10],
        "renew_periods": [1] if is_cctld else [1, 2, 3, 5, 10] if is_new_gtld else [1, 2, 3, 5, 10],
        "auto_renew_periods": [1],
        "transfer_periods": [1] if not is_infra else [],
        "features": ["CREATE", "RENEW", "TRANSFER", "RESTORE", "UPDATE"],
        "contacts": [
            {
                "contact_type": "registrant",
                "min": 1, "max": 1,
                "required": True,
                "organization_allowed": True,
                "organization_required": False,
                "scope": "registrar"
            }
        ],
        "allowed_dnssec_algorithms": [8, 13, 14],
        "whois_exposure": "unknown",
        "gdpr_category": "unknown",
    }

    # Not-assigned TLDs get minimal features
    if registry == "Not assigned":
        entry["features"] = []
        entry["create_periods"] = []
        entry["renew_periods"] = []
        entry["transfer_periods"] = []
        entry["contacts"] = []
        entry["whois_server"] = None
        entry["dnssec"] = "unknown"

    return entry


def parse_iana_root_db(path: Path) -> dict[str, tuple[str, str]]:
    """Parse the IANA root DB extract (format: .tld | type | registry)"""
    entries = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or not line.startswith("."):
                continue
            parts = [p.strip() for p in line.split("|")]
            if len(parts) >= 3:
                tld = parts[0].lstrip(".").lower()
                tld_type = TYPE_MAP.get(parts[1].strip(), "generic")
                registry = parts[2].strip()
                entries[tld] = (tld_type, registry)
    return entries


def parse_iana_tld_list(path: Path) -> set[str]:
    """Parse the IANA TLD alpha list."""
    tlds = set()
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                tlds.add(line.lower())
    return tlds


def main():
    # Load existing rich metadata
    existing = {}
    if EXISTING_FILE.exists():
        with open(EXISTING_FILE) as f:
            for entry in json.load(f):
                existing[entry["tld"]] = entry
        print(f"Loaded {len(existing)} existing entries with rich metadata")

    # Parse IANA sources
    iana_tlds = parse_iana_tld_list(IANA_FILE)
    print(f"IANA TLD list: {len(iana_tlds)} TLDs")

    iana_db = {}
    if IANA_DB_FILE.exists():
        iana_db = parse_iana_root_db(IANA_DB_FILE)
        print(f"IANA root DB: {len(iana_db)} entries with type + registry")

    # Merge: IANA list is authoritative, enrich with existing metadata
    result = []
    new_count = 0
    enriched_count = 0

    for tld in sorted(iana_tlds):
        if tld in existing:
            entry = existing[tld]
            # Update type/registry from IANA if available (more authoritative)
            if tld in iana_db:
                iana_type, iana_registry = iana_db[tld]
                if iana_registry != "Not assigned":
                    entry["registry"] = iana_registry
                entry["tld_type"] = iana_type
                # Reclassify new gTLDs
                if iana_type == "generic" and tld not in {"com", "net", "org", "int", "edu", "gov", "mil"} and len(tld) > 2:
                    entry["tld_type"] = "new_generic"
            enriched_count += 1
            result.append(entry)
        else:
            # New entry from IANA
            tld_type = "generic"
            registry = "Unknown"
            if tld in iana_db:
                tld_type, registry = iana_db[tld]
            entry = make_default_tld(tld, tld_type, registry)
            new_count += 1
            result.append(entry)

    # Also keep any existing entries not in IANA (SLDs like com.au, co.uk, etc.)
    sld_count = 0
    for tld, entry in existing.items():
        if tld not in iana_tlds:
            result.append(entry)
            sld_count += 1

    result.sort(key=lambda e: e["tld"])

    print(f"\nResult: {len(result)} total entries")
    print(f"  - {enriched_count} enriched from existing metadata")
    print(f"  - {new_count} new from IANA")
    print(f"  - {sld_count} SLDs from existing data (not in IANA root)")

    with open(OUTPUT_FILE, "w") as f:
        json.dump(result, f, indent=2, ensure_ascii=False)
    print(f"\nWritten to {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
