use std::collections::HashMap;

use diesel::SqliteConnection;
use eframe::{App, CreationContext};
use egui::{Color32, ComboBox, Frame, Grid, Id, Layout, Modal, RichText, ScrollArea, Stroke, Ui};

use crate::{
    database::{
        delete_env, delete_linked_groups_cfg, establish_connection, get_all, get_environments,
        get_envs_for_group, get_executables, get_groups, update_configuration, update_env,
        update_group,
    },
    models::{
        hashset_comparison, DbObject as _, Environment, Executable, GroupCfgLinkInsert,
        GroupEnvLinkInsert, GroupedEnvironment, LinkedConfiguration, LinkedGroups,
    },
};

pub fn show() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]), // .with_icon(
        //     // NOTE: Adding an icon is optional
        //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
        //         .expect("Failed to load icon"),
        // )
        ..Default::default()
    };
    eframe::run_native(
        "Configuration Manager",
        native_options,
        Box::new(|cc| Ok(Box::new(ConfigurationManager::new(cc)))),
    )
}

#[derive(Default)]
struct EnvFields {
    name: String,
    value: String,
    tip: bool,
}

#[derive(Default)]
struct GroupFields {
    group_name: String,
    tip: bool,
}
#[derive(Default)]
struct ExecutableFields {
    name: String,
    exec: String,
    tip: bool,
}
#[derive(Default)]
struct Fields {
    configuration_name: String,
    executable: ExecutableFields,
    group: GroupFields,
    env: EnvFields,
    tip: bool,
}
#[derive(Default)]
struct EditableFields {
    configuration_fields: Fields,
}
type DbId = i32;

#[derive(Default)]
enum FieldState<T> {
    #[default]
    Create,
    Edit(T),
}

#[derive(Default)]
struct EditableGroup {
    checkboxes: HashMap<DbId, bool>,
    env_checkboxes: HashMap<DbId, bool>,
}

#[derive(Default)]
struct EditableConfiguration {
    exec: EditableExecutable,
    groups: EditableGroup,
}
#[derive(Default)]
struct EditableExecutable {
    id: i32,
}

#[derive(Default)]
struct ModalState<T> {
    field: FieldState<T>,
    open: bool,
}
#[derive(Default)]
struct Modals {
    main_state: ModalState<(DbId, String, Vec<LinkedGroups>, DbId)>,
    exec_state: ModalState<(DbId, String, String)>,
    group_state: ModalState<(DbId, String, Vec<i32>)>,
    env_state: ModalState<(DbId, String, String)>,
    show_env: ShowEnvModal,
}
#[derive(Default)]
struct ShowEnvModal {
    show: bool,
    group_id: Option<GroupedEnvironment>,
    //TODO could optimize with a lifetime and a slice,
    //but we'd have to pollute Modals with it too.
    envs: Vec<Environment>,
}

struct ConfigurationManager {
    conn: SqliteConnection,
    configurations: Vec<LinkedConfiguration>,
    groups: HashMap<DbId, GroupedEnvironment>,
    executables: HashMap<DbId, Executable>,
    environment_variables: HashMap<DbId, Environment>,
    fields: EditableFields,
    editable: EditableConfiguration,
    modals: Modals,
}

impl App for ConfigurationManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.all_styles_mut(|style| {
            style.spacing.button_padding = [7., 8.].into();
        });
        egui::TopBottomPanel::top("MAIN_CONTROLS").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let txt = RichText::new("Configuration Manager").heading();
                ui.label(txt);
                if ui.button("Create Configuration").clicked() {
                    self.modals.main_state.open = true;
                }

                if ui.button("print").clicked() {
                    dbg!(self.configurations.clone());
                }
                ui.add_space(10.);
                if self.modals.main_state.open {
                    self.main_modal(ui);
                }
                if self.modals.group_state.open {
                    self.group_state_modal(ui);
                }
                if self.modals.exec_state.open {
                    self.exec_state_modal(ui);
                }
                if self.modals.env_state.open {
                    self.env_state_modal(ui);
                }
                if self.modals.show_env.show {
                    self.show_env_modal(ui);
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.display_configurations(ui);
        });
    }
}

