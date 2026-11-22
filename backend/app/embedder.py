import threading
import time

from fastembed import TextEmbedding

from app.observability import embedding_duration_seconds

_model: TextEmbedding | None = None
_model_lock = threading.Lock()


def get_model() -> TextEmbedding:
    global _model
    if _model is None:
        with _model_lock:
            if _model is None:  # re-check: another thread may have loaded it while we waited
                _model = TextEmbedding(model_name="BAAI/bge-small-en-v1.5")
    return _model

def embed_query(text: str) -> list[float]:
    start = time.perf_counter()
    model = get_model()
    embeddings = list(model.embed([text]))
    embedding_duration_seconds.observe(time.perf_counter() - start)
    return embeddings[0].tolist()  

def to_pgvector_literal(embedding: list[float]) -> str:
    """asyncpg doesn't know about Postgres's vector type natively, so we
    format it as pgvector's text representation ('[0.1,0.2,...]') and cast
    it in the SQL itself with ::vector."""
    return "[" + ",".join(f"{x:.8f}" for x in embedding) + "]"

def embed_text(text: str) -> list[float]:
    """Same underlying call as embed_query — separate name because
    'query' implies search input, and this is document content being
    embedded for storage. Same model, same vector space, different intent."""
    return embed_query(text)