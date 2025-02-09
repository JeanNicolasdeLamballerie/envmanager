CREATE TABLE m_to_m_group_envs (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  group_id INTEGER NOT NULL references group_environments(id),
  env_id INTEGER NOT NULL references environments(id)
)
