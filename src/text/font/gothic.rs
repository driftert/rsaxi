use super::{
    group::{FontGroup, OCCIDENTAL_FONT_GROUP},
    variant::{
        BaseFontFace, FontVariant, GermanTriplex, GreatBritainTriplex, ItalianTriplex, TypeFace,
    },
};

/// Структура для шрифту Gothic, яка використовує базову композицію з `BaseFontFace`.
#[derive(Debug, Clone)]
pub struct Gothic {
    /// Базовий шрифт.
    base: BaseFontFace,
}

impl Gothic {
    /// Створює новий екземпляр Gothic зі статичними значеннями.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр Gothic.
    pub fn new() -> Self {
        Gothic {
            base: BaseFontFace::new("goth", &OCCIDENTAL_FONT_GROUP),
        }
    }
}

/// Реалізація трейту `TypeFace` для шрифту Gothic.
impl TypeFace for Gothic {
    /// Повертає назву шрифту Gothic.
    fn name(&self) -> &str {
        self.base.name
    }

    /// Повертає групу шрифту Gothic.
    fn group(&self) -> &FontGroup {
        self.base.group
    }
}

/// Реалізація трейту `FontVariant` для шрифту Gothic.
impl FontVariant for Gothic {}

/// Реалізація трейтів варіантів шрифту для Gothic.
impl GreatBritainTriplex for Gothic {}
impl GermanTriplex for Gothic {}
impl ItalianTriplex for Gothic {}
