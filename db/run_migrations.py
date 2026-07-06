import asyncio
import os
from pathlib import Path

import asyncpg

MIGRATIONS_DIR = Path(__file__).parent / "migrations"


def get_dsn() -> str:
    user = os.environ.get("POSTGRES_USER", "cairn")
    password = os.environ.get("POSTGRES_PASSWORD", "changeme")
    db = os.environ.get("POSTGRES_DB", "cairn")
    host = os.environ.get("POSTGRES_HOST", "db")
    port = os.environ.get("POSTGRES_PORT", "5432")
    return f"postgresql://{user}:{password}@{host}:{port}/{db}"


async def run_migrations() -> None:
    conn = await asyncpg.connect(dsn=get_dsn())

    await conn.execute(
        """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            filename TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        """
    )

    applied = {
        row["filename"]
        for row in await conn.fetch("SELECT filename FROM schema_migrations")
    }

    for path in sorted(MIGRATIONS_DIR.glob("*.sql")):
        if path.name in applied:
            print(f"skip (already applied): {path.name}")
            continue

        print(f"applying: {path.name}")
        sql = path.read_text()
        async with conn.transaction():
            await conn.execute(sql)
            await conn.execute(
                "INSERT INTO schema_migrations (filename) VALUES ($1)", path.name
            )
        print(f"done: {path.name}")

    await conn.close()


if __name__ == "__main__":
    asyncio.run(run_migrations())