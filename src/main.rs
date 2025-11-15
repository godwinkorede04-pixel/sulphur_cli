extern crate chrono;

use anyhow::Result;
use sulphur_core::{SaveableDefaultPath, SulphurConfig};

mod ui;
mod menu;
mod file_utils;
mod instance_management;
mod asset_management;
mod duration_utils;

fn main() -> Result<()> {
    let mut config = SulphurConfig::load().unwrap_or_else(|_| SulphurConfig::new());
    ui::run_main_loop(&mut config)
}