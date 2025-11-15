use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::ffi::OsString;
use sulphur_core::SulphurConfig;
use sulphur_core::{GameData, Instance, Metadata};

use crate::asset_management::{additional_params_management, iwad_management, mod_management};
use crate::file_utils::select_file;
use crate::menu::{Menu, BACK_BUTTON};

pub enum InstanceManagementExitState {
    Some(Instance),
    None,
    Delete,
}

pub fn create_new_instance() -> Result<InstanceManagementExitState> {
    edit_single_instance(Instance {
        metadata: Metadata {
            name: "".to_string(),
            image: None,
            playtime: Default::default(),
            last_played: None,
            last_session_duration: None,
        },
        gamedata: GameData {
            iwads: vec![],
            mods: vec![],
            savedir: Default::default(),
            additional_params: vec![],
        },
    })
}

pub fn manage_instances(config: &mut SulphurConfig, indexes: &[usize]) -> Result<()> {
    let selection = instance_selection(config, indexes)?;

    if let Some(instance_index) = selection {
        let instance = &config.instances[instance_index];

        match edit_single_instance(instance.clone())? {
            InstanceManagementExitState::Some(edited_instance) => {
                config.instances[instance_index] = edited_instance;
            }
            InstanceManagementExitState::None => {
                println!("Cancelled!");
            }
            InstanceManagementExitState::Delete => {
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!(
                        "Are you sure you want to delete '{}'?",
                        instance.metadata.name
                    ))
                    .default(false)
                    .interact()?
                {
                    config.instances.remove(instance_index);
                    return Ok(());
                }
            }
        }
    } else {
        return Ok(());
    }
    manage_instances(config, indexes)
}

pub fn instance_selection(config: &SulphurConfig, indexes: &[usize]) -> Result<Option<usize>> {
    let instances = config.get_instances();
    let instance_names: Vec<&str> = indexes
        .iter()
        .map(|&index| instances[index].metadata.name.as_str())
        .collect();

    let mut menu_items = instance_names;
    menu_items.push(BACK_BUTTON);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose Instance")
        .default(0)
        .items(&menu_items)
        .interact()?;

    let back_button_index = indexes.len();
    if selection == back_button_index {
        Ok(None)
    } else {
        Ok(Some(indexes[selection]))
    }
}

pub fn edit_single_instance(instance: Instance) -> Result<InstanceManagementExitState> {
    let mut result = instance.clone();
    let initial_savedir = result.gamedata.savedir.clone();
    loop {
        let selection = if !result.metadata.name.is_empty() {
            Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose Action")
                .items(&Menu::EditInstanceMenu.options())
                .default(0)
                .interact()?
        } else {
            0
        };

        match selection {
            0 => {
                result.metadata.name = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter instance name")
                    .default(result.metadata.name.clone())
                    .interact_text()?;
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!(
                        "Want to update the saves directory of '{}'?",
                        result.metadata.name
                    ))
                    .default(true)
                    .interact()?
                {
                    result.initialize_absolute_savedir()?
                }
            }
            1 => iwad_management(&mut result.gamedata)?,
            2 => mod_management(&mut result.gamedata)?,
            3 => {
                println!(
                    "Current save directory: '{}'",
                    result.gamedata.savedir.to_string_lossy()
                );
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to change it?")
                    .default(false)
                    .interact()?
                {
                    if let Some(new_path) = select_file(
                        "Choose new save folder",
                        true,
                        None,
                        Some(result.gamedata.savedir.clone()),
                    )? {
                        result.gamedata.savedir = new_path;
                    }
                }
            }
            4 => additional_params_management(&mut result.gamedata)?,
            5 => println!(
                "Full command: {}",
                result
                    .get_full_command(OsString::from("gzdoom").as_os_str())
                    .to_string_lossy()
            ),
            6 => return Ok(InstanceManagementExitState::Delete),
            7 => {
                if initial_savedir.exists() {
                    if &initial_savedir != &result.gamedata.savedir {
                        println!(
                            "The old save directory '{}' has been moved to {}.",
                            initial_savedir.to_string_lossy(),
                            result.gamedata.savedir.to_string_lossy()
                        );
                        std::fs::rename(initial_savedir, &result.gamedata.savedir)?;
                    }
                } else {
                    result.create_savedir()?;
                }
                return Ok(InstanceManagementExitState::Some(result));
            }
            8 => {
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Are you sure you want to exit? Unsaved changes will be lost.")
                    .default(false)
                    .interact()?
                {
                    return Ok(InstanceManagementExitState::None);
                }
            }
            _ => {}
        }
    }
}

pub fn import_export(config: &mut SulphurConfig, indexes: &[usize]) -> Result<()> {
    let mut selection = 0;
    loop {
        let menu_items = Menu::ImportExportMenu.options();
        selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose Action")
            .items(&menu_items)
            .default(selection)
            .interact()?;
        match selection {
            1 => {
                if let Some(file_path) = select_file(
                    "Choose Brimpkg file",
                    false,
                    Some(&["zip", "brimpkg"]),
                    None,
                )? {
                    let new_instance_index =
                        config.add_instance(Instance::load_brimpkg(file_path.as_path())?);
                    println!(
                        "Instance {} added!",
                        config.instances[new_instance_index].metadata.name
                    );
                }
            }
            0 => {
                let index = instance_selection(config, indexes)?;
                if let Some(index) = index {
                    if let Some(save_path) =
                        select_file("Choose Folder to save the instance in", true, None, None)?
                    {
                        let save_file_path = save_path.join(format!(
                            "{}.brimpkg",
                            &config.instances[index].metadata.name
                        ));
                        println!(
                            "Saving instance {} to {}",
                            &config.instances[index].metadata.name,
                            save_file_path.to_string_lossy()
                        );
                        let _ = config.instances[index].save_brimpkg(
                            &save_file_path,
                            Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt("Do you want to transfer your saves?")
                                .interact()?,
                            Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt("Do you want to transfer your playtime data?")
                                .interact()?,
                        );
                        println!("Instance saved!");
                    }
                }
            }
            2 => return Ok(()),
            _ => {}
        }
    }
}
