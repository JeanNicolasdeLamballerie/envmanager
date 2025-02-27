use crate::{models::*, schema};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use directories::ProjectDirs;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
type DbResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn establish_connection() -> SqliteConnection {
    let db_path = get_db_path();

    let database_url: &str = db_path.to_str().unwrap();
    println!("db url : {}", database_url);
    let mut conn = SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to database url..."));
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    conn
}

fn get_db_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Dekharen", "vimming") {
        let local = proj_dirs.data_local_dir();
        match local.try_exists() {
            Ok(exists) => {
                if !exists {
                    std::fs::create_dir_all(local).unwrap();
                }
            }
            Err(err) => panic!(
                "An error occured while acquiring the local directories : {}",
                err
            ),
        }
        local.join("vimming_storage.db")
    } else {
        panic!("Could not determine database path");
    }
}

pub fn new_configuration(
    conn: &mut SqliteConnection,
    name: &str,
    id: &i32,
) -> DbResult<Configuration> {
    let cfg = diesel::insert_into(schema::configurations::table)
        .values((
            schema::configurations::name.eq(name),
            schema::configurations::exec.eq(id),
        ))
        .returning(Configuration::as_returning())
        .get_result(conn)?;
    Ok(cfg)
}

pub fn new_executable(conn: &mut SqliteConnection, name: &str, exe: &str) -> DbResult<Executable> {
    let exe = diesel::insert_into(schema::executables::table)
        .values((
            schema::executables::name.eq(name),
            schema::executables::executable.eq(exe),
        ))
        .returning(Executable::as_returning())
        .get_result(conn)?;
    Ok(exe)
}

pub fn new_grouped_envs(conn: &mut SqliteConnection, name: &str) -> DbResult<GroupedEnvironment> {
    let grpenvs = diesel::insert_into(schema::group_environments::table)
        .values(schema::group_environments::name.eq(name))
        .returning(GroupedEnvironment::as_returning())
        .get_result(conn)?;
    Ok(grpenvs)
}
pub fn update_group(
    conn: &mut SqliteConnection,
    id: &i32,
    name: &str,
) -> DbResult<GroupedEnvironment> {
    let group = diesel::update(
        schema::group_environments::table.filter(schema::group_environments::id.eq(id)),
    )
    .set((schema::group_environments::name.eq(name),))
    .returning(GroupedEnvironment::as_returning())
    .get_result(conn)?;
    Ok(group)
}
pub fn update_env(
    conn: &mut SqliteConnection,
    id: &i32,
    name: &str,
    value: &str,
) -> DbResult<Environment> {
    let env = diesel::update(schema::environments::table.filter(schema::environments::id.eq(id)))
        .set((
            schema::environments::name.eq(name),
            schema::environments::value.eq(value),
        ))
        .returning(Environment::as_returning())
        .get_result(conn)?;
    Ok(env)
}
pub fn update_exec(
    conn: &mut SqliteConnection,
    id: &i32,
    name: &str,
    exec: &str,
) -> DbResult<Executable> {
    let exec = diesel::update(schema::executables::table.filter(schema::executables::id.eq(id)))
        .set((
            schema::executables::name.eq(name),
            schema::executables::executable.eq(exec),
        ))
        .returning(Executable::as_returning())
        .get_result(conn)?;
    Ok(exec)
}
pub fn new_env(conn: &mut SqliteConnection, name: &str, value: &str) -> DbResult<Environment> {
    let env = diesel::insert_into(schema::environments::table)
        .values((
            schema::environments::name.eq(name),
            schema::environments::value.eq(value),
        ))
        .returning(Environment::as_returning())
        .get_result(conn)?;
    Ok(env)
}
pub fn delete_env(conn: &mut SqliteConnection, id: &i32) -> DbResult<usize> {
    let env = diesel::delete(
        schema::environments::table.filter(schema::environments::columns::id.eq(id)),
    )
    .execute(conn)?;
    Ok(env)
}

pub fn new_linked_groups_cfg(
    conn: &mut SqliteConnection,
    ids: &[GroupCfgLinkInsert],
) -> DbResult<usize> {
    let m_to_m = diesel::insert_into(schema::m_to_m_group_configs::table)
        .values(ids)
        .execute(conn)?;
    Ok(m_to_m)
}

pub fn delete_linked_group_envs(
    conn: &mut SqliteConnection,
    ids: &[i32],
    group_id: i32,
) -> DbResult<usize> {
    let m_to_m = diesel::delete(
        schema::m_to_m_group_envs::table.filter(
            schema::m_to_m_group_envs::env_id
                .eq_any(ids)
                .and(schema::m_to_m_group_envs::group_id.eq(group_id)),
        ),
    )
    .execute(conn)?;
    Ok(m_to_m)
}

pub fn new_linked_group_envs(
    conn: &mut SqliteConnection,
    ids: &[GroupEnvLinkInsert],
) -> DbResult<usize> {
    let m_to_m = diesel::insert_into(schema::m_to_m_group_envs::table)
        .values(ids)
        .execute(conn)?;
    Ok(m_to_m)
}

pub fn get_linked_group_env(
    conn: &mut SqliteConnection,
    group_id: &i32,
) -> DbResult<Vec<GroupEnvsLink>> {
    use schema::m_to_m_group_envs::{self as rep, table};
    let res = table.filter(rep::dsl::group_id.eq(group_id)).load(conn)?;
    Ok(res)
}
pub fn get_multiple_linked_group_env(
    conn: &mut SqliteConnection,
    group_ids: &[i32],
) -> DbResult<Vec<GroupEnvsLink>> {
    use schema::m_to_m_group_envs::{self as rep, table};
    let res = table
        .filter(rep::dsl::group_id.eq_any(group_ids))
        .load(conn)?;
    Ok(res)
}

