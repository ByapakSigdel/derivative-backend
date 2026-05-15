-- Migration: tighten contact_requests
-- 1. Phone is now required (sales / support flow leans on phone callbacks).
-- 2. Message is now optional (people sometimes just want a callback and
--    don't have anything specific to write).
-- 3. New `user_type` column distinguishes individuals from organizations,
--    so admin can route the lead appropriately.
-- 4. Unique constraint on (email, phone, message) so re-submits with
--    identical content don't pile up in the database. The handler uses
--    ON CONFLICT DO NOTHING to swallow dupes silently.

-- Fill any existing NULL phones with a placeholder so the NOT NULL flip
-- doesn't fail. Same for any messages we plan to relax — no-op for now
-- since message was previously required.
UPDATE contact_requests SET phone = '' WHERE phone IS NULL;

-- Phone: required
ALTER TABLE contact_requests
    ALTER COLUMN phone SET NOT NULL;

-- Message: optional. Existing rows already have content; we just allow
-- future inserts to omit it.
ALTER TABLE contact_requests
    ALTER COLUMN message DROP NOT NULL;

-- New user_type column. Plain text rather than an enum so we can adjust
-- the option list without another migration.
ALTER TABLE contact_requests
    ADD COLUMN IF NOT EXISTS user_type VARCHAR(40) NOT NULL DEFAULT 'individual';

-- Dedupe: identical email + phone + message is the same request. The
-- handler does ON CONFLICT DO NOTHING against this constraint and just
-- echoes back the existing row.
--
-- COALESCE on message so NULL collapses to '' for comparison purposes —
-- otherwise NULL != NULL in the unique index and we'd get duplicates with
-- empty messages.
CREATE UNIQUE INDEX IF NOT EXISTS idx_contact_requests_unique_payload
    ON contact_requests (LOWER(email), phone, COALESCE(message, ''));
