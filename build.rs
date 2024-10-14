use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::{env, io};

/// Парсить офсети з файлу .hmp та повертає вектор з офсетами.
fn parse_offsets_file(file_path: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut glyphs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        for part in line.split_whitespace() {
            if part.contains('-') {
                // Обробляємо діапазон гліфів, наприклад, "32-127"
                let parts: Vec<&str> = part.split('-').collect();
                if let (Ok(start), Ok(end)) = (
                    parts[0].trim().parse::<u32>(),
                    parts[1].trim().parse::<u32>(),
                ) {
                    glyphs.extend(start..=end);
                }
            } else if let Ok(glyph) = part.parse::<u32>() {
                glyphs.push(glyph);
            }
        }
    }

    glyphs.sort_unstable();
    glyphs.dedup();

    Ok(glyphs)
}

/// Генерує мапу офсетів для шрифтів та записує її у файл.
fn generate_offsets_map(
    offsets_files: &[PathBuf],
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join(output_file);
    let mut out_file = File::create(&dest_path)?;

    writeln!(out_file, "use phf::phf_map;")?;
    writeln!(
        out_file,
        "pub static OFFSETS: phf::Map<&'static str, &'static [u32]> = phf_map! {{"
    )?;

    for file_path in offsets_files {
        // Get the file name from the path and strip the ".hmp" extension
        let file_name = file_path
            .file_stem() // Extract the file name without extension
            .unwrap()
            .to_str()
            .unwrap();

        let offsets = parse_offsets_file(file_path.to_str().unwrap())?;
        let offsets_str = format!("{:?}", offsets); // Convert to string
        writeln!(out_file, "    \"{}\" => &{},", file_name, offsets_str)?;
    }

    writeln!(out_file, "}};")?;
    Ok(())
}

/// Парсить файли гліфів та повертає мапу з номером гліфа та його даними.
fn parse_glyphs_files(
    glyphs_data: &[&str],
) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let mut glyphs_map: HashMap<u32, String> = HashMap::new();
    let mut current_glyph_id: Option<u32> = None;

    // Проходимо по кожному файлу з масиву glyphs_data.
    for file_content in glyphs_data {
        for line in file_content.lines().filter(|line| !line.trim().is_empty()) {
            // Перевіряємо, чи рядок має достатню довжину для парсингу номера гліфа.
            if line.len() >= 5 {
                let potential_id = &line[0..5];
                if potential_id.trim().chars().all(|c| c.is_digit(10)) {
                    match potential_id.trim().parse::<u32>() {
                        Ok(glyph_id) => {
                            current_glyph_id = Some(glyph_id);
                            let glyph_data = line.to_string();
                            // Використовуємо тимчасову змінну для зберігання гліфів
                            glyphs_map
                                .entry(glyph_id)
                                .and_modify(|e| {
                                    e.push_str(&glyph_data);
                                })
                                .or_insert_with(|| glyph_data.clone());

                            continue;
                        }
                        Err(_) => {
                            return Err(
                                format!("Не вдалося парсити ID гліфа з рядка: '{}'", line).into()
                            );
                        }
                    }
                }
            }

            if let Some(glyph_id) = current_glyph_id {
                if let Some(existing_data) = glyphs_map.get_mut(&glyph_id) {
                    existing_data.push_str(line.trim());
                } else {
                    return Err(
                        format!("Продовження гліфа без існуючого ID. Рядок: '{}'", line).into(),
                    );
                }
            } else {
                return Err(format!(
                    "Рядок без номера гліфа та відсутність попереднього гліфа: '{}'",
                    line
                )
                .into());
            }
        }
    }

    Ok(glyphs_map)
}

/// Генерує Rust файл з гліфами та записує його у файл `occidental_fonts.rs`.
fn generate_fonts_map(
    glyphs_map: &HashMap<u32, String>,
    map_name: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join(file_name);
    let mut out_file = File::create(&dest_path)?;

    // Записуємо початкову частину статичної змінної
    writeln!(
        out_file,
        "pub static {}: phf::Map<u32, &'static str> = phf_map! {{",
        map_name
    )?;

    // Створюємо вектор ключів, сортуємо їх
    let mut glyph_keys: Vec<_> = glyphs_map.keys().collect();
    glyph_keys.sort();

    // Ітеруємо по відсортованих ключах і записуємо їх до файлу
    for &glyph_id in &glyph_keys {
        if let Some(glyph_data) = glyphs_map.get(glyph_id) {
            let escaped_glyph_data = glyph_data.replace("\\", "\\\\").replace("\"", "\\\"");
            writeln!(
                out_file,
                "    {}u32 => \"{}\",",
                glyph_id, escaped_glyph_data
            )?;
        }
    }

    writeln!(out_file, "}};")?;
    Ok(())
}

