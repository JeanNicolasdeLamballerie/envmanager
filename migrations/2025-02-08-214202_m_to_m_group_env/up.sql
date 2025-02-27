CREATE TABLE m_to_m_group_envs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL
    CONSTRAINT fk_group
    REFERENCES group_environments (id)
    ON DELETE CASCADE,
    env_id INTEGER NOT NULL
    CONSTRAINT fk_env
    REFERENCES environments (id)
    ON DELETE CASCADE
)

