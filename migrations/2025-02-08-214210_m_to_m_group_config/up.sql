CREATE TABLE m_to_m_group_configs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL
    CONSTRAINT fk_group_env
    REFERENCES group_environments (id)
    ON DELETE CASCADE,
    config_id INTEGER NOT NULL
    CONSTRAINT fk_config
    REFERENCES configurations (id)
    ON DELETE CASCADE
)

