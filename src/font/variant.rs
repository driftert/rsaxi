use log::{debug, error};

use crate::font::cache::Fonts;

use super::{error::FontError, font::Font, group::FontGroup};

/// Трейт, що представляє шрифт з його групою та унікальною назвою.
pub trait TypeFace {
    /// Повертає назву шрифту (наприклад, "roman", "gothic", "italic").
    fn name(&self) -> &str;

    /// Повертає групу шрифту, яка містить відповідні файли.
    fn group(&self) -> &FontGroup;
}

/// Базова структура для всіх шрифтів, яка містить назву та групу.
#[derive(Debug, Clone)]
pub struct BaseFontFace {
    pub name: &'static str,        // Назва шрифту.
    pub group: &'static FontGroup, // Посилання на групу шрифтів.
}

impl BaseFontFace {
    /// Створює новий базовий шрифт з вказаною назвою та групою.
    ///
    /// # Аргументів
    ///
    /// * `name` - назва шрифту.
    /// * `group` - група шрифту, до якої належить цей шрифт.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр базового шрифту.
    pub fn new(name: &'static str, group: &'static FontGroup) -> Self {
        BaseFontFace { name, group }
    }
}

/// Базовий трейт для варіантів шрифтів.
pub trait FontVariant: TypeFace {
    /// Створює шрифт з додаванням суфікса до назви файлу офсетів.
    ///
    /// # Аргументів
    ///
    /// * `suffix` - суфікс, який додається до назви шрифту для формування назви файлу офсетів.
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn font_with_suffix(&self, suffix: &str) -> Result<Font, FontError> {
        let name = self.name();
        let full_name = format!("{}{}", name, suffix);

        // Перевіряємо, чи є шрифт у кеші.
        if let Some(cached_font) = Fonts::get(&full_name) {
            debug!("Шрифт '{}' знайдено у кеші.", full_name);
            return Ok(cached_font);
        }

        // Якщо шрифту немає в кеші, формуємо назву файлу офсетів.
        let offsets_filename = format!("{}{}", name, suffix);
        debug!(
            "Шрифт '{}' не знайдено в кеші. Створюємо з файлу офсетів '{}'.",
            full_name, offsets_filename
        );

        // Отримуємо групу шрифтів.
        let font_group = self.group();

        // Створюємо шрифт з гліфів, файлу офсетів та мапи Unicode.
        let font = Font::from_glyphs_offsets(
            name,
            font_group.fonts,
            &offsets_filename,
            font_group.unicode_map, // Передаємо мапу Unicode
        )
        .map_err(|e| {
            error!(
                "Помилка створення шрифту з файлу офсетів '{}': {}",
                offsets_filename, e
            );
            FontError::FontCreationError {
                file: offsets_filename,
                source: Box::new(e),
            }
        })?;

        // Додаємо створений шрифт у кеш.
        Fonts::insert(full_name.clone(), font.clone());
        debug!("Шрифт '{}' успішно додано до кешу.", full_name);

        Ok(font)
    }
}

/// Трейт для Plain, що наслідує від FontVariant.
pub trait Plain: FontVariant {
    /// Створює простий варіант шрифту (Plain).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn plain(&self) -> Result<Font, FontError> {
        self.font_with_suffix("p")
    }
}

/// Трейт для Simplex, що наслідує від FontVariant.
pub trait Simplex: FontVariant {
    /// Створює простий варіант шрифту (Simplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn simplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("s")
    }
}

/// Трейт для Duplex, що наслідує від FontVariant.
pub trait Duplex: FontVariant {
    /// Створює двосторонній варіант шрифту (Duplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn duplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("d")
    }
}

/// Трейт для Complex, що наслідує від FontVariant.
pub trait Complex: FontVariant {
    /// Створює складний варіант шрифту (Complex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn complex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("c")
    }
}

/// Трейт для Triplex, що наслідує від FontVariant.
pub trait Triplex: FontVariant {
    /// Створює трифазний варіант шрифту (Triplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn triplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("t")
    }
}

/// Трейт для ComplexSmall, що наслідує від FontVariant.
pub trait ComplexSmall: FontVariant {
    /// Створює складний малий варіант шрифту (ComplexSmall).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn complex_small(&self) -> Result<Font, FontError> {
        self.font_with_suffix("cs")
    }
}

/// Трейт для ItalianTriplex, що наслідує від FontVariant.
pub trait ItalianTriplex: FontVariant {
    /// Створює італійський трифазний варіант шрифту (ItalianTriplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn italian_triplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("itt")
    }
}

/// Трейт для GreatBritainTriplex, що наслідує від FontVariant.
pub trait GreatBritainTriplex: FontVariant {
    /// Створює британський трифазний варіант шрифту (GreatBritainTriplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn great_britain_triplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("gbt")
    }
}

/// Трейт для GermanTriplex, що наслідує від FontVariant.
pub trait GermanTriplex: FontVariant {
    /// Створює німецький трифазний варіант шрифту (GermanTriplex).
    ///
    /// # Повертає
    ///
    /// * `Result<Font, FontError>` - новий екземпляр шрифту або помилка.
    fn german_triplex(&self) -> Result<Font, FontError> {
        self.font_with_suffix("grt")
    }
}
