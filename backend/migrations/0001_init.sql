-- Code Academy core schema.
-- Catalog tables (levels, departments, courses, lessons, ...) are shared content;
-- user tables (enrollments, activity, missions, ...) hold per-student state.

CREATE TABLE levels (
    id                 SERIAL PRIMARY KEY,
    year               SMALLINT NOT NULL UNIQUE CHECK (year BETWEEN 1 AND 6),
    slug               TEXT NOT NULL UNIQUE,
    name_fa            TEXT NOT NULL,
    name_en            TEXT NOT NULL,
    year_blurb         TEXT NOT NULL,
    description        TEXT NOT NULL,
    required_credits   INTEGER NOT NULL CHECK (required_credits > 0),
    course_count_label TEXT NOT NULL
);

CREATE TABLE departments (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name_fa     TEXT NOT NULL,
    name_en     TEXT NOT NULL,
    code_prefix TEXT NOT NULL UNIQUE,
    accent      TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL
);

CREATE TABLE courses (
    id            SERIAL PRIMARY KEY,
    slug          TEXT NOT NULL UNIQUE,
    code          TEXT NOT NULL UNIQUE,
    title         TEXT NOT NULL,
    summary       TEXT NOT NULL,
    department_id INTEGER NOT NULL REFERENCES departments(id),
    level_id      INTEGER NOT NULL REFERENCES levels(id),
    credits       INTEGER NOT NULL CHECK (credits >= 0),
    term_in_year  SMALLINT NOT NULL CHECK (term_in_year IN (1, 2)),
    kind          TEXT NOT NULL CHECK (kind IN ('course', 'project')),
    sort_order    INTEGER NOT NULL
);
CREATE INDEX courses_department_idx ON courses(department_id);
CREATE INDEX courses_level_idx ON courses(level_id);

CREATE TABLE course_prerequisites (
    course_id       INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    prerequisite_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    PRIMARY KEY (course_id, prerequisite_id),
    CHECK (course_id <> prerequisite_id)
);

CREATE TABLE lessons (
    id                SERIAL PRIMARY KEY,
    course_id         INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    sort_order        INTEGER NOT NULL,
    title             TEXT NOT NULL,
    estimated_minutes INTEGER NOT NULL CHECK (estimated_minutes > 0),
    what              TEXT NOT NULL,
    why               TEXT NOT NULL,
    simple_example    JSONB NOT NULL,
    real_example      JSONB NOT NULL,
    mistakes          TEXT[] NOT NULL,
    best_practices    TEXT[] NOT NULL,
    related_terms     TEXT[] NOT NULL,
    try_it            TEXT NOT NULL,
    UNIQUE (course_id, sort_order)
);

CREATE TABLE skills (
    id            SERIAL PRIMARY KEY,
    department_id INTEGER NOT NULL REFERENCES departments(id),
    course_id     INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    sort_order    INTEGER NOT NULL
);

CREATE TABLE dictionary_terms (
    id         SERIAL PRIMARY KEY,
    slug       TEXT NOT NULL UNIQUE,
    name_en    TEXT NOT NULL,
    name_fa    TEXT NOT NULL,
    category   TEXT NOT NULL,
    definition TEXT NOT NULL,
    example    TEXT NOT NULL DEFAULT '',
    related    TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE semesters (
    id           SERIAL PRIMARY KEY,
    number       SMALLINT NOT NULL UNIQUE CHECK (number BETWEEN 1 AND 12),
    year         SMALLINT NOT NULL,
    term_in_year SMALLINT NOT NULL CHECK (term_in_year IN (1, 2)),
    title_fa     TEXT NOT NULL
);

-- One academic advisor profile per department; the advice itself is rule-based.
CREATE TABLE advisors (
    id            SERIAL PRIMARY KEY,
    name          TEXT NOT NULL,
    title         TEXT NOT NULL,
    department_id INTEGER NOT NULL UNIQUE REFERENCES departments(id),
    bio           TEXT NOT NULL
);

CREATE SEQUENCE student_number_seq START 1;

CREATE TABLE users (
    id               SERIAL PRIMARY KEY,
    name             TEXT NOT NULL CHECK (char_length(name) BETWEEN 2 AND 60),
    email            TEXT NOT NULL UNIQUE CHECK (email = lower(email)),
    password_hash    TEXT NOT NULL,
    student_id       TEXT NOT NULL UNIQUE,
    current_level_id INTEGER NOT NULL REFERENCES levels(id),
    current_term     SMALLINT NOT NULL CHECK (current_term BETWEEN 1 AND 12),
    total_xp         INTEGER NOT NULL DEFAULT 0 CHECK (total_xp >= 0),
    streak_days      INTEGER NOT NULL DEFAULT 0,
    longest_streak   INTEGER NOT NULL DEFAULT 0,
    joined_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    graduated_at     TIMESTAMPTZ
);
CREATE INDEX users_level_xp_idx ON users(current_level_id, total_xp DESC);

CREATE TABLE enrollments (
    id           SERIAL PRIMARY KEY,
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_id    INTEGER NOT NULL REFERENCES courses(id),
    status       TEXT NOT NULL CHECK (status IN ('locked', 'enrolled', 'in_progress', 'done')),
    grade        DOUBLE PRECISION CHECK (grade BETWEEN 0 AND 20),
    semester_id  INTEGER REFERENCES semesters(id),
    started_at   TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    UNIQUE (user_id, course_id)
);

CREATE TABLE lesson_completions (
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    lesson_id    INTEGER NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    self_score   DOUBLE PRECISION NOT NULL CHECK (self_score BETWEEN 0 AND 20),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, lesson_id)
);

