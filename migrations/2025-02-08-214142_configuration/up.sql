create table configurations (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  exec INTEGER NOT NULL references executables(id)
)
