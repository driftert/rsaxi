use super::{error::FontError, glyph::Glyph};
use log::{debug, error, info};
use std::collections::HashMap;

// Include generated maps
include!(concat!(env!("OUT_DIR"), "/offsets.rs"));

/// Представляє шрифт Hershey.
#[derive(Debug, Clone)]
pub struct Font {
    pub name: String,                // Назва шрифту.
    pub glyphs: HashMap<u32, Glyph>, // Карта гліфів для швидкого доступу за символом.
}

impl Font {
    /// Створює новий шрифт із вказаною назвою і вже парсеним набором гліфів.
    ///
    /// # Аргументів
    ///
    /// * `name` - назва шрифту.
    /// * `glyphs` - карта гліфів.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр шрифту.
    fn new(name: String, mut glyphs: HashMap<u32, Glyph>) -> Self {
        // Додаємо константу SPACE для символу пробілу (Unicode U+0020)
        glyphs.insert(0x0020, Glyph::SPACE.clone());
        Font { name, glyphs }
    }

    /// Створює шрифт на основі назви, гліфів та файлу офсетів.
    ///
    /// # Аргументів
    ///
    /// * `name` - назва шрифту.
    /// * `glyphs_map` - мапа гліфів.
    /// * `offsets_filename` - назва файлу офсетів.
    /// * `unicode_map` - мапа відповідностей Hershey кодів та Unicode кодів.
    ///
    /// # Повертає
    ///
    /// * `Result<Self, FontError>` - новий екземпляр шрифту або помилка.
    pub fn from_glyphs_offsets(
        name: &str,
        glyphs_map: &phf::Map<u32, &'static str>,
        offsets_filename: &str,
        unicode_map: &phf::Map<u32, u32>,
    ) -> Result<Self, FontError> {
        info!(
            "Створення шрифту '{}' з файлу офсетів '{}'.",
            name, offsets_filename
        );

        // Отримуємо офсети для даного файлу офсетів безпосередньо з мапи OFFSETS.
        let offsets = OFFSETS.get(offsets_filename).copied().ok_or_else(|| {
            error!("Файл офсетів '{}' не знайдено.", offsets_filename);
            FontError::GenericError(format!("Файл {} не знайдено в офсетах", offsets_filename))
        })?;

        debug!("Отримані офсети: {:?}", offsets);

        // Вибираємо гліфи відповідно до офсетів і одразу створюємо шрифт.
        let selected_glyphs: Vec<&'static str> = offsets
            .iter()
            .filter_map(|&glyph_id| glyphs_map.get(&glyph_id).copied())
            .collect();

        debug!("Вибрані гліфи для створення шрифту: {:?}", selected_glyphs);

        // Створюємо шрифт з відфільтрованих гліфів.
        Font::from_glyphs(name, &selected_glyphs, unicode_map)
    }

    /// Створює шрифт на основі назви та гліфів.
    ///
    /// # Аргументів
    ///
    /// * `name` - назва шрифту.
    /// * `glyphs` - зріз рядків, кожен з яких представляє гліф.
    /// * `cmap` - мапа відповідностей Hershey кодів та Unicode кодів для даної групи.
    ///
    /// # Повертає
    ///
    /// * `Result<Self, FontError>` - новий екземпляр шрифту або помилка.
    pub fn from_glyphs(
        name: &str,
        glyphs: &[&str],
        cmap: &phf::Map<u32, u32>,
    ) -> Result<Self, FontError> {
        info!("Створення шрифту '{}' з наданих гліфів.", name);

        let mut glyph_map = HashMap::new();
        for line in glyphs {
            // Парсимо кожний рядок гліфа, передаючи cmap.
            match Glyph::from_line(line, cmap) {
                Ok(glyph) => {
                    // Перевіряємо, чи визначено charcode. Якщо немає, пропускаємо гліф.
                    if let Some(charcode) = glyph.charcode {
                        glyph_map.insert(charcode, glyph);
                    } else {
                        debug!(
                            "Гліф пропущено, оскільки charcode не було визначено для лінії: '{}'.",
                            line
                        );
                    }
                }
                Err(e) => {
                    error!("Помилка парсингу гліфа для лінії '{}': {:?}", line, e);
                }
            }
        }

        debug!(
            "Шрифт '{}' успішно створено з {} гліфів.",
            name,
            glyph_map.len()
        );

        Ok(Font::new(name.to_string(), glyph_map))
    }

    /// Повертає гліф для вказаного Unicode коду, якщо він існує.
    ///
    /// # Аргументів
    ///
    /// * `charcode` - Unicode код, для якого потрібно знайти гліф.
    ///
    /// # Повертає
    ///
    /// * `Option<&Glyph>` - посилання на гліф або `None`, якщо гліф не знайдено.
    pub fn glyph_by_unicode(&self, charcode: u32) -> Option<&Glyph> {
        self.glyphs
            .values()
            .find(|glyph| glyph.charcode == Some(charcode))
    }
}
