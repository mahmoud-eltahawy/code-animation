use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::html::ClassStyle;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn create_dark_theme() -> String {
    let ts = ThemeSet::load_defaults();
    let dark_theme = &ts.themes["Solarized (dark)"];
    css_for_theme_with_class_style(dark_theme, ClassStyle::Spaced).unwrap()
}

fn file_write(content: String, path: &str) -> Result<(), std::io::Error> {
    let file = File::create(Path::new(path))?;
    let mut writer = BufWriter::new(&file);
    writeln!(writer, "{}", content)?;
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    tauri_build::build();

    file_write(create_dark_theme(), "../style.css")?;

    Ok(())
}
