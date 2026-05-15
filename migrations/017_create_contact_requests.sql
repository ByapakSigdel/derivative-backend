-- Migration: Contact requests
-- Stores "Get access" form submissions from anonymous trial users on the
-- editor page. Admins read these on /admin to follow up out-of-band (email
-- / phone). No FK to user_profiles by design — submitters are anonymous.

CREATE TABLE IF NOT EXISTS contact_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(120) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(60),
    message TEXT NOT NULL,
    contacted BOOLEAN NOT NULL DEFAULT FALSE,
    contacted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Admin list view is ordered newest-first; this index turns the implicit
-- sort into an index scan.
CREATE INDEX IF NOT EXISTS idx_contact_requests_created_at
    ON contact_requests (created_at DESC);

-- Quick filter for "open / not yet contacted" requests on the admin page.
CREATE INDEX IF NOT EXISTS idx_contact_requests_contacted
    ON contact_requests (contacted);
