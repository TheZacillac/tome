from fastapi import APIRouter, HTTPException

router = APIRouter()


@router.get("/{name}")
async def get_record_type(name: str):
    """Look up information about a DNS record type."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/")
async def list_record_types(common: bool = False):
    """List all DNS record types."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/search/{query}")
async def search_record_types(query: str):
    """Search record types by partial match."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")
