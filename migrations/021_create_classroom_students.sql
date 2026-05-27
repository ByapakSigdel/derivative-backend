-- Migration: 021_create_classroom_students
-- Description: Enrollment join table — which students belong to which classroom.

CREATE TABLE classroom_students (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    classroom_id UUID NOT NULL REFERENCES classrooms(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (classroom_id, student_id)
);

CREATE INDEX idx_classroom_students_classroom_id ON classroom_students(classroom_id);
CREATE INDEX idx_classroom_students_student_id ON classroom_students(student_id);

COMMENT ON TABLE classroom_students IS 'Student enrollment in classrooms (many-to-many)';
