CREATE TABLE m_to_m_group_configs (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  group_id INTEGER NOT NULL references group_environments(id),
  config_id INTEGER NOT NULL references configurations(id)
)
