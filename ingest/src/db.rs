//! Postgres writer. Deliberately uses runtime `sqlx::query()` rather than
//! the compile-time-checked `query!` macro — that macro needs a live
//! DATABASE_URL (or a cached `.sqlx` directory) just to compile, which
//! would make `cargo build` fail on a machine without Postgres running.
//! Runtime queries trade compile-time type-checking for that flexibility.

use pgvector::Vector;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
pub enum DocOutcome {
    New,
    Unchanged,
    Updated,
}

pub async fn upsert_document(
    pool: &PgPool,
    source: &str,
    title: Option<&str>,
    content_hash: &str,
    source_path: &str,
) -> anyhow::Result<(Uuid, DocOutcome)> {
    let existing = sqlx::query("SELECT id, content_hash FROM documents WHERE source_path = $1")
        .bind(source_path)
        .fetch_optional(pool)
        .await?;

    match existing {
        None => {
            let row = sqlx::query(
                "INSERT INTO documents (source, title, content_hash, source_path)
                 VALUES ($1, $2, $3, $4)
                 RETURNING id",
            )
            .bind(source)
            .bind(title)
            .bind(content_hash)
            .bind(source_path)
            .fetch_one(pool)
            .await?;
            let id: Uuid = row.get("id");
            Ok((id, DocOutcome::New))
        }
        Some(row) => {
            let id: Uuid = row.get("id");
            let existing_hash: String = row.get("content_hash");

            if existing_hash == content_hash {
                Ok((id, DocOutcome::Unchanged))
            } else {
                sqlx::query(
                    "UPDATE documents SET content_hash = $1, title = $2, updated_at = now()
                     WHERE id = $3",
                )
                .bind(content_hash)
                .bind(title)
                .bind(id)
                .execute(pool)
                .await?;

                sqlx::query("DELETE FROM chunks WHERE document_id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                Ok((id, DocOutcome::Updated))
            }
        }
    }
}

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