CREATE TABLE activity_logs (
    id            BIGSERIAL PRIMARY KEY,
    user_id       INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date          DATE NOT NULL,
    minutes_spent INTEGER NOT NULL CHECK (minutes_spent > 0),
    topic_tag     TEXT NOT NULL,
    course_id     INTEGER REFERENCES courses(id),
    source        TEXT NOT NULL CHECK (source IN ('lesson', 'study')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX activity_user_date_idx ON activity_logs(user_id, date);

CREATE TABLE daily_missions (
    id               SERIAL PRIMARY KEY,
    user_id          INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date             DATE NOT NULL,
    description      TEXT NOT NULL,
    course_id        INTEGER REFERENCES courses(id),
    target_minutes   INTEGER NOT NULL CHECK (target_minutes > 0),
    progress_minutes INTEGER NOT NULL DEFAULT 0,
    xp_reward        INTEGER NOT NULL,
    completed_at     TIMESTAMPTZ,
    UNIQUE (user_id, date)
);

CREATE TABLE check_ins (
    id                SERIAL PRIMARY KEY,
    user_id           INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    date              DATE NOT NULL,
    minutes_available INTEGER NOT NULL CHECK (minutes_available IN (10, 30, 60)),
    suggestion_kind   TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, date)
);

-- XP ledger. The unique key makes every award idempotent.
CREATE TABLE xp_events (
    id         BIGSERIAL PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount     INTEGER NOT NULL,
    reason     TEXT NOT NULL,
    ref        TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, reason, ref)
);

CREATE TABLE achievements (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    icon        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL
);

CREATE TABLE user_achievements (
    user_id        INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id INTEGER NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    unlocked_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, achievement_id)
);

CREATE TABLE certificates (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL
);

CREATE TABLE user_certificates (
    user_id        INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    certificate_id INTEGER NOT NULL REFERENCES certificates(id) ON DELETE CASCADE,
    serial_no      TEXT NOT NULL UNIQUE,
    issued_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, certificate_id)
);

CREATE TABLE announcements (
    id           SERIAL PRIMARY KEY,
    title        TEXT NOT NULL,
    body         TEXT NOT NULL,
    priority     TEXT NOT NULL CHECK (priority IN ('normal', 'important')),
    published_at TIMESTAMPTZ NOT NULL
);

-- Login sessions. Only a SHA-256 hash of the cookie token is stored.
CREATE TABLE sessions (
    id           BIGSERIAL PRIMARY KEY,
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash   TEXT NOT NULL UNIQUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sessions_user_idx ON sessions(user_id);

-- Failed/successful logins, for brute-force throttling.
CREATE TABLE login_attempts (
    id         BIGSERIAL PRIMARY KEY,
    email      TEXT NOT NULL,
    success    BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX login_attempts_email_idx ON login_attempts(email, created_at);

-- Editable academic content. Rule logic is keyed by `key`; texts live here.
CREATE TABLE promotion_rules (
    key         TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order  INTEGER NOT NULL
);

CREATE TABLE score_options (
    score      DOUBLE PRECISION PRIMARY KEY CHECK (score BETWEEN 0 AND 20),
    label      TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

-- Texts composed by the backend (missions, suggestions, advice). `{name}` placeholders.
CREATE TABLE message_templates (
    key  TEXT PRIMARY KEY,
    body TEXT NOT NULL
);

CREATE TABLE site_highlights (
    id         SERIAL PRIMARY KEY,
    section    TEXT NOT NULL,
    icon       TEXT NOT NULL,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

-- Hash of the seed bundle last applied, so content syncs only when it changes.
CREATE TABLE app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Transcript: one row per graded enrollment, placed in the semester it was taken.
CREATE VIEW transcript_entries AS
SELECT e.user_id,
       s.id           AS semester_id,
       s.number       AS semester_number,
       s.title_fa     AS semester_title,
       c.id           AS course_id,
       c.slug         AS course_slug,
       c.code         AS course_code,
       c.title        AS course_title,
       c.kind         AS course_kind,
       c.credits,
       e.grade,
       e.status
FROM enrollments e
JOIN courses c ON c.id = e.course_id
JOIN semesters s ON s.id = e.semester_id;
