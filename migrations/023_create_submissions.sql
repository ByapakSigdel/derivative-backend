-- Migration: 023_create_submissions
-- Description: One row per (assignment, student). Holds the student's work (a
-- Derivative project) and its review lifecycle.
--
-- Status lifecycle:
--   in_progress -> student is still working; the teacher CANNOT see the work.
--   submitted   -> student marked it done; now visible to the teacher.
--   reviewed    -> teacher has graded / given feedback.
--
-- The "teacher only sees submitted work" rule is enforced in the query layer
-- by filtering on status <> 'in_progress'.

CREATE TABLE submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    project_id UUID REFERENCES user_projects(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress'
        CHECK (status IN ('in_progress', 'submitted', 'reviewed')),
    student_note TEXT,
    grade VARCHAR(50),
    feedback TEXT,
    submitted_at TIMESTAMPTZ,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (assignment_id, student_id)
);

CREATE INDEX idx_submissions_assignment_id ON submissions(assignment_id);
CREATE INDEX idx_submissions_student_id ON submissions(student_id);
CREATE INDEX idx_submissions_status ON submissions(status);

COMMENT ON TABLE submissions IS 'A student''s work on an assignment, plus its review state';
COMMENT ON COLUMN submissions.status IS 'in_progress | submitted | reviewed';
COMMENT ON COLUMN submissions.project_id IS 'The student''s Derivative project for this assignment';
