CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    api_key_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE documents ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE conversations ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE CASCADE;

-- source_path uniqueness must now be scoped per-user, not global,
-- two different users can both have a file called "note.md".
DROP INDEX IF EXISTS documents_source_path_idx;
CREATE UNIQUE INDEX IF NOT EXISTS documents_user_source_path_idx
    ON documents (user_id, source_path) WHERE source_path IS NOT NULL AND user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS documents_user_id_idx ON documents (user_id);
CREATE INDEX IF NOT EXISTS conversations_user_id_idx ON conversations (user_id);