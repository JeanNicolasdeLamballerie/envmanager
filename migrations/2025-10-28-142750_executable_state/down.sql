-- SQLite doesn’t support DROP COLUMN directly,
-- so you’ll need to recreate the table without the column if you want a full rollback.

CREATE TABLE executables_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    executable TEXT NOT NULL
);

INSERT INTO executables_new (id, name, executable)
SELECT id, name, executable FROM executables;

DROP TABLE executables;
ALTER TABLE executables_new RENAME TO executables;
