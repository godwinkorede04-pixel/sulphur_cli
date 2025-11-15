use anyhow::Result;
use chrono::{DateTime, Local};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Select};
use sulphur_core::{Instance, SaveableDefaultPath, SulphurConfig};
use tabled::{builder::Builder, settings::Style as TabledStyle};

use crate::duration_utils::ToString;
use crate::instance_management;
use crate::menu::Menu;

pub fn run_main_loop(config: &mut SulphurConfig) -> Result<()> {
    let mut sort_by_playtime: bool = false;

    loop {
        config.save()?;
        let instances_order: Vec<usize> = config
            .get_unplayed_instances()
            .iter()
            .cloned()
            .chain(if sort_by_playtime {
                config.get_indices_by_playtime()
            } else {
                config.get_indices_by_last_played()
            })
            .collect();

        list_instances(config, &instances_order);
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .items(Menu::MainMenu.options())
            .default(0)
            .interact()?;

        match selection {
            0 => run_instance(config, instances_order.as_slice())?,
            1 => {
                sort_by_playtime = !sort_by_playtime;
                println!(
                    "Sorting by {}",
                    if sort_by_playtime {
                        "playtime"
                    } else {
                        "last played"
                    }
                );
            }
            2 => {
                let new = instance_management::create_new_instance()?;
                if let instance_management::InstanceManagementExitState::Some(new) = new {
                    config.instances.push(new);
                }
            }
            3 => instance_management::manage_instances(config, instances_order.as_slice())?,
            4 => instance_management::import_export(config, instances_order.as_slice())?,
            5 => global_settings(config)?,
            6 => {
                return Ok(());
            }
            _ => {}
        }
    }
}

fn global_settings(config: &mut SulphurConfig) -> Result<()> {
    let mut selection = 0;
    loop {
        let menu_items = Menu::GlobalSettingsMenu.options();
        selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose Action")
            .items(&menu_items)
            .default(selection)
            .interact()?;

        match selection {
            0 => {
                let input_string = dialoguer::Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter command to run gzdoom")
                    .default(config.gzdoom_command.to_string_lossy().to_string())
                    .interact_text()?;
                config.gzdoom_command = input_string.into();
            }
            1 => return Ok(()),
            _ => {}
        }
    }
}

fn list_instances(config: &SulphurConfig, indexes: &[usize]) {
    let index_style = Style::new().bold().fg(console::Color::Cyan);
    let separator_style = Style::new().fg(console::Color::White).dim();

    for (display_index, &actual_index) in indexes.iter().enumerate() {
        if display_index > 0 {
            println!("{}", separator_style.apply_to("─".repeat(80)));
        }

        println!(
            "{}",
            index_style.apply_to(format!("Instance #{}", display_index + 1)),
        );

        let table = instance_table(&config.instances[actual_index]);
        for line in table.lines() {
            println!("\t\t{}", line);
        }
        println!();
    }
}

fn instance_table(instance: &Instance) -> String {
    let last_played = &format!("{}", {
        if let Some(a) = instance.metadata.last_played {
            let date_time: DateTime<Local> = a.into();
            format!(
                "{}  (lasted {})",
                date_time.format("%c").to_string(),
                instance.metadata.last_session_duration.unwrap().to_string()
            )
        } else {
            "Never".to_string()
        }
    },);

    let data = [
        ("Name", &instance.metadata.name),
        ("Playtime", &instance.metadata.playtime.to_string()),
        ("Last Played", last_played),
    ];

    let mut table = Builder::new();
    for (name, value) in data.iter() {
        table.push_record([name, value.as_str()]);
    }

    let mut table = table.build();
    table.with(TabledStyle::extended());

    table.to_string()
}

fn run_instance(config: &mut SulphurConfig, indexes: &[usize]) -> Result<()> {
    if let Some(instance_index) = instance_management::instance_selection(config, indexes)? {
        let command = config.get_command();
        let instance = &mut config.instances[instance_index];
        instance.run(instance.get_full_command(command.as_os_str()));
    }
    Ok(())
}
