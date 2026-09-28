# کد آکادمی — Code Academy

A university-style learning platform for software engineering, Dart and Flutter. Students move through a six-year path, from Junior (جونیور) to Senior+ (سینیور پلاس). Along the way they take courses, earn credits, get grades out of 20, deliver a project each year, collect XP and achievements, and finally graduate.

| Layer    | Stack |
|----------|-------|
| Frontend | React 19, TypeScript, Vite, React Router, TanStack Query, Vitest + Testing Library |
| Backend  | Rust, axum 0.8, sqlx 0.8 (parameterized queries), argon2, tokio |
| Database | PostgreSQL 16 (docker compose) |

## Run locally

```bash
make db      # PostgreSQL on localhost:5435
make api     # http://127.0.0.1:8080/api/v1 (runs migrations and syncs content on every start)
make web     # http://localhost:5173
```

Open http://localhost:5173 and register (ثبت‌نام). Every new student starts in year 1, term 1. The database contains no demo or sample users; everything shown comes from real accounts and their activity.

Configuration lives in `backend/.env`, with a template in `backend/.env.example`:

- `DATABASE_URL`, `BIND_ADDR`, `CORS_ORIGINS`, `APP_TIMEZONE`
- `COOKIE_SECURE`: set it to `true` when you serve over HTTPS.

## Accounts and security

- **Accounts:** register and log in with email and password. Passwords are hashed with Argon2id. Sessions are random 256-bit tokens in an HttpOnly, SameSite=Lax cookie, and the database stores only their SHA-256 hash. Sessions last 30 days.
- **Login throttling:** after 5 failed logins, an email is locked for 15 minutes. When the email doesn't exist, the server still runs a password check, so response timing doesn't reveal whether an email is registered.
- **Password change:** changing your password signs out every other device.
- **Input handling:** all input is validated on the server. SQL uses bound parameters only. Internal errors are logged and never returned to the client.
- **HTTP hardening:** request bodies are capped at 64 KB, responses send `nosniff`, `DENY` framing and `no-store` headers, and CORS is an allow-list.

## Content (everything is in the database)

The academic content is authored in `backend/seed/*.json`:

- the catalog, lessons and glossary;
- promotion rules and self-assessment options;
- the message templates for missions, check-in suggestions and the advisor;
- home-page highlights, achievements, certificates and announcements.

On startup the API compares a hash of these files with the database and upserts anything that changed, keyed by slug, year or lesson order. Student progress is kept, and course statuses and grades are recalculated if lessons changed. `make sync` applies edits without starting the server.

Content rules are in [backend/seed/STYLE_GUIDE.md](backend/seed/STYLE_GUIDE.md):

- fluent Persian, Iranian examples, short sections;
- exactly 3 common mistakes and 3 best practices per lesson;
- every Dart example must pass the Dart analyzer.

Checks:

```bash
make check                               # validators + lint + typecheck
make content-check VERIFY=<flutter-dir>  # dart analyze every code example
```

For `content-check`, create the Flutter project first with `flutter create --empty verify`, then `flutter pub add` the packages listed in STYLE_GUIDE.md.

Current content:

- 35 courses (29 courses and 6 projects), 99 credits in total;
- 129 lessons, each with the 8-part structure;
- 46 glossary terms;
- 225 code examples, all analyzer-clean.

## Tests

```bash
make test    # 67 unit tests + 17 API tests + 23 frontend tests
```

The API tests use `#[sqlx::test]`, which creates a fresh database for each test.

## Architecture

```
backend/src
├── api/        routes (/api/v1), handlers, cookie handling
├── auth.rs     session-cookie extractor (CurrentUser)
├── services/   use cases: auth, dashboard, lessons/progression, engagement, records, stats…
├── domain/     pure rules, unit-tested: gpa, xp, streak, progression, course_status,
│               mission, checkin, advisor, achievements, ranking, templates, jalali
├── repo/       data access (sqlx)
└── seed/       idempotent content sync
frontend/src
├── api/        typed client, DTO types, React Query hooks (session, auth, data)
├── components/ layout (sidebar ↔ mobile drawer), RequireAuth, states, accordion…
├── pages/      one file per screen (incl. login, register, account)
└── lib/        Persian digits, Jalali dates, theme
```

### Business rules

- **GPA:** Σ(grade × credits) / Σ credits. A course passes at 14 or above.
- **Course grade:** the mean of the lesson self-assessment scores. A failed course can be redone by re-scoring its lessons. Dependent courses unlock only after their prerequisites are *passed*.
- **XP:** 25 per lesson, 50 per daily mission, 150 per year project and 1000 for the graduation project. XP is recorded in an idempotent ledger.
- **Streak:** consecutive active days (Asia/Tehran). A full missed day resets it to 0.
- **Promotion:** every course of the year must be passed with at least 14, the year project must be delivered, and no course may be more than one term behind.

## Curriculum note

The course list in the spec adds up to 77 credits, but the level table requires 99 (15/16/18/18/17/15). To make every year match its credits and course count, the catalog:

- adds 7 courses: نرم-۲۱۰, نرم-۵۲۰, فلاتر-۵۲۰, نرم-۶۲۰, دارت-۶۰۱, فلاتر-۶۱۰, حرفه-۶۰۰;
- adds a year project for each of years 1–5: دارت-۱۹۰, فلاتر-۲۹۰, فلاتر-۳۹۰, نرم-۴۹۰, نرم-۵۹۰. فلاتر-۳۹۰ is assessment-only and carries 0 credits;
- keeps نرم-۶۱۰ as the graduation project.
