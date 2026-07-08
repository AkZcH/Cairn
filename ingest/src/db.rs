//! Postgres writer. Deliberately uses runtime `sqlx::query()` rather than
//! the compile-time-checked `query!` macro — that macro needs a live
//! DATABASE_URL (or a cached `.sqlx` directory) just to compile, which
//! would make `cargo build` fail on a machine without Postgres running.
//! Runtime queries trade compile-time type-checking for that flexibility.

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use pgvector::Vector;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Inserts a document, or returns the existing one's id if this exact
/// content (by hash) was already ingested. This is the dedup hook the
/// schema was built for.
pub async fn upsert_document(
    pool: &PgPool,
    source: &str,
    title: Option<&str>,
    content_hash: &str,
) -> anyhow::Result<(Uuid, bool)> {
    // Try the insert first.
    let inserted = sqlx::query(
        "INSERT INTO documents (source, title, content_hash)
         VALUES ($1, $2, $3)
         ON CONFLICT (content_hash) DO NOTHING
         RETURNING id",
    )
    .bind(source)
    .bind(title)
    .bind(content_hash)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = inserted {
        let id: Uuid = row.get("id");
        return Ok((id, true)); // true = newly inserted
    }

    // Already existed — fetch its id instead.
    let existing = sqlx::query("SELECT id FROM documents WHERE content_hash = $1")
        .bind(content_hash)
        .fetch_one(pool)
        .await?;

    let id: Uuid = existing.get("id");
    Ok((id, false)) // false = already existed
}

// pub async fn insert_chunk(
//     pool: &PgPool,
//     document_id: Uuid,
//     chunk_index: i32,
//     content: &str,
// ) -> anyhow::Result<()> {
//     sqlx::query(
//         "INSERT INTO chunks (document_id, chunk_index, content)
//          VALUES ($1, $2, $3)
//          ON CONFLICT (document_id, chunk_index) DO NOTHING",
//     )
//     .bind(document_id)
//     .bind(chunk_index)
//     .bind(content)
//     .execute(pool)
//     .await?;
//     Ok(())
// }

pub async fn insert_chunk(
    pool: &PgPool,
    document_id: Uuid,
    chunk_index: i32,
    content: &str,
    embedding: &[f32],
) -> anyhow::Result<()> {
    let vector = Vector::from(embedding.to_vec());

    sqlx::query(
        "INSERT INTO chunks (document_id, chunk_index, content, embedding)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (document_id, chunk_index) DO NOTHING",
    )
    .bind(document_id)
    .bind(chunk_index)
    .bind(content)
    .bind(vector)
    .execute(pool)
    .await?;
    Ok(())
}