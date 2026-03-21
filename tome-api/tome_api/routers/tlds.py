from fastapi import APIRouter, HTTPException

router = APIRouter()


@router.get("/{tld}")
async def get_tld(tld: str):
    """Look up information about a top-level domain."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/")
async def list_tlds(tld_type: str | None = None):
    """List all TLDs, optionally filtered by type."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/search/{query}")
async def search_tlds(query: str):
    """Search TLDs by partial match."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")
