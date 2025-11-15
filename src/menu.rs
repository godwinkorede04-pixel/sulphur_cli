pub const BACK_BUTTON: &str = "Back";

pub enum Menu {
    MainMenu,
    EditInstanceMenu,
    IwadManagementMenu,
    ModManagementMenu,
    GlobalSettingsMenu,
    AdditionalParamsMenu,
    ImportExportMenu,
}

impl Menu {
    pub fn options(&self) -> &'static [&'static str] {
        match self {
            Menu::MainMenu => &[
                "Run Instance",
                "Change List Order",
                "Create New Instance",
                "Edit Instance",
                "Import/Export Instance",
                "Configure Global Settings",
                "Save & Exit",
            ],
            Menu::EditInstanceMenu => &[
                "Edit Name",
                "Edit IWADs",
                "Edit Mods",
                "Edit Save Directory",
                "Edit Additional Parameters",
                "See Full Command",
                "Remove Instance",
                "Save Changes",
                BACK_BUTTON,
            ],
            Menu::IwadManagementMenu => &["Add IWAD", "Toggle IWAD", "Remove IWAD", BACK_BUTTON],
            Menu::ModManagementMenu => &["Add Mod", "Toggle Mod", "Remove Mod", BACK_BUTTON],
            Menu::GlobalSettingsMenu => &["Set GZDoom Path", BACK_BUTTON],
            Menu::AdditionalParamsMenu => &["Add New Parameter", "Remove Parameter", BACK_BUTTON],
            Menu::ImportExportMenu => &["Export as .brimpkg", "Import .brimpkg", BACK_BUTTON],
        }
    }
}
