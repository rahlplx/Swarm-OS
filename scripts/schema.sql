CREATE TABLE IF NOT EXISTS build_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    compile_time_ms INTEGER,
    test_total INTEGER DEFAULT 0,
    test_passed INTEGER DEFAULT 0,
    test_failed INTEGER DEFAULT 0,
    test_skipped INTEGER DEFAULT 0,
    binary_size_bytes INTEGER,
    coverage_percent REAL,
    lint_errors INTEGER DEFAULT 0,
    stack TEXT CHECK(stack IN ('rust', 'react', 'python', 'all'))
);

CREATE TABLE IF NOT EXISTS session_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    query_count INTEGER DEFAULT 0,
    duration_seconds INTEGER,
    files_changed INTEGER DEFAULT 0,
    lines_added INTEGER DEFAULT 0,
    lines_removed INTEGER DEFAULT 0,
    day_number INTEGER
);

CREATE TABLE IF NOT EXISTS quality_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    pre_push_failures INTEGER DEFAULT 0,
    regression_count INTEGER DEFAULT 0,
    lint_fixes_applied INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS phase_progress (
    day_number INTEGER PRIMARY KEY,
    status TEXT CHECK(status IN ('planned', 'in_progress', 'completed', 'blocked')) DEFAULT 'planned',
    acceptance_tests_total INTEGER DEFAULT 0,
    acceptance_tests_passing INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Initialize Phase 0 days
INSERT OR IGNORE INTO phase_progress (day_number) VALUES
    (1),(2),(3),(4),(5),(6),(7),(8),(9),(10),
    (11),(12),(13),(14),(15),(16),(17),(18),(19),(20);
