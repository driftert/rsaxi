use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{BaseFontFace, Complex, FontVariant, TypeFace},
};

/// Структура для шрифту Cyrilic, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Cyrilic {
    base: BaseFontFace, // Базовий шрифт.
}

impl Cyrilic {
    /// Створює новий екземпляр Cyrilic зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Cyrilic.
    pub fn new() -> Self {
        Cyrilic {
            base: BaseFontFace::new("cyril", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Cyrilic.
impl TypeFace for Cyrilic {
    /// Повертає назву шрифту Cyrilic.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Cyrilic.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Cyrilic.
impl FontVariant for Cyrilic {}

/// Реалізація трейтів варіантів шрифту для Cyrilic.
impl Complex for Cyrilic {}