impl ConfigurationManager {
    pub fn new(_cc: &CreationContext) -> Self {
        let mut conn = establish_connection();
        //TODO error handling
        let cfgs = get_all(&mut conn).unwrap();
        let groups = get_groups(&mut conn).unwrap();
        let envs = get_environments(&mut conn).unwrap();
        let envs: HashMap<i32, Environment> = envs.into_iter().map(|el| (el.id, el)).collect();
        let execs = get_executables(&mut conn).unwrap();
        let executables = execs.into_iter().map(|el| (el.id, el)).collect();
        let mut editable = EditableConfiguration::default();
        let fields = EditableFields::default();
        let modals = Modals::default();
        let groups: HashMap<i32, GroupedEnvironment> =
            groups.into_iter().map(|el| (el.id, el)).collect();
        editable.groups.checkboxes = groups
            .clone()
            .into_iter()
            .map(|pair| (pair.0, false))
            .collect();
        editable.groups.env_checkboxes = envs
            .clone()
            .into_iter()
            .map(|pair| (pair.0, false))
            .collect();
        Self {
            conn,
            editable,
            executables,
            configurations: cfgs,
            groups,
            environment_variables: envs,
            fields,
            modals,
        }
    }
    fn reload_group_checkboxes(&mut self) {
        self.editable.groups.checkboxes = self
            .groups
            .clone()
            .into_iter()
            .map(|pair| (pair.0, false))
            .collect();
    }
    fn reload_env_checkboxes(&mut self) {
        self.editable.groups.env_checkboxes = self
            .environment_variables
            .clone()
            .into_iter()
            .map(|pair| (pair.0, false))
            .collect();
    }
    fn main_modal(&mut self, ui: &mut Ui) {
        let modal_id = Id::new("CONFIG_FIELD_2");
        let modal = egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let title = match self.modals.main_state.field {
                FieldState::Create => "CREATE MODE",
                FieldState::Edit(_) => "EDIT MODE",
            };
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();
            ui.label("Configuration name :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.configuration_name);
            let selected = match self.executables.get(&self.editable.exec.id) {
                Some(exec) => &exec.name,
                None => "No executable selected",
            };
            ComboBox::from_label("Executable")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for executable in self.executables.iter() {
                        ui.horizontal(|ui| {
                            let exec = executable.1;
                            if ui
                                .add(
                                    egui::Button::new("EDIT")
                                        .corner_radius(4.)
                                        .fill(Color32::BLACK),
                                )
                                .clicked()
                            {
                                self.fields.configuration_fields.executable.name =
                                    exec.name.clone();
                                self.fields.configuration_fields.executable.exec =
                                    exec.executable.clone();
                                self.modals.exec_state.field = FieldState::Edit((
                                    exec.id,
                                    exec.name.clone(),
                                    exec.executable.clone(),
                                ));
                                self.modals.exec_state.open = true;
                            };
                            ui.selectable_value(
                                &mut self.editable.exec.id,
                                *executable.0,
                                &executable.1.name,
                            );
                        });
                    }
                    ui.separator();
                    if ui.button("Add new executable").clicked() {
                        self.modals.exec_state.open = true;
                    };
                });

