use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{BaseFontFace, Complex, ComplexSmall, FontVariant, Plain, Simplex, TypeFace},
};

/// Структура для шрифту Greek, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Greek {
    /// Базовий шрифт.
    base: BaseFontFace,
}

impl Greek {
    /// Створює новий екземпляр Greek зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Greek.
    pub fn new() -> Self {
        Greek {
            base: BaseFontFace::new("greek", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Greek.
impl TypeFace for Greek {
    /// Повертає назву шрифту Greek.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Greek.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Greek.
impl FontVariant for Greek {}

/// Реалізація трейтів варіантів шрифту для Greek.
impl Plain for Greek {}
impl Complex for Greek {}
impl ComplexSmall for Greek {}
impl Simplex for Greek {}
