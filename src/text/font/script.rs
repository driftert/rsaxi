use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{BaseFontFace, Complex, FontVariant, Simplex, TypeFace},
};

/// Структура для шрифту Script, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Script {
    /// Базовий шрифт.
    base: BaseFontFace,
}

impl Script {
    /// Створює новий екземпляр Script зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Script.
    pub fn new() -> Self {
        Script {
            base: BaseFontFace::new("script", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Script.
impl TypeFace for Script {
    /// Повертає назву шрифту Script.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Script.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Script.
impl FontVariant for Script {}

/// Реалізація трейтів варіантів шрифту для Script.
impl Complex for Script {}
impl Simplex for Script {}
