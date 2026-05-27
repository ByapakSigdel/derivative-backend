-- Migration: 019_extend_user_type_roles
-- Description: Add org_admin / teacher / student roles to the user_type enum.
--
-- Role hierarchy: admin (platform) > org_admin > teacher > student > user.
--
-- NOTE: `ALTER TYPE ... ADD VALUE` is allowed inside a transaction only on
-- PostgreSQL 12+ (DigitalOcean managed Postgres is 12+). The new values are
-- NOT used anywhere in this migration, so committing the transaction makes
-- them available to every later migration and to the application.

ALTER TYPE user_type ADD VALUE IF NOT EXISTS 'org_admin';
ALTER TYPE user_type ADD VALUE IF NOT EXISTS 'teacher';
ALTER TYPE user_type ADD VALUE IF NOT EXISTS 'student';
