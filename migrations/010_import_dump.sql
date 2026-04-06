-- Migration: 010_import_dump.sql
-- Data import from Supabase dump.sql
-- This migration imports only DATA (not schema) since migrations 001-009 already created the tables.
-- Uses INSERT ... ON CONFLICT DO NOTHING for idempotency.

-- ============================================
-- organizations data
-- ============================================
INSERT INTO public.organizations (id, name, description, created_at, updated_at)
VALUES (
    '6eadc5b9-b111-4b27-b17f-34aa5aa79865',
    'Default Organization',
    'Default organization for new users',
    '2026-02-06 06:50:01.000732+00',
    '2026-02-06 06:50:01.000732+00'
)
ON CONFLICT (id) DO NOTHING;

-- ============================================
-- user_profiles data
-- Note: These profiles reference auth.users which may not exist in the new DB.
-- The foreign key constraint will fail if the user doesn't exist.
-- We'll use DO NOTHING to skip if already exists or FK fails.
-- ============================================

-- Profile: poshan@poshan.com
-- Note: full_name and password_hash are NOT NULL in local schema, using placeholder values
INSERT INTO public.user_profiles (id, email, full_name, password_hash, user_type, organization_id, avatar_url, is_active, created_at, updated_at)
VALUES (
    'adfcb56a-38cf-4f0b-863f-5fe2c97987ab',
    'poshan@poshan.com',
    'poshan',
    '$argon2id$v=19$m=65536,t=3,p=4$IMPORTED_USER_NEEDS_PASSWORD_RESET',
    'user',
    NULL,
    NULL,
    true,
    '2026-02-07 13:46:25.827082+00',
    '2026-02-07 13:46:25.827082+00'
)
ON CONFLICT (id) DO NOTHING;

-- Profile: byapak@byapak.com (admin)
-- Note: full_name and password_hash are NOT NULL in local schema, using placeholder values
INSERT INTO public.user_profiles (id, email, full_name, password_hash, user_type, organization_id, avatar_url, is_active, created_at, updated_at)
VALUES (
    'f7dc94b1-13b2-4a85-92e8-92da6dbd03cc',
    'byapak@byapak.com',
    'byapak',
    '$argon2id$v=19$m=65536,t=3,p=4$IMPORTED_USER_NEEDS_PASSWORD_RESET',
    'admin',
    NULL,
    NULL,
    true,
    '2026-02-06 06:53:28.840277+00',
    '2026-02-08 04:32:20.044758+00'
)
ON CONFLICT (id) DO NOTHING;

-- Profile: testuser1@derivative.com
-- Note: password_hash is NOT NULL in local schema, using placeholder value
INSERT INTO public.user_profiles (id, email, full_name, password_hash, user_type, organization_id, avatar_url, is_active, created_at, updated_at)
VALUES (
    '204b9247-4e97-4309-a71a-c2483bb8b46f',
    'testuser1@derivative.com',
    'testUser1',
    '$argon2id$v=19$m=65536,t=3,p=4$IMPORTED_USER_NEEDS_PASSWORD_RESET',
    'user',
    '6eadc5b9-b111-4b27-b17f-34aa5aa79865',
    NULL,
    true,
    '2026-02-08 04:58:51.508223+00',
    '2026-02-08 04:58:51.674565+00'
)
ON CONFLICT (id) DO NOTHING;

-- Profile: testuser2@codeatderivative.com
-- Note: password_hash is NOT NULL in local schema, using placeholder value
INSERT INTO public.user_profiles (id, email, full_name, password_hash, user_type, organization_id, avatar_url, is_active, created_at, updated_at)
VALUES (
    '854a964f-a844-48b4-bbe1-57a57046c68e',
    'testuser2@codeatderivative.com',
    'testuser2',
    '$argon2id$v=19$m=65536,t=3,p=4$IMPORTED_USER_NEEDS_PASSWORD_RESET',
    'user',
    '6eadc5b9-b111-4b27-b17f-34aa5aa79865',
    NULL,
    true,
    '2026-02-08 05:25:22.514364+00',
    '2026-02-08 05:25:45.998683+00'
)
ON CONFLICT (id) DO NOTHING;

-- Profile: sandip@sandip.com (admin)
-- Note: full_name and password_hash are NOT NULL in local schema, using placeholder values
INSERT INTO public.user_profiles (id, email, full_name, password_hash, user_type, organization_id, avatar_url, is_active, created_at, updated_at)
VALUES (
    '31684c1d-7acd-44b3-ad95-a8717ab2860d',
    'sandip@sandip.com',
    'sandip',
    '$argon2id$v=19$m=65536,t=3,p=4$IMPORTED_USER_NEEDS_PASSWORD_RESET',
    'admin',
    NULL,
    NULL,
    true,
    '2026-02-08 15:03:24.05782+00',
    '2026-02-08 15:03:46.16377+00'
)
ON CONFLICT (id) DO NOTHING;

-- ============================================
-- user_projects data
-- Note: category enum only allows: tutorial, game, simulation, art, music, utility, education, other
-- Mapping invalid values (LED, led, Uncategorized) to 'other'
-- ============================================

-- Project: sandip_led - SKIPPED: references non-existent user_id '00350d75-2d87-49e9-bafc-32d0fcc53326'
-- This project was likely created by a user that no longer exists or was never migrated
-- Original data preserved in comment for reference:
-- user_id: '00350d75-2d87-49e9-bafc-32d0fcc53326', title: 'sandip_led', category: 'LED'

