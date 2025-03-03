use std::collections::{HashMap, HashSet};

use crate::schema;
use diesel::prelude::*;
#[derive(Queryable, Identifiable, Selectable, Associations, PartialEq, Clone, Debug)]
#[diesel(table_name = schema::configurations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(Executable, foreign_key = exec))]
pub struct Configuration {
    pub id: i32,
    pub name: String,
    pub exec: i32,
}

#[derive(PartialEq, Clone, Debug)]
pub struct LinkedConfiguration {
    pub configuration: Configuration,
    pub groups: Vec<LinkedGroups>,
}
impl LinkedConfiguration {
    pub fn get_environments(&self) -> HashMap<String, String> {
        let envs: HashMap<String, String> = self
            .groups
            .clone()
            .into_iter()
            .flat_map(|x| x.environments)
            .map(|y| (y.name, y.value))
            .collect();
        envs
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct LinkedGroups {
    pub group: GroupedEnvironment,
    pub environments: Vec<Environment>,
}
#[derive(Insertable, Clone, Copy, Debug)]
#[diesel(table_name = schema::m_to_m_group_configs)]
pub struct GroupCfgLinkInsert<'a> {
    pub group_id: &'a i32,
    pub config_id: &'a i32,
}
impl DbObject for GroupCfgLinkInsert<'_> {
    fn id(&self) -> i32 {
        *self.group_id
    }
}
impl DbObject for LinkedGroups {
    fn id(&self) -> i32 {
        self.group.id
    }
}
#[derive(Insertable, Clone, Copy)]
#[diesel(table_name = schema::m_to_m_group_envs)]
pub struct GroupEnvLinkInsert<'a> {
    pub group_id: &'a i32,
    pub env_id: &'a i32,
}
#[derive(Queryable, Identifiable, Selectable, PartialEq, Clone)]
#[diesel(table_name = schema::executables)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Executable {
    pub id: i32,
    pub name: String,
    pub executable: String,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Clone, Debug)]
#[diesel(table_name = schema::group_environments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GroupedEnvironment {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Selectable, Eq, Hash, PartialEq, Debug, Clone)]
#[diesel(table_name = schema::environments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Queryable, Identifiable, Associations, Selectable, PartialEq, Clone)]
#[diesel(table_name = schema::m_to_m_group_configs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(GroupedEnvironment, foreign_key = group_id))]
#[diesel(belongs_to(Configuration, foreign_key = config_id))]
pub struct GroupConfigLink {
    pub id: i32,
    pub group_id: i32,
    pub config_id: i32,
}

#[derive(Queryable, Identifiable, Selectable, Associations, PartialEq, Clone)]
#[diesel(table_name = schema::m_to_m_group_envs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(belongs_to(GroupedEnvironment, foreign_key = group_id))]
#[diesel(belongs_to(Environment, foreign_key = env_id))]
pub struct GroupEnvsLink {
    pub id: i32,
    pub group_id: i32,
    pub env_id: i32,
}

pub trait DbObject {
    fn id(&self) -> i32;
}

impl DbObject for Environment {
    fn id(&self) -> i32 {
        self.id
    }
}
impl DbObject for GroupEnvLinkInsert<'_> {
    fn id(&self) -> i32 {
        *self.env_id
    }
}
impl DbObject for i32 {
    fn id(&self) -> i32 {
        *self
    }
}
pub type Added<T> = Vec<T>;
pub type Removed<T> = Vec<T>;
//
// pub fn hashset_comparison<T, U>(old_list: &[T], new_list: &[U]) -> (Added, Removed)
// where
//     T: DbObject,
//     U: DbObject,
// {
//     let old_ids: HashSet<i32> = old_list.iter().map(|s| s.id()).collect();
//     let new_ids: HashSet<i32> = new_list.iter().map(|s| s.id()).collect();
//
//     let added: Vec<i32> = new_ids.difference(&old_ids).copied().collect();
//     let removed: Vec<i32> = old_ids.difference(&new_ids).copied().collect();
//
//     (added, removed)
// }
pub fn hashset_comparison<'a, T, U>(
    old_list: &'a [T],
    new_list: &'a [U],
) -> (Vec<&'a U>, Vec<&'a T>)
where
    T: DbObject,
    U: DbObject,
{
    let old_ids: HashSet<i32> = old_list.iter().map(|s| s.id()).collect();
    let new_ids: HashSet<i32> = new_list.iter().map(|s| s.id()).collect();

    // Get added structs from new_list
    let added: Vec<&U> = new_list
        .iter()
        .filter(|s| !old_ids.contains(&s.id()))
        .collect();

    // Get removed structs from old_list
    let removed: Vec<&T> = old_list
        .iter()
        .filter(|s| !new_ids.contains(&s.id()))
        .collect();

    (added, removed)
}
