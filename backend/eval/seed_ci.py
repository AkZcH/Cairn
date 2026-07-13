"""Seeds a fixed test user and ingests README.md + PRD.md for CI eval runs.
Only these two documents are used in CI because they're the only ones
actually checked into the repo, everything else in the full dataset lives
only in a developer's local Postgres and can't be reproduced from a clean
checkout."""

import asyncio
from pathlib import Path

from app.api.upload import _ingest_text
from app.auth import hash_password
from app.db import get_pool

CI_EMAIL = "ci-eval@cairn.local"
REPO_ROOT = Path(__file__).resolve().parents[2]


async def main():
    pool = await get_pool()

    row = await pool.fetchrow("SELECT id FROM users WHERE email = $1", CI_EMAIL)
    if row is None:
        row = await pool.fetchrow(
            """INSERT INTO users (email, password_hash, api_key_hash)
               VALUES ($1, $2, 'unused') RETURNING id""",
            CI_EMAIL, hash_password("ci-only-not-a-real-password"),
        )
    user_id = row["id"]

    readme = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    prd = (REPO_ROOT / "docs" / "PRD.md").read_text(encoding="utf-8")

    await _ingest_text(user_id, "ci:README.md", "README.md", readme)
    await _ingest_text(user_id, "ci:PRD.md", "PRD.md", prd)

    print(f"Seeded CI eval user: {user_id}")


if __name__ == "__main__":
    asyncio.run(main())