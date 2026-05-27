-- Migration: 020_create_classrooms
-- Description: Classrooms owned by a teacher, scoped to one organization.

CREATE TABLE classrooms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    teacher_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_classrooms_organization_id ON classrooms(organization_id);
CREATE INDEX idx_classrooms_teacher_id ON classrooms(teacher_id);

COMMENT ON TABLE classrooms IS 'A teacher-owned classroom within an organization';
COMMENT ON COLUMN classrooms.teacher_id IS 'The teacher who owns/manages this classroom';
