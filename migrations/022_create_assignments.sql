-- Migration: 022_create_assignments
-- Description: Assignments a teacher sets within a classroom. May reference a
-- starter Derivative project the students begin from.

CREATE TABLE assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    classroom_id UUID NOT NULL REFERENCES classrooms(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    starter_project_id UUID REFERENCES user_projects(id) ON DELETE SET NULL,
    due_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assignments_classroom_id ON assignments(classroom_id);

COMMENT ON TABLE assignments IS 'Tasks set by a teacher within a classroom';
COMMENT ON COLUMN assignments.starter_project_id IS 'Optional Derivative project students start from';