pub fn get_linked_group_cfg(
    conn: &mut SqliteConnection,
    cfg_id: &i32,
) -> DbResult<Vec<GroupConfigLink>> {
    use schema::m_to_m_group_configs::{self as rep, table};
    let res = table.filter(rep::dsl::group_id.eq(cfg_id)).load(conn)?;
    Ok(res)
}

pub fn get_all(conn: &mut SqliteConnection) -> DbResult<Vec<LinkedConfiguration>> {
    use schema::configurations::table;
    let res: Vec<(Configuration, Executable)> = table
        .inner_join(schema::executables::table)
        .select((Configuration::as_select(), Executable::as_select()))
        .load(conn)?;
    let cfgs: Vec<Configuration> = res.iter().map(|pair| pair.0.clone()).collect();
    let linker: Vec<(GroupConfigLink, Option<GroupedEnvironment>)> =
        GroupConfigLink::belonging_to(&cfgs)
            .left_outer_join(schema::group_environments::table)
            .select((
                GroupConfigLink::as_select(),
                schema::group_environments::all_columns.nullable(),
            ))
            .load(conn)?;
    let configurations_with_groups: Vec<(Configuration, Vec<GroupedEnvironment>)> = linker
        .grouped_by(&cfgs)
        .into_iter()
        .zip(cfgs)
        .map(|(links, config)| {
            (
                config,
                links
                    .into_iter()
                    .filter_map(|(_, group)| group)
                    .collect::<Vec<GroupedEnvironment>>(),
            )
        })
        .collect();
    let mut mapped: HashMap<i32, GroupedEnvironment> = HashMap::new();
    for (_, groups) in configurations_with_groups.iter() {
        for group in groups {
            mapped.entry(group.id).or_insert_with(|| group.clone());
        }
    }
    let all_grouped: Vec<GroupedEnvironment> = mapped.into_values().collect();
    let envs: Vec<(GroupEnvsLink, Option<Environment>)> = GroupEnvsLink::belonging_to(&all_grouped)
        .left_outer_join(schema::environments::table)
        .select((
            GroupEnvsLink::as_select(),
            schema::environments::all_columns.nullable(),
        ))
        .load(conn)?;

    let all_grouped_environments: HashMap<i32, (GroupedEnvironment, Vec<Environment>)> = envs
        .grouped_by(&all_grouped)
        .into_iter()
        .zip(all_grouped)
        .map(|(links, grouped)| {
            (
                grouped.id,
                (
                    grouped,
                    links
                        .into_iter()
                        .filter_map(|(_, env)| env)
                        .collect::<Vec<Environment>>(),
                ),
            )
        })
        .collect();
    let results: Vec<LinkedConfiguration> = configurations_with_groups
        .iter()
        .map(|(cfg, children)| {
            let new_groups = children
                .iter()
                .map(|group| {
                    let envs = match all_grouped_environments.get(&group.id) {
                        Some(tuple) => tuple.1.clone(),
                        None => vec![],
                    };
                    LinkedGroups {
                        group: group.to_owned(),
                        environments: envs,
                    }
                })
                .collect();
            LinkedConfiguration {
                configuration: cfg.to_owned(),
                groups: new_groups,
            }
        })
        .collect();
    Ok(results)
}
pub fn get_groups_for_config(
    conn: &mut SqliteConnection,
    config: &Configuration,
) -> DbResult<Vec<GroupedEnvironment>> {
    let groups: Vec<GroupedEnvironment> = GroupConfigLink::belonging_to(config)
        .inner_join(schema::group_environments::table)
        .select(GroupedEnvironment::as_select())
        .load(conn)?;
    Ok(groups)
}
pub fn get_envs_for_group(
    conn: &mut SqliteConnection,
    group: &GroupedEnvironment,
) -> DbResult<Vec<Environment>> {
    let envs: Vec<Environment> = GroupEnvsLink::belonging_to(group)
        .inner_join(schema::environments::table)
        .select(Environment::as_select())
        .load(conn)?;
    Ok(envs)
}

pub fn get_multiple_linked_group_cfg_shaped(
    conn: &mut SqliteConnection,
    cfg_ids: &[i32],
) -> DbResult<Vec<GroupEnvsLink>> {
    use schema::m_to_m_group_configs::{self as rep, table};
    let res = table
        .filter(rep::dsl::config_id.eq_any(cfg_ids))
        .load(conn)?;
    Ok(res)
}

pub fn get_groups(conn: &mut SqliteConnection) -> DbResult<Vec<GroupedEnvironment>> {
    use schema::group_environments::table;
    let res = table.load(conn)?;
    Ok(res)
}
pub fn get_configurations(conn: &mut SqliteConnection) -> DbResult<Vec<Configuration>> {
    use schema::configurations::table;
    //TODO can add a limit/offset if needed
    let res = table.load(conn)?;
    Ok(res)
}
pub fn get_environments(conn: &mut SqliteConnection) -> DbResult<Vec<Environment>> {
    use schema::environments::table;
    let res = table.load(conn)?;
    Ok(res)
}
pub fn get_executables(conn: &mut SqliteConnection) -> DbResult<Vec<Executable>> {
    use schema::executables::table;
    let res = table.load(conn)?;
    Ok(res)
}
pub fn get_environment_variables_by_id(
    conn: &mut SqliteConnection,
    ids: &[i32],
) -> DbResult<Vec<Environment>> {
    use schema::environments::{self as rep, table};
    let res = table.filter(rep::dsl::id.eq_any(ids)).load(conn)?;
    Ok(res)
}