-- Project: title (by sandip@sandip.com)
-- Note: category 'led' mapped to 'other'
INSERT INTO public.user_projects (
    id, user_id, title, description, difficulty, category, 
    nodes, edges, materials, learning_goals, tags, 
    is_public, featured, view_count, clone_count, 
    created_at, updated_at, published_at, like_count, comment_count
)
VALUES (
    '779085b0-e584-43d5-b89e-17eeb3eb001c',
    '31684c1d-7acd-44b3-ad95-a8717ab2860d',
    'title',
    'test',
    'beginner',
    'other',
    '[{"id": "setup-start", "data": {"type": "SetupStart", "label": "Setup (Run Once)"}, "type": "SetupStart", "width": 200, "height": 149, "dragging": false, "position": {"x": 250, "y": 50}, "selected": false, "deletable": false, "draggable": true, "selectable": true}, {"id": "loop-start", "data": {"type": "LoopStart", "label": "Loop (Run Forever)"}, "type": "LoopStart", "width": 200, "height": 149, "position": {"x": 250, "y": 300}, "deletable": false, "draggable": true, "selectable": true}, {"id": "n1", "data": {"type": "PinConfig", "label": "Configure Pin", "inputs": 1, "params": [{"name": "Pin", "value": 13}, {"name": "Mode", "value": "OUTPUT"}], "outputs": 1}, "type": "custom", "width": 200, "height": 139, "dragging": false, "position": {"x": 525, "y": 180}, "selected": false, "draggable": true, "selectable": true, "positionAbsolute": {"x": 525, "y": 180}}]'::jsonb,
    '[{"id": "reactflow__edge-setup-start-n1", "type": "default", "style": {"stroke": "#64748b", "strokeWidth": 3}, "source": "setup-start", "target": "n1", "animated": false, "sourceHandle": null, "targetHandle": null}]'::jsonb,
    ARRAY[]::TEXT[],
    ARRAY[]::TEXT[],
    ARRAY[]::TEXT[],
    true,
    false,
    0,
    0,
    '2026-02-09 06:37:33.771779+00',
    '2026-02-09 06:37:33.771779+00',
    NULL,
    0,
    0
)
ON CONFLICT (id) DO NOTHING;

-- Project: test project (by byapak@byapak.com)
-- Note: category 'Uncategorized' mapped to 'other'
INSERT INTO public.user_projects (
    id, user_id, title, description, difficulty, category, 
    nodes, edges, materials, learning_goals, tags, 
    is_public, featured, view_count, clone_count, 
    created_at, updated_at, published_at, like_count, comment_count
)
VALUES (
    '010917dc-f9bd-4b9f-b466-c38a7d001ecf',
    'f7dc94b1-13b2-4a85-92e8-92da6dbd03cc',
    'test project',
    NULL,
    'beginner',
    'other',
    '[{"id": "setup-start", "data": {"type": "SetupStart", "label": "Setup (Run Once)"}, "type": "SetupStart", "width": 200, "height": 149, "dragging": false, "position": {"x": 120, "y": 105}, "selected": false, "deletable": false, "draggable": true, "selectable": true, "positionAbsolute": {"x": 120, "y": 105}}, {"id": "loop-start", "data": {"type": "LoopStart", "label": "Loop (Run Forever)"}, "type": "LoopStart", "width": 200, "height": 149, "dragging": false, "position": {"x": 465, "y": 90}, "selected": false, "deletable": false, "draggable": true, "selectable": true, "positionAbsolute": {"x": 465, "y": 90}}, {"id": "n1", "data": {"type": "PinConfig", "label": "Configure Pin", "inputs": 1, "params": [{"name": "Pin", "value": 13}, {"name": "Mode", "value": "OUTPUT"}], "outputs": 1}, "type": "custom", "width": 200, "height": 139, "dragging": false, "position": {"x": 105, "y": 300}, "selected": false, "draggable": true, "selectable": true, "positionAbsolute": {"x": 105, "y": 300}}, {"id": "n2", "data": {"type": "DigitalWrite", "label": "Turn LED On/Off", "inputs": 1, "params": [{"name": "Pin", "value": 13}, {"name": "Value", "value": true}], "outputs": 1}, "type": "custom", "width": 200, "height": 139, "position": {"x": 555, "y": 390}, "draggable": true, "selectable": true}]'::jsonb,
    '[{"id": "reactflow__edge-setup-start-n1", "type": "default", "style": {"stroke": "#64748b", "strokeWidth": 3}, "source": "setup-start", "target": "n1", "animated": false, "sourceHandle": null, "targetHandle": null}, {"id": "reactflow__edge-loop-start-n2", "type": "default", "style": {"stroke": "#64748b", "strokeWidth": 3}, "source": "loop-start", "target": "n2", "animated": false, "sourceHandle": null, "targetHandle": null}]'::jsonb,
    ARRAY[]::TEXT[],
    ARRAY[]::TEXT[],
    ARRAY[]::TEXT[],
    true,
    false,
    0,
    0,
    '2026-02-08 06:32:50.225369+00',
    '2026-03-24 10:16:57.749538+00',
    '2026-02-10 05:55:16.53179+00',
    0,
    0
)
ON CONFLICT (id) DO NOTHING;

-- ============================================
-- project_comments data (empty in dump)
-- ============================================
-- No data to insert

-- ============================================
-- project_likes data (empty in dump)
-- ============================================
-- No data to insert

-- ============================================
-- project_views data (empty in dump)
-- ============================================
-- No data to insert