            ComboBox::from_label("Groups")
                .selected_text("Select your environment groups")
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show_ui(ui, |ui| {
                    for group in self.groups.iter() {
                        let checked = self.editable.groups.checkboxes.get_mut(group.0).expect(
                            "A checkbox shouldn't point to a dangling id.
                                                This error happened because the list of groups 
                                                and the list of checkboxes got out of sync.",
                        );
                        ui.horizontal(|ui| {
                            let envs = get_envs_for_group(&mut self.conn, group.1).unwrap();
                            if ui.button("show").clicked() {
                                self.modals.show_env.group_id = Some(group.1.clone());
                                self.modals.show_env.envs = envs.clone();
                                self.modals.show_env.show = true;
                            };
                            ui.separator();
                            if ui.button("Edit").clicked() {
                                let mut environments: HashMap<i32, bool> = self
                                    .environment_variables
                                    .clone()
                                    .into_iter()
                                    .map(|x| (x.0, false))
                                    .collect();
                                for env in envs.iter() {
                                    environments.insert(env.id, true);
                                }
                                self.fields.configuration_fields.group.group_name =
                                    group.1.name.clone();
                                self.editable.groups.env_checkboxes = environments;
                                self.modals.group_state.field = FieldState::Edit((
                                    group.1.id,
                                    group.1.name.clone(),
                                    envs.iter().map(|x| x.id).collect(),
                                ));
                                self.modals.group_state.open = true;
                            }
                            ui.separator();
                            ui.checkbox(checked, &group.1.name);
                            ui.add_space(12.);
                        });
                    }
                    ui.separator();

                    if ui.button("Add new group").clicked() {
                        self.modals.group_state.open = true;
                    };
                });
            if self.fields.configuration_fields.tip {
                ui.label(
                    "You at the very least need to set a configuration name and select an \
                     executable.",
                );
            }
            if ui.button("Save").clicked() {
                if self
                    .fields
                    .configuration_fields
                    .configuration_name
                    .is_empty()
                    || self.editable.exec.id == 0
                {
                    self.fields.configuration_fields.tip = true;
                } else {
                    self.fields.configuration_fields.tip = false;
                    match &self.modals.main_state.field {
                        FieldState::Edit(previous) => {
                            if previous.1 != self.fields.configuration_fields.configuration_name
                                || self.editable.exec.id != previous.3
                            {
                                update_configuration(
                                    &mut self.conn,
                                    &previous.0,
                                    &self.fields.configuration_fields.configuration_name,
                                    &self.editable.exec.id,
                                )
                                .unwrap();
                            }
                            let ids: Vec<GroupCfgLinkInsert> = self
                                .editable
                                .groups
                                .checkboxes
                                .iter()
                                .filter_map(|x| {
                                    if *x.1 {
                                        Some(GroupCfgLinkInsert {
                                            group_id: x.0,
                                            config_id: &previous.0,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let (added, removed) = hashset_comparison(&previous.2, &ids);
                            dbg!(&added, &removed);
                            if !removed.is_empty() {
                                let remove: Vec<i32> = removed.iter().map(|x| x.id()).collect();
                                delete_linked_groups_cfg(&mut self.conn, &remove, previous.0)
                                    .unwrap();
                                // delete_
                            }
                            if !added.is_empty() {
                                let v: Vec<GroupCfgLinkInsert> =
                                    added.into_iter().copied().collect();
                                crate::database::new_linked_groups_cfg(&mut self.conn, &v).unwrap();
                            }
                        }
                        FieldState::Create => {
                            let config = crate::database::new_configuration(
                                &mut self.conn,
                                &self.fields.configuration_fields.configuration_name,
                                &self.editable.exec.id,
                            )
                            .unwrap();
                            let ids: Vec<GroupCfgLinkInsert> = self
                                .editable
                                .groups
                                .checkboxes
                                .iter()
                                .filter_map(|x| {
                                    if *x.1 {
                                        Some(GroupCfgLinkInsert {
                                            group_id: x.0,
                                            config_id: &config.id,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            crate::database::new_linked_groups_cfg(&mut self.conn, &ids).unwrap();
                        }
                    };
                    self.reload();
                    self.reload_group_checkboxes();
                    self.modals.main_state = Default::default();
                }
            }
        });
        if modal.should_close() {
            self.reload_group_checkboxes();
            self.modals.main_state = Default::default();
        }
    }
    fn exec_state_modal(&mut self, ui: &mut Ui) {
        let modal_id = Id::new("CONFIG_EXEC_CREATOR");
        let sub_modal = egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let title = match self.modals.exec_state.field {
                FieldState::Create => "CREATE MODE",
                FieldState::Edit(_) => "EDIT MODE",
            };
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();
            ui.label("Name :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.executable.name);
            ui.label("Executable name :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.executable.exec);
            if self.fields.configuration_fields.executable.tip {
                ui.separator();
                ui.label(
                    "You need to set both an executable's friendly name and what command the \
                     service should call.",
                );
            }
            ui.separator();
            if ui.button("Save").clicked() {
                if self.fields.configuration_fields.executable.name.is_empty()
                    || self.fields.configuration_fields.executable.exec.is_empty()
                {
                    self.fields.configuration_fields.executable.tip = true;
                } else {
                    self.fields.configuration_fields.executable.tip = false;
                    match &self.modals.exec_state.field {
                        FieldState::Edit(edit) => {
                            let id = &edit.0;
                            let name: &str = &edit.1;
                            let exec: &str = &edit.2;
                            if name != self.fields.configuration_fields.executable.name
                                || exec != self.fields.configuration_fields.executable.exec
                            {
                                crate::database::update_exec(&mut self.conn, id, name, exec)
                                    .unwrap();
                            }
                        }
                        FieldState::Create => {
                            let exec = crate::database::new_executable(
                                &mut self.conn,
                                &self.fields.configuration_fields.executable.name,
                                &self.fields.configuration_fields.executable.exec,
                            )
                            .unwrap();
                            self.editable.exec.id = exec.id;
                        }
                    };
                    self.reload();
                    self.modals.exec_state = Default::default();
                }
            }
        });
        if sub_modal.should_close() {
            self.modals.exec_state = Default::default();
        }
    }
    fn group_state_modal(&mut self, ui: &mut Ui) {
        let modal_id = Id::new("CONFIG_GROUP_CREATOR");
        let sub_modal = egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let title = match self.modals.group_state.field {
                FieldState::Create => "CREATE MODE",
                FieldState::Edit(_) => "EDIT MODE",
            };
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();
            ui.label("Name :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.group.group_name);
            ComboBox::from_label("Environments within group")
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .selected_text("Select Environments")
                .show_ui(ui, |ui| {
                    for env in self.environment_variables.iter() {
                        let checked = self.editable.groups.env_checkboxes.get_mut(env.0).expect(
                            "A checkbox shouldn't point to a dangling id.
                                                This error happened because the list of envs 
                                                and the list of checkboxes got out of sync.",
                        );
                        let checkbox_text = String::new() + &env.1.name + " : " + &env.1.value;
                        ui.checkbox(checked, &checkbox_text);
                    }
                    ui.separator();

                    if ui.button("Add new environment variable").clicked() {
                        self.modals.env_state.open = true;
                    };
                });
            if self.fields.configuration_fields.group.tip {
                ui.label("You need to set a group name.");
            }

            if ui.button("Save").clicked() {
                if self.fields.configuration_fields.group.group_name.is_empty() {
                    self.fields.configuration_fields.group.tip = true;
                } else {
                    match &self.modals.group_state.field {
                        FieldState::Create => {
                            self.fields.configuration_fields.group.tip = false;
                            let group = crate::database::new_grouped_envs(
                                &mut self.conn,
                                &self.fields.configuration_fields.group.group_name,
                            )
                            .unwrap();
                            let ids: Vec<GroupEnvLinkInsert> = self
                                .editable
                                .groups
                                .env_checkboxes
                                .iter()
                                .filter_map(|x| {
                                    if *x.1 {
                                        Some(GroupEnvLinkInsert {
                                            env_id: x.0,
                                            group_id: &group.id,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            crate::database::new_linked_group_envs(&mut self.conn, &ids).unwrap();
                            self.editable.groups.checkboxes.insert(group.id, true);
                        }
                        FieldState::Edit(previous) => {
                            let id = &previous.0;
                            let title: &str = &previous.1;
                            let old_envs: &[i32] = &previous.2;
                            let new_envs: Vec<GroupEnvLinkInsert> = self
                                .editable
                                .groups
                                .env_checkboxes
                                .iter()
                                .filter_map(|x| {
                                    if *x.1 {
                                        Some(GroupEnvLinkInsert {
                                            env_id: x.0,
                                            group_id: id,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let input: &str = &self.fields.configuration_fields.group.group_name;
                            if input != title {
                                update_group(&mut self.conn, id, input).unwrap();
                            }
                            let (added, removed) = hashset_comparison(old_envs, &new_envs);
                            //TODO we're wasting a lot of resources here re-creating envlinks. It's
                            //dumb but I didn't think it through with a basic implementation.
                            if !added.is_empty() {
                                let ids: Vec<GroupEnvLinkInsert> =
                                    added.into_iter().cloned().collect();
                                crate::database::new_linked_group_envs(&mut self.conn, &ids)
                                    .unwrap();
                            }
                            if !removed.is_empty() {
                                let r: Vec<i32> = removed.into_iter().cloned().collect();
                                crate::database::delete_linked_group_envs(&mut self.conn, &r, *id)
                                    .unwrap();
                            }
                        }
                    };
                    self.reload();
                    self.reload_env_checkboxes();
                    self.modals.group_state = Default::default();
                }
            }
        });
        if sub_modal.should_close() {
            self.reload_env_checkboxes();
            self.modals.group_state = Default::default();
        }
    }
    fn env_state_modal(&mut self, ui: &mut Ui) {
        use FieldState::Edit;
        let modal_id = Id::new("CONFIG_ENV_CREATOR");
        let sub_modal = egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let title = match self.modals.env_state.field {
                FieldState::Create => "CREATE MODE",
                FieldState::Edit(_) => "EDIT MODE",
            };
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();

            ui.label("Environment variable name :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.env.name);
            ui.label("Environment variable value :");
            ui.text_edit_singleline(&mut self.fields.configuration_fields.env.value);
            if self.fields.configuration_fields.env.tip {
                ui.label("You need to set both an environment variable name and value.");
            };
            ui.separator();

            if ui.button("Save and close").clicked() {
                if self.fields.configuration_fields.env.name.is_empty()
                    || self.fields.configuration_fields.env.value.is_empty()
                {
                    self.fields.configuration_fields.env.tip = true;
                } else {
                    self.fields.configuration_fields.env.tip = false;
                    //FIXME REMOVE UNWRAP
                    if let Edit((id, _name, _value)) = &self.modals.env_state.field {
                        update_env(
                            &mut self.conn,
                            id,
                            &self.fields.configuration_fields.env.name,
                            &self.fields.configuration_fields.env.value,
                        )
                        .unwrap();
                        self.reload();
                    } else {
                        let new_env = crate::database::new_env(
                            &mut self.conn,
                            &self.fields.configuration_fields.env.name,
                            &self.fields.configuration_fields.env.value,
                        )
                        .unwrap();
                        self.reload();
                        self.editable.groups.env_checkboxes.insert(new_env.id, true);
                    };
                    self.modals.env_state = Default::default();
                }
            };
            if let FieldState::Create = &self.modals.env_state.field {
                if ui.button("Save and add more").clicked() {
                    if self.fields.configuration_fields.env.name.is_empty()
                        || self.fields.configuration_fields.env.value.is_empty()
                    {
                        self.fields.configuration_fields.env.tip = true;
                    } else {
                        self.fields.configuration_fields.env.tip = false;
                        //FIXME REMOVE UNWRAP
                        let new_env = crate::database::new_env(
                            &mut self.conn,
                            &self.fields.configuration_fields.env.name,
                            &self.fields.configuration_fields.env.value,
                        )
                        .unwrap();
                        self.reload();

                        self.editable.groups.env_checkboxes.insert(new_env.id, true);
                        self.fields.configuration_fields.env = EnvFields::default();
                    }
                };
            };
        });
        if sub_modal.should_close() {
            self.modals.env_state = Default::default();
        }
    }
    fn reload(&mut self) {
        let conn = &mut self.conn;
        let cfgs = get_all(conn).unwrap();
        let groups = get_groups(conn).unwrap();
        let envs = get_environments(conn).unwrap();
        let execs = get_executables(conn).unwrap();
        let editable = &mut self.editable;
        let envs: HashMap<i32, Environment> = envs.into_iter().map(|el| (el.id, el)).collect();
        let executables = execs.into_iter().map(|el| (el.id, el)).collect();
        let groups: HashMap<i32, GroupedEnvironment> =
            groups.into_iter().map(|el| (el.id, el)).collect();
        //FIXME this is nice because it doesn't reset the checkboxes if you add something else.
        //On the other hand, it's terrible because it might lead to a desync if you DELETE a group
        //or environment.
        for (id, _) in groups.iter() {
            if !editable.groups.checkboxes.contains_key(id) {
                editable.groups.checkboxes.insert(*id, false);
            };
        }
        for (id, _) in envs.iter() {
            if !editable.groups.env_checkboxes.contains_key(id) {
                editable.groups.env_checkboxes.insert(*id, false);
            };
        }
        self.environment_variables = envs;
        self.configurations = cfgs;
        self.executables = executables;
        self.groups = groups;
    }
    fn display_configurations(&mut self, ui: &mut Ui) {
        if self.configurations.is_empty() {
            ui.centered_and_justified(|ui| {
                let text = RichText::new("No configurations yet...")
                    .heading()
                    .italics();
                ui.label(text);
            });
        } else {
            self.configurations(ui);
        }
    }
    fn show_env_modal(&mut self, ui: &mut Ui) {
        let modal = Modal::new(Id::new("SHOW_ENVIRONMENT_VARIABLES")).show(ui.ctx(), |ui| {
            let envs = self.modals.show_env.envs.clone();
            if envs.is_empty() {
                ui.heading("This group is empty... There are no environment variables.");
            } else {
                egui::Grid::new("GRID_ENVIRONMENTS_SHOW")
                    .striped(true)
                    .min_col_width(100.)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("NAME");
                        });

                        ui.vertical_centered(|ui| {
                            ui.heading("VALUE");
                        });
                        ui.end_row();
                        for env in envs.iter() {
                            ui.label(&env.name);
                            ui.label(&env.value);
                            ui.horizontal_centered(|ui| {
                                if ui.button("delete").clicked() {
                                    delete_env(&mut self.conn, &env.id).unwrap();
                                    self.modals.show_env.envs = get_envs_for_group(
                                        &mut self.conn,
                                        &self.modals.show_env.group_id.clone().unwrap(),
                                    )
                                    .unwrap();
                                    self.reload();
                                }

                                if ui.button("edit").clicked() {
                                    self.fields.configuration_fields.env.name = env.name.clone();
                                    self.fields.configuration_fields.env.value = env.value.clone();
                                    self.modals.env_state.field = FieldState::Edit((
                                        env.id,
                                        env.name.clone(),
                                        env.value.clone(),
                                    ));
                                    self.modals.env_state.open = true;
                                }
                            });
                            ui.end_row();
                        }
                    });
            };
        });
        if modal.should_close() {
            self.modals.show_env = ShowEnvModal::default();
        }
    }
    fn configurations(&mut self, ui: &mut Ui) {
        Frame::new()
            .corner_radius(10.0)
            .stroke(Stroke::new(3., Color32::BLACK))
            .inner_margin(10.)
            .show(ui, |ui| {
                let size = ui.max_rect().size();
                let mut col_width = 400.;
                let xsize = (self.configurations.len() as f32).min((size.x / col_width).floor());
                let mut x: usize = xsize as usize;
                let leftover = if x == 0 {
                    col_width = size.x;
                    x = 1;
                    0.
                } else {
                    (size.x - (col_width * xsize) - 20.) / xsize
                };
                let min_col_width = col_width + leftover;
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    let configurations = self.configurations.clone();
                    Grid::new(min_col_width.to_string())
                        .striped(true)
                        .min_col_width(min_col_width)
                        .show(ui, |ui| {
                            for (idx, cfg) in configurations.iter().enumerate() {
                                Frame::NONE
                                    .stroke(Stroke::new(2., Color32::DARK_GRAY))
                                    .corner_radius(2.5)
                                    .outer_margin(5.)
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(12.);
                                            ui.heading(&cfg.configuration.name);
                                            ui.separator();
                                            let exec = self
                                                .executables
                                                .get(&cfg.configuration.exec)
                                                .expect(
                                                    "If the executable doesn't exist, we have a \
                                                     problem.",
                                                );
                                            ui.small(format!("id : {}", &cfg.configuration.id));
                                            ui.separator();
                                            ui.label(
                                                String::from("Executable Name : ") + &exec.name,
                                            );
                                            ui.label(
                                                String::from("Executable Value : ")
                                                    + &exec.executable,
                                            );

                                            ui.separator();
                                            if ui
                                                .add(
                                                    egui::Button::new("Modify Configuration")
                                                        .corner_radius(4.)
                                                        .fill(Color32::BLACK),
                                                )
                                                .clicked()
                                            {
                                                //ERROR HERE
                                                self.fields
                                                    .configuration_fields
                                                    .configuration_name =
                                                    cfg.configuration.name.clone();
                                                self.editable.exec.id = cfg.configuration.exec;
                                                for group in cfg.groups.iter() {
                                                    *self
                                                        .editable
                                                        .groups
                                                        .checkboxes
                                                        .get_mut(&group.group.id)
                                                        .unwrap() = true;
                                                }
                                                self.modals.main_state.field = FieldState::Edit((
                                                    cfg.configuration.id,
                                                    cfg.configuration.name.clone(),
                                                    cfg.groups.clone(),
                                                    cfg.configuration.exec,
                                                ));
                                                self.modals.main_state.open = true;
                                            };
                                            ui.separator();
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        RichText::new("Use as Base"), // .color(Color32),
                                                    )
                                                    .corner_radius(3.)
                                                    .fill(Color32::from_rgb(15, 15, 30)),
                                                )
                                                .clicked()
                                            {
                                                //ERROR HERE
                                                self.fields
                                                    .configuration_fields
                                                    .configuration_name =
                                                    cfg.configuration.name.clone();
                                                self.editable.exec.id = cfg.configuration.exec;
                                                for group in cfg.groups.iter() {
                                                    *self
                                                        .editable
                                                        .groups
                                                        .checkboxes
                                                        .get_mut(&group.group.id)
                                                        .unwrap() = true;
                                                }
                                                self.modals.main_state.open = true;
                                            };
                                            Frame::NONE.inner_margin(5.).show(ui, |ui| {
                                                ScrollArea::horizontal()
                                                    .id_salt(cfg.configuration.id.to_string())
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.add_space(15.);
                                                            for group in cfg.groups.iter() {
                                                                self.display_group(ui, group);
                                                            }
                                                            ui.add_space(15.);
                                                        });
                                                        //Space for the scrollbar
                                                        ui.add_space(12.);
                                                    });
                                            });
                                        });
                                    });

                                if (idx + 1) % (x) == 0 {
                                    ui.end_row();
                                }
                            }
                        })
                });
            });
    }
    fn display_group(&mut self, ui: &mut Ui, group: &LinkedGroups) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        Frame::NONE
            .outer_margin(4.)
            .inner_margin(5.)
            .corner_radius(2.5)
            .stroke(Stroke::new(0.5, Color32::WHITE))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!("Group Name : {}", group.group.name));
                    let s = ui.min_size();
                    ui.allocate_ui_with_layout(s, Layout::top_down(egui::Align::TOP), |ui| {
                        ui.separator();
                    });
                    // Separator::default()
                    //     .horizontal(); //shrink(self, shrink)
                    //                    // ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Show").clicked() {
                            self.modals.show_env.group_id = Some(GroupedEnvironment {
                                id: group.group.id,
                                name: group.group.name.clone(),
                            });
                            self.modals.show_env.envs = group.environments.clone();
                            self.modals.show_env.show = true;
                        };
                        if ui.button("Edit").clicked() {
                            let mut environments: HashMap<i32, bool> = self
                                .environment_variables
                                .clone()
                                .into_iter()
                                .map(|x| (x.0, false))
                                .collect();
                            for env in group.environments.iter() {
                                environments.insert(env.id, true);
                            }
                            self.fields.configuration_fields.group.group_name =
                                group.group.name.clone();
                            self.editable.groups.env_checkboxes = environments;
                            self.modals.group_state.field = FieldState::Edit((
                                group.group.id,
                                group.group.name.clone(),
                                group.environments.iter().map(|i| i.id).collect(),
                            ));
                            self.modals.group_state.open = true;
                        }
                    });
                });
            });
    }
}
