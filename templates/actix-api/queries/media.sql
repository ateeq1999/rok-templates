-- The `media` feature's SQL: each caller's uploaded files. `rok sqlc g`
-- generates `src/db/queries/media.rs`; only the `media` feature calls it.
-- Every lookup is scoped to the owner, so another caller's file is never
-- found. A file uploaded straight to the store starts as an upload intent
-- (`completed_at` unset); the reads below see completed files only.

-- name: InsertMedia :one
-- A stored file's row; its bytes are already under `storage_key`.
INSERT INTO media (id, owner_id, content_type, size_bytes, original_filename, storage_key)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING *;

-- name: OwnedMedia :maybe_one
-- The file `$1`, when `$2` owns it.
SELECT *
FROM media
WHERE id = $1 AND owner_id = $2 AND completed_at IS NOT NULL;

-- name: ListOwnedMedia :many
-- One page of `$1`'s files, newest first.
-- param: $2 limit i64
-- param: $3 offset i64
SELECT *
FROM media
WHERE owner_id = $1 AND completed_at IS NOT NULL
ORDER BY created_at DESC, id DESC
LIMIT $2 OFFSET $3;

-- name: CountOwnedMedia :one
-- How many files `$1` has.
SELECT count(*) AS total
FROM media
WHERE owner_id = $1 AND completed_at IS NOT NULL;

-- name: DeleteOwnedMedia :maybe_one
-- Deletes the file `$1`'s row, when `$2` owns it: the key of its bytes.
DELETE FROM media
WHERE id = $1 AND owner_id = $2 AND completed_at IS NOT NULL
RETURNING storage_key;

-- name: InsertUploadIntent :one
-- An upload intent: the row of a file whose bytes the client puts straight
-- into the store, under `storage_key`, before `expires_at`.
-- param: $7 expires_at chrono::DateTime<chrono::Utc>
INSERT INTO media (id, owner_id, content_type, size_bytes, original_filename, storage_key, completed_at, expires_at)
VALUES ($1, $2, $3, $4, $5, $6, NULL, $7)
RETURNING *;

-- name: PendingMedia :maybe_one
-- The upload intent `$1`, when `$2` owns it and it hasn't expired.
SELECT *
FROM media
WHERE id = $1 AND owner_id = $2 AND completed_at IS NULL AND expires_at > now();

-- name: CompleteMedia :maybe_one
-- Completes the upload intent `$1`, when `$2` owns it: the file, now
-- visible to every read.
UPDATE media
SET completed_at = now(), expires_at = NULL
WHERE id = $1 AND owner_id = $2 AND completed_at IS NULL
RETURNING *;

-- name: DeletePendingMedia :maybe_one
-- Deletes the upload intent `$1`, when `$2` owns it: the key of its bytes.
DELETE FROM media
WHERE id = $1 AND owner_id = $2 AND completed_at IS NULL
RETURNING storage_key;

-- name: DeleteExpiredUploads :many
-- Deletes every upload intent past its `expires_at`: the keys of their bytes.
DELETE FROM media
WHERE completed_at IS NULL AND expires_at <= now()
RETURNING storage_key;
