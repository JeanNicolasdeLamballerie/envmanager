// @generated automatically by Diesel CLI.

diesel::table! {
    configurations (id) {
        id -> Integer,
        name -> Text,
        exec -> Integer,
    }
}

diesel::table! {
    environments (id) {
        id -> Integer,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    executables (id) {
        id -> Integer,
        name -> Text,
        executable -> Text,
    }
}

diesel::table! {
    group_environments (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    m_to_m_group_configs (id) {
        id -> Integer,
        group_id -> Integer,
        config_id -> Integer,
    }
}

diesel::table! {
    m_to_m_group_envs (id) {
        id -> Integer,
        group_id -> Integer,
        env_id -> Integer,
    }
}

diesel::joinable!(configurations -> executables (exec));
diesel::joinable!(m_to_m_group_configs -> configurations (config_id));
diesel::joinable!(m_to_m_group_configs -> group_environments (group_id));
diesel::joinable!(m_to_m_group_envs -> environments (env_id));
diesel::joinable!(m_to_m_group_envs -> group_environments (group_id));

diesel::allow_tables_to_appear_in_same_query!(
    configurations,
    environments,
    executables,
    group_environments,
    m_to_m_group_configs,
    m_to_m_group_envs,
);
