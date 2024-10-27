use std::{collections::HashMap, sync::Mutex};

use log::{debug, info};
use once_cell::sync::Lazy;

use super::font::Font;

/// Статична структура для кешування шрифтів.
pub struct Fonts {
    cache: Mutex<HashMap<String, Font>>, // Глобальний кеш для збереження вже оброблених шрифтів.
}

/// Статичний екземпляр кешу шрифтів.
static FONTS: Lazy<Fonts> = Lazy::new(|| Fonts {
    cache: Mutex::new(HashMap::new()),
});

impl Fonts {
    /// Отримує шрифт з кешу за назвою або `None`, якщо шрифт не знайдено.
    ///
    /// # Аргументів
    ///
    /// * `font_name` - назва шрифту для отримання.
    ///
    /// # Повертає
    ///
    /// * `Option<Font>` - знайдений шрифт або `None`.
    pub fn get(font_name: &str) -> Option<Font> {
        let cache = FONTS.cache.lock().unwrap();
        cache.get(font_name).cloned()
    }

    /// Додає шрифт у кеш.
    ///
    /// # Аргументів
    ///
    /// * `font_name` - назва шрифту.
    /// * `font` - екземпляр шрифту для додавання.
    pub fn insert(font_name: String, font: Font) {
        let mut cache = FONTS.cache.lock().unwrap();
        cache.insert(font_name.clone(), font);
        debug!("Шрифт '{}' додано до кешу.", font_name);
    }

    /// Очищує весь кеш шрифтів.
    pub fn clear() {
        let mut cache = FONTS.cache.lock().unwrap();
        cache.clear();
        info!("Кеш шрифтів очищено.");
    }
}
