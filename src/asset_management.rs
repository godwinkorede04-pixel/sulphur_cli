use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;
use sulphur_core::{Asset, GameData, Iwad, Mod, Movable};

use crate::file_utils::select_file;
use crate::menu::{Menu, BACK_BUTTON};

pub trait AssetCollection<T: Movable + AsMut<Asset>> {
    fn get_assets(&self) -> &Vec<T>;
    fn get_assets_mut(&mut self) -> &mut Vec<T>;
    fn create_asset(&self, path: PathBuf) -> T;
    fn get_asset_name(&self, asset: &T) -> String;
    fn get_asset_mut(&mut self, index: usize) -> &mut Asset;
    fn get_asset_state(&self, asset: &T) -> bool;
}

impl AssetCollection<Iwad> for GameData {
    fn get_assets(&self) -> &Vec<Iwad> {
        &self.iwads
    }

    fn get_assets_mut(&mut self) -> &mut Vec<Iwad> {
        &mut self.iwads
    }

    fn create_asset(&self, path: PathBuf) -> Iwad {
        Iwad {
            0: Asset {
                path,
                enabled: true,
            },
        }
    }

    fn get_asset_name(&self, asset: &Iwad) -> String {
        asset
            .0
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn get_asset_mut(&mut self, index: usize) -> &mut Asset {
        &mut self.iwads[index].0
    }

    fn get_asset_state(&self, asset: &Iwad) -> bool {
        asset.0.enabled
    }
}

impl AssetCollection<Mod> for GameData {
    fn get_assets(&self) -> &Vec<Mod> {
        &self.mods
    }

    fn get_assets_mut(&mut self) -> &mut Vec<Mod> {
        &mut self.mods
    }

    fn create_asset(&self, path: PathBuf) -> Mod {
        Mod {
            0: Asset {
                path,
                enabled: true,
            },
        }
    }

    fn get_asset_name(&self, asset: &Mod) -> String {
        asset
            .0
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn get_asset_mut(&mut self, index: usize) -> &mut Asset {
        &mut self.mods[index].0
    }

    fn get_asset_state(&self, asset: &Mod) -> bool {
        asset.0.enabled
    }
}

fn asset_management<T: Movable + AsMut<Asset>>(
    game_data: &mut GameData,
    menu: Menu,
    file_prompt: &str,
    extensions: &[&str],
    toggle_prompt: &str,
    remove_prompt: &str,
) -> Result<()>
where
    GameData: AssetCollection<T>,
{
    let mut has_entered_loop = false;
    loop {
        let names: Vec<String> = game_data
            .get_assets()
            .iter()
            .map(|asset| {
                format!(
                    "{} ({})",
                    game_data.get_asset_name(asset),
                    if game_data.get_asset_state(asset) {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                )
            })
            .collect();
        let selection = if !game_data.get_assets().is_empty() || has_entered_loop {
            Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose Action")
                .items(&menu.options())
                .default(0)
                .interact()?
        } else {
            0
        };

        has_entered_loop = true;

        match selection {
            0 => {
                if let Some(path) = select_file(file_prompt, false, Some(extensions), None)? {
                    let asset = game_data.create_asset(path);
                    game_data.get_assets_mut().push(asset);
                    if Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to move the file?")
                        .default(true)
                        .interact()?
                    {
                        // Since we're adding an element to the vector, the old length is the last element
                        game_data.get_assets_mut()[names.len()].move_file(false)?;
                    }
                }
            }
            1 => {
                if game_data.get_assets().is_empty() {
                    println!("No assets available to toggle.");
                    continue;
                }

                let mut toggle_names = names.clone();
                toggle_names.push(BACK_BUTTON.to_string());

                let selection_index = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(toggle_prompt)
                    .items(&toggle_names)
                    .default(0)
                    .interact()?;

                if selection_index == toggle_names.len() - 1 {
                    continue;
                }

                let asset = game_data.get_asset_mut(selection_index);
                asset.enabled = !asset.enabled;
                println!(
                    "{} {}",
                    if asset.enabled { "Enabled" } else { "Disabled" },
                    asset.path.file_name().unwrap().to_string_lossy()
                );
            }
            2 => {
                if game_data.get_assets().is_empty() {
                    println!("No assets available to remove.");
                    continue;
                }

                let mut remove_names = names.clone();
                remove_names.push(BACK_BUTTON.to_string());

                let selection_index = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(remove_prompt)
                    .items(&remove_names)
                    .default(0)
                    .interact()?;

                if selection_index == remove_names.len() - 1 {
                    continue;
                }

                let selected_name = &remove_names[selection_index];

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!(
                        "Are you sure you want to delete '{}'?",
                        selected_name
                    ))
                    .default(false)
                    .interact()?
                {
                    game_data.get_assets_mut().remove(selection_index);
                    return Ok(());
                }
            }
            3 => {
                return Ok(());
            }
            _ => {}
        }
    }
}

pub fn iwad_management(game_data: &mut GameData) -> Result<()> {
    asset_management::<Iwad>(
        game_data,
        Menu::IwadManagementMenu,
        "Select IWAD file",
        &["wad", "iwad"],
        "Select IWAD to toggle",
        "Select IWAD to remove",
    )
}

pub fn mod_management(game_data: &mut GameData) -> Result<()> {
    asset_management::<Mod>(
        game_data,
        Menu::ModManagementMenu,
        "Select Mod file",
        &["wad", "pk3", "zip"],
        "Select Mod to toggle",
        "Select Mod to remove",
    )
}

pub fn additional_params_management(game_data: &mut GameData) -> Result<()> {
    loop {
        match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose Action")
            .items(&Menu::AdditionalParamsMenu.options())
            .default(0)
            .interact()?
        {
            0 => {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter new parameter")
                    .interact_text()?;
                game_data.additional_params.push(input.into());
            }
            1 => {
                if game_data.additional_params.is_empty() {
                    println!("No parameters to remove!");
                    continue;
                }

                let mut param_list: Vec<String> = game_data
                    .additional_params
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                param_list.push(BACK_BUTTON.to_string());

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select parameter to remove")
                    .items(&param_list)
                    .default(0)
                    .interact()?;

                if selection == param_list.len() - 1 {
                    continue;
                }

                if selection < game_data.additional_params.len() {
                    game_data.additional_params.remove(selection);
                }
            }
            2 => return Ok(()),
            _ => {}
        }
    }
}
