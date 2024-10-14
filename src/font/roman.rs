use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{
        BaseFontFace, Complex, ComplexSmall, Duplex, FontVariant, Simplex, Triplex, TypeFace,
    },
};

/// Структура для шрифту Roman, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Roman {
    /// Базовий шрифт.
    base: BaseFontFace,
}

impl Roman {
    /// Створює новий екземпляр Roman зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Roman.
    pub fn new() -> Self {
        Roman {
            base: BaseFontFace::new("roman", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Roman.
impl TypeFace for Roman {
    /// Повертає назву шрифту Roman.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Roman.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Roman.
impl FontVariant for Roman {}

/// Реалізація трейтів варіантів шрифту для Roman.
impl Complex for Roman {}
impl ComplexSmall for Roman {}
impl Duplex for Roman {}
impl Simplex for Roman {}
impl Triplex for Roman {}
