use anyhow::Result;
use console::Style;
use dialoguer::FuzzySelect;
use std::path::PathBuf;

pub fn select_file(
    prompt: &str,
    select_folder: bool,
    allowed_extensions: Option<&[&str]>,
    default: Option<PathBuf>,
) -> Result<Option<PathBuf>> {
    const SPECIAL_DIR_STYLE: console::Style = Style::new().fg(console::Color::Magenta);
    const DIR_STYLE: console::Style = Style::new().fg(console::Color::Blue);
    const FILE_STYLE: console::Style = Style::new().fg(console::Color::Green);
    const HIDDEN_DIR_STYLE: console::Style = Style::new().fg(console::Color::Color256(23)); // Dark blue
    const HIDDEN_FILE_STYLE: console::Style = Style::new().fg(console::Color::Color256(22)); // Dark green

    let mut dir = if let Some(default) = default {
        default.parent().unwrap().to_path_buf()
    } else {
        std::env::home_dir().unwrap_or(std::env::current_dir()?)
    };

    loop {
        let folder_contents = std::fs::read_dir(&dir)?;
        let mut entries: Vec<(String, PathBuf, bool)> = Vec::new(); // (display_name, path, is_dir)

        entries.push((
            SPECIAL_DIR_STYLE.apply_to("← Back (Cancel)").to_string(),
            PathBuf::new(),
            false,
        ));

        if select_folder {
            entries.push((
                SPECIAL_DIR_STYLE
                    .apply_to("./\t(choose current directory)")
                    .to_string(),
                dir.clone(),
                true,
            ));
        }
        if let Some(parent) = dir.parent() {
            entries.push((
                SPECIAL_DIR_STYLE
                    .apply_to("../\t(parent directory)")
                    .to_string(),
                parent.to_path_buf(),
                true,
            ));
        }

        let mut dirs: Vec<(String, PathBuf, bool)> = Vec::new();
        let mut files = dirs.clone(); // Aint writing all that again

        for entry in folder_contents {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let is_hidden = file_name.starts_with('.');

            if path.is_dir() {
                let style = if is_hidden {
                    HIDDEN_DIR_STYLE
                } else {
                    DIR_STYLE
                };
                let styled_name = style.apply_to(format!(" {}/", file_name));
                dirs.push((styled_name.to_string(), path, true));
            } else if !select_folder {
                if let Some(extensions) = allowed_extensions {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if !extensions
                            .iter()
                            .any(|&allowed| allowed.to_lowercase() == ext_str)
                        {
                            continue; // Skip files that don't match allowed extensions
                        }
                    } else {
                        continue; // Skip files without extensions if filter is specified
                    }
                }

                let style = if is_hidden {
                    HIDDEN_FILE_STYLE
                } else {
                    FILE_STYLE
                };
                let styled_name = style.apply_to(format!(" {}", file_name));
                files.push((styled_name.to_string(), path, false));
            }
        }

        // we don't nid dirs and files after this loop.
        for mut list in [dirs, files] {
            list.sort_by(|a, b| a.1.file_name().cmp(&b.1.file_name()));
            entries.extend(list);
        }

        let display_items: Vec<&str> = entries.iter().map(|(name, _, _)| name.as_str()).collect();

        let selection = FuzzySelect::new()
            .with_prompt(prompt)
            .items(&display_items)
            .interact()?;

        if selection == 0 {
            return Ok(None);
        }

        let (_, selected_path, is_dir) = &entries[selection];

        if *is_dir {
            if select_folder && selection == 1 {
                return Ok(Some(selected_path.clone()));
            }
            dir = selected_path.clone();
        } else {
            return Ok(Some(selected_path.clone()));
        }
    }
}
