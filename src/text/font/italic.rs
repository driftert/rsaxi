use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{BaseFontFace, Complex, ComplexSmall, FontVariant, Triplex, TypeFace},
};

/// Структура для шрифту Italic, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Italic {
    /// Базовий шрифт.
    base: BaseFontFace,
}

impl Italic {
    /// Створює новий екземпляр Italic зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Italic.
    pub fn new() -> Self {
        Italic {
            base: BaseFontFace::new("italic", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Italic.
impl TypeFace for Italic {
    /// Повертає назву шрифту Italic.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Italic.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Italic.
impl FontVariant for Italic {}

/// Реалізація трейтів варіантів шрифту для Italic.
impl Complex for Italic {}
impl ComplexSmall for Italic {}
impl Triplex for Italic {}
