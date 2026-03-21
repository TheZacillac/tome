"""Tome — a reference database for internet TLDs, DNS record types, and domain name terminology."""

from tome._tome import (
    tld_lookup,
    tld_search,
    record_lookup,
    record_search,
    glossary_lookup,
    glossary_search,
)

try:
    from importlib.metadata import version

    __version__ = version("tome")
except Exception:
    __version__ = "0.1.0"

__all__ = [
    "tld_lookup",
    "tld_search",
    "record_lookup",
    "record_search",
    "glossary_lookup",
    "glossary_search",
]
