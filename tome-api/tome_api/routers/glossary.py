from fastapi import APIRouter, HTTPException

router = APIRouter()


@router.get("/")
async def list_glossary_terms(category: str | None = None):
    """List all glossary terms, optionally filtered by category."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/search/{query}")
async def search_glossary_terms(query: str):
    """Search glossary terms by partial match."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")


@router.get("/{term}")
async def get_glossary_term(term: str):
    """Look up a glossary term."""
    # TODO: Wire up to tome Python bindings
    raise HTTPException(status_code=501, detail="Not yet implemented")
