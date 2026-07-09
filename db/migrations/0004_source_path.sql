ALTER TABLE documents ADD COLUMN IF NOT EXISTS source_path TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS documents_source_path_idx
    ON documents (source_path) WHERE source_path IS NOT NULL;