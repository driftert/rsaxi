use crate::drawing::Drawable;
use crate::font::font::Font;
use crate::font::{error::FontError, glyph::Glyph};
use anyhow::Result;
use geo::algorithm::scale::Scale;
use geo::algorithm::translate::Translate;
use geo::{MultiLineString, Point};
use log::{debug, info};

/// Структура для представлення тексту як набору гліфів для малювання.
pub struct Text {
    pub glyphs: Vec<Glyph>,   // Набір гліфів, що складають текст.
    pub position: Point<f64>, // Позиція початку малювання тексту.
    pub scale: f64,           // Масштаб тексту.
    pub spacing: f64,         // Відстань між символами.
}

impl Text {
    /// Returns a `TextBuilder` for constructing a `Text` instance.
    ///
    /// # Returns
    ///
    /// * `TextBuilder` - A new `TextBuilder` instance.
    pub fn builder() -> TextBuilder {
        TextBuilder::default()
    }

    /// Виконує перенесення тексту на нові рядки, якщо його ширина перевищує задану максимальну ширину.
    /// Повертає новий екземпляр `Text` з модифікованим розташуванням гліфів.
    ///
    /// # Аргумент
    ///
    /// * `max_width` - максимальна ширина для переносу рядка.
    ///
    /// # Повертає
    ///
    /// * `Result<Self, FontError>` - новий екземпляр `Text` або помилка у разі невдачі.
    pub fn wrap(&self, max_width: f64) -> Result<Self, FontError> {
        debug!(
            "Перенесення тексту з максимальною шириною: {}, позиція: {:?}, масштаб: {}",
            max_width, self.position, self.scale
        );
        todo!()
    }
}

impl Drawable for Text {
    /// Генерує геометричні шляхи, що представляють текст.
    ///
    /// # Повертає
    ///
    /// * `Result<MultiLineString<f64>>` - Геометричні шляхи або помилка.
    fn draw(&self) -> Result<MultiLineString<f64>> {
        info!(
            "Малювання тексту з позицією: {:?}, масштабом: {}, відстанню між символами: {}",
            self.position, self.scale, self.spacing
        );

        let mut paths = Vec::new();

        // Початкова позиція для малювання тексту
        let mut current_x = self.position.x();
        let current_y = self.position.y();

        for glyph in &self.glyphs {
            let glyph_paths = &glyph.paths;

            // Масштабуємо гліф
            let scaled_glyph = glyph_paths.scale(self.scale);

            // Переносимо гліф до поточної позиції
            let translated_glyph = scaled_glyph.translate(current_x, current_y);

            // Додаємо трансформовані шляхи гліфу до загальних шляхів
            paths.extend(translated_glyph.0.into_iter());

            // Оновлюємо позицію X для наступного гліфу
            current_x += glyph.width * self.scale;
        }

        Ok(MultiLineString(paths))
    }
}

/// Будівельник для структури `Text`.
#[derive(Default)]
pub struct TextBuilder {
    content: Option<String>,
    font: Option<Font>,
    position: Option<Point<f64>>,
    scale: Option<f64>,
    spacing: Option<f64>,
}

impl TextBuilder {
    /// Встановлює текстовий контент.
    ///
    /// # Аргументів
    ///
    /// * `content` - текстовий рядок.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим контентом.
    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    /// Встановлює шрифт для тексту.
    ///
    /// # Аргументів
    ///
    /// * `font` - вибраний шрифт.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим шрифтом.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Встановлює початкову позицію тексту.
    ///
    /// # Аргументів
    ///
    /// * `position` - початкова позиція тексту.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленою позицією.
    pub fn position(mut self, position: Point<f64>) -> Self {
        self.position = Some(position);
        self
    }

    /// Встановлює масштаб тексту.
    ///
    /// # Аргументів
    ///
    /// * `scale` - масштаб тексту.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим масштабом.
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Встановлює відстань між символами.
    ///
    /// # Аргументів
    ///
    /// * `spacing` - відстань між символами.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим відступом між символами.
    pub fn spacing(mut self, spacing: f64) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Створює об'єкт `Text` на основі встановлених параметрів.
    ///
    /// # Повертає
    ///
    /// * `Result<Text, FontError>` - новий екземпляр `Text` або помилка `FontError`.
    ///
    /// # Приклад
    ///
    /// ```rust
    /// let text = Text::builder()
    ///     .content("Hello, World!")
    ///     .font(font)
    ///     .position(Point::new(0.0, 0.0))
    ///     .scale(1.5)
    ///     .spacing(2.0)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<Text, FontError> {
        let content = self.content.ok_or(FontError::GenericError(
            "Не вказано контент тексту".to_string(),
        ))?;
        let font = self.font.ok_or(FontError::GenericError(
            "Не вказано шрифт тексту".to_string(),
        ))?;
        let position = self.position.unwrap_or_else(|| Point::new(0.0, 0.0));
        let scale = self.scale.unwrap_or(1.0);
        let spacing = self.spacing.unwrap_or(2.0); // Відстань за замовчуванням між символами

        debug!(
            "Створення тексту з параметрами: контент: '{}', позиція: {:?}, масштаб: {}, відстань між символами: {}",
            content, position, scale, spacing
        );

        let glyphs: Vec<Glyph> = content
            .chars()
            .filter_map(|c| font.glyph_by_unicode(c as u32))
            .cloned()
            .collect();

        if glyphs.is_empty() {
            return Err(FontError::GenericError(format!(
                "Не знайдено гліфів для тексту '{}'",
                content
            )));
        }

        Ok(Text {
            glyphs,
            position,
            scale,
            spacing,
        })
    }
}
