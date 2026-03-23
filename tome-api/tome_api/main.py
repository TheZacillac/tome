import logging
import os

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address

from tome_api.routers import glossary, records, tlds

log_format = os.getenv("TOME_LOG_FORMAT", "text")
log_level = os.getenv("TOME_LOG_LEVEL", "INFO").upper()
logging.basicConfig(level=getattr(logging, log_level, logging.INFO))
logger = logging.getLogger(__name__)

rate_limit = os.getenv("TOME_RATE_LIMIT", "60/minute")

limiter = Limiter(key_func=get_remote_address, default_limits=[rate_limit])

app = FastAPI(
    title="Tome API",
    description="A reference database for internet TLDs, DNS record types, and domain name terminology",
    version="0.1.0",
)

app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

cors_origins = os.getenv("TOME_CORS_ORIGINS", "*").split(",")
app.add_middleware(
    CORSMiddleware,
    allow_origins=cors_origins,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(tlds.router, prefix="/tlds", tags=["TLDs"])
app.include_router(records.router, prefix="/records", tags=["Record Types"])
app.include_router(glossary.router, prefix="/glossary", tags=["Glossary"])


@app.get("/health")
async def health():
    return {"status": "ok"}


def run():
    import uvicorn

    host = os.getenv("TOME_HOST", "0.0.0.0")
    port = int(os.getenv("TOME_PORT", "8000"))
    reload = os.getenv("TOME_RELOAD", "false").lower() in ("true", "1", "yes")
    uvicorn.run("tome_api.main:app", host=host, port=port, reload=reload)