/// Зчитує файли гліфів, парсить їх вміст і повертає мапу гліфів.
fn load_glyphs(glyph_files: &[String]) -> Result<HashMap<u32, String>, Box<dyn std::error::Error>> {
    let mut glyphs_data = Vec::new();

    // Проходимо по кожному файлу у списку
    for file_path in glyph_files {
        let path = Path::new(file_path);

        // Відкриваємо файл і читаємо його вміст
        let file = File::open(&path)
            .map_err(|e| format!("Не вдалося відкрити файл '{}': {}", file_path, e))?;
        let reader = BufReader::new(file);

        // Збираємо всі рядки, але без втрати символів переходу на новий рядок
        let mut content = String::new();
        for line in reader.lines() {
            let line = line.map_err(|e| {
                format!("Не вдалося прочитати рядок з файлу '{}': {}", file_path, e)
            })?;

            // Об'єднуємо зчитані лінії, додаючи символ переходу на новий рядок
            content.push_str(&line);
            content.push('\n'); // Явно додаємо символ нової строки для коректного злиття
        }

        glyphs_data.push(content);
    }

    // Створюємо вектор посилань на рядки для подальшої обробки
    let glyphs_data_refs: Vec<&str> = glyphs_data.iter().map(|s| s.as_str()).collect();

    // Парсимо зчитані дані та генеруємо мапу гліфів
    let glyphs_map = parse_glyphs_files(&glyphs_data_refs)?;

    Ok(glyphs_map)
}

/// Генерує Rust файл, що містить HERSHEY_OCCIDENTAL_UNICODE_MAP за допомогою крейту `phf`.
fn generate_unicode_map(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(csv_path)?;
    let reader = io::BufReader::new(file);

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR не встановлено");
    let dest_path = Path::new(&out_dir).join("occidental_unicode_map.rs");
    let mut out_file = File::create(&dest_path)?;

    writeln!(out_file, "")?;
    writeln!(
        out_file,
        "pub static HERSHEY_OCCIDENTAL_UNICODE_MAP: phf::Map<u32, u32> = phf_map! {{"
    )?;

    let mut csv_reader = csv::Reader::from_reader(reader);

    for result in csv_reader.records() {
        let record = result?;
        let hid_str = record.get(0).unwrap_or("").trim();
        let unicode_str = record.get(1).unwrap_or("").trim();

        let hid: u32 = match hid_str.parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let unicode: u32 = match unicode_str.parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        writeln!(out_file, "    {}u32 => {}u32,", hid, unicode)?;
    }

    writeln!(out_file, "}};")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.oc1");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.oc2");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.oc3");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.oc4");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.or1");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.or2");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.or3");
    println!("cargo:rerun-if-changed=fonts/hershey/hersh.or4");
    println!("cargo:rerun-if-changed=fonts/hershey/unicodemap.csv");

    let occidental_glyph_files = vec![
        "fonts/hershey/hersh.oc1".to_string(),
        "fonts/hershey/hersh.oc2".to_string(),
        "fonts/hershey/hersh.oc3".to_string(),
        "fonts/hershey/hersh.oc4".to_string(),
    ];

    let occidental_glyphs_map = load_glyphs(&occidental_glyph_files)?;
    generate_fonts_map(
        &occidental_glyphs_map,
        "OCCIDENTAL_HERSHEY_FONTS",
        "occidental_fonts.rs",
    )?;

    let oriental_glyph_files = vec![
        "fonts/hershey/hersh.or1".to_string(),
        "fonts/hershey/hersh.or2".to_string(),
        "fonts/hershey/hersh.or3".to_string(),
        "fonts/hershey/hersh.or4".to_string(),
    ];

    let oriental_glyphs_map = load_glyphs(&oriental_glyph_files)?;
    generate_fonts_map(
        &oriental_glyphs_map,
        "ORIENTAL_HERSHEY_FONTS",
        "oriental_fonts.rs",
    )?;

    // Список файлів з офсетами для Occidental шрифтів
    let occidental_offset_files = vec![
        PathBuf::from("fonts/hershey/cyrilc.hmp"),
        PathBuf::from("fonts/hershey/gothgbt.hmp"),
        PathBuf::from("fonts/hershey/gothgrt.hmp"),
        PathBuf::from("fonts/hershey/gothitt.hmp"),
        PathBuf::from("fonts/hershey/greekc.hmp"),
        PathBuf::from("fonts/hershey/greekcs.hmp"),
        PathBuf::from("fonts/hershey/greekp.hmp"),
        PathBuf::from("fonts/hershey/greeks.hmp"),
        PathBuf::from("fonts/hershey/italicc.hmp"),
        PathBuf::from("fonts/hershey/italiccs.hmp"),
        PathBuf::from("fonts/hershey/italict.hmp"),
        PathBuf::from("fonts/hershey/romanc.hmp"),
        PathBuf::from("fonts/hershey/romancs.hmp"),
        PathBuf::from("fonts/hershey/romand.hmp"),
        PathBuf::from("fonts/hershey/romans.hmp"),
        PathBuf::from("fonts/hershey/romant.hmp"),
        PathBuf::from("fonts/hershey/scriptc.hmp"),
        PathBuf::from("fonts/hershey/scripts.hmp"),
    ];

    // Генеруємо мапи для Occidental офсетів
    generate_offsets_map(&occidental_offset_files, "offsets.rs")?;

    let csv_path = "fonts/hershey/unicodemap.csv";
    generate_unicode_map(csv_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Read};

    #[test]
    fn test_parse_glyphs_files_from_file() {
        let glyph_files = vec![
            "fonts/hershey/hersh.oc1".to_string(),
            "fonts/hershey/hersh.oc2".to_string(),
        ];

        let result = load_glyphs(&glyph_files).expect("Failed to load and parse glyphs");

        assert_eq!(result.get(&1).unwrap(), "    1  9MWRMNV RRMVV RPSTS");
        assert_eq!(
            result.get(&2).unwrap(),
            "    2 16MWOMOV ROMSMUNUPSQ ROQSQURUUSVOV"
        );
    }
}
