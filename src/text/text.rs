use std::str::FromStr;

use crate::drawing::Drawable;
use crate::text::font::error::FontError;
use anyhow::Result;
use geo::{MultiLineString, Point};
use thiserror::Error;

use super::font::font::Font;
use super::font::glyph::Glyph;

/// Перерахування для визначення горизонтального вирівнювання тексту.
///
/// `TextAlign` задає, як вирівнювати текст відносно вказаної ширини рядка:
/// - `Left`: вирівнювання за лівим краєм.
/// - `Center`: центроване вирівнювання.
/// - `Right`: вирівнювання за правим краєм.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,   // Вирівнювання за лівим краєм.
    Center, // Центроване вирівнювання.
    Right,  // Вирівнювання за правим краєм.
}

impl FromStr for TextAlign {
    type Err = FontError;

    /// Конвертує текстовий рядок у значення `TextAlign`.
    ///
    /// # Аргументи
    ///
    /// * `s` - Рядок, що представляє вирівнювання (`"left"`, `"center"`, `"right"`).
    ///
    /// # Повертає
    ///
    /// * `Result<TextAlign, FontError>` - Успішне значення `TextAlign` або помилка, якщо рядок недопустимий.
    ///
    /// # Приклад
    ///
    /// ```
    /// use std::str::FromStr;
    /// let align = TextAlign::from_str("center").unwrap();
    /// assert_eq!(align, TextAlign::Center);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(TextAlign::Left),
            "center" => Ok(TextAlign::Center),
            "right" => Ok(TextAlign::Right),
            _ => Err(FontError::GenericError(
                "Недопустиме значення align".to_string(),
            )),
        }
    }
}

/// Спеціалізовані помилки для побудови тексту.
#[derive(Debug, Error)]
pub enum TextBuilderError {
    #[error("Не вказано контент тексту")]
    MissingContent,

    #[error("Не вказано шрифт тексту")]
    MissingFont,

    #[error("Не вказано ширину тексту")]
    MissingWidth,

    #[error("Недопустиме значення align")]
    InvalidAlignment,
}

/// Структура для представлення тексту як набору гліфів для малювання.
/// Включає тільки скомпільовані шляхи `glyphs`.
pub struct Text {
    pub glyphs: Vec<Glyph>, // Вектор гліфів, що представляють текст.
}

impl Drawable for Text {
    /// Повертає скомпільовані шляхи для всіх гліфів тексту.
    ///
    /// # Повертає
    ///
    /// * `Result<MultiLineString<f64>>` - об'єднані шляхи всіх гліфів тексту,
    ///   представлені як `MultiLineString<f64>`, або помилка, якщо шляхи не вдалося скомпілювати.
    fn draw(&self) -> Result<MultiLineString<f64>> {
        let paths: Vec<_> = self
            .glyphs
            .iter()
            .flat_map(|glyph| glyph.paths.clone())
            .collect();

        Ok(MultiLineString(paths))
    }
}

/// Будівельник для структури `Text`, що дозволяє налаштовувати різні параметри
/// і автоматично генерувати скомпільовані шляхи для малювання тексту.
#[derive(Default)]
pub struct TextBuilder {
    content: Option<String>,
    font: Option<Font>,
    position: Option<Point<f64>>,
    scale: Option<f64>,
    width: Option<f64>,
    line_height: Option<f64>,
    align: Option<TextAlign>,
    justify: Option<bool>,
}

impl TextBuilder {
    /// Встановлює текстовий контент.
    ///
    /// # Аргумент
    ///
    /// * `content` - текстовий рядок, який буде конвертовано в гліфи.
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
    /// # Аргумент
    ///
    /// * `font` - вибраний шрифт для малювання тексту.
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
    /// # Аргумент
    ///
    /// * `position` - початкова позиція для малювання тексту.
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
    /// # Аргумент
    ///
    /// * `scale` - масштаб для розміру тексту.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим масштабом.
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Встановлює максимальну ширину рядка, після якої текст буде переноситись.
    ///
    /// # Аргумент
    ///
    /// * `width` - максимальна ширина рядка.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленою шириною рядка.
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Встановлює відстань між рядками.
    ///
    /// # Аргумент
    ///
    /// * `line_height` - відстань між рядками тексту.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленою висотою рядка.
    pub fn line_height(mut self, line_height: f64) -> Self {
        self.line_height = Some(line_height);
        self
    }

    /// Встановлює параметр вирівнювання тексту.
    ///
    /// # Аргумент
    ///
    /// * `align` - варіант вирівнювання для тексту (`Left`, `Center`, `Right`).
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим вирівнюванням.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
        self
    }

    /// Встановлює параметр виправлення тексту.
    ///
    /// # Аргумент
    ///
    /// * `justify` - `true`, якщо текст має бути виправленим по ширині.
    ///
    /// # Повертає
    ///
    /// * `TextBuilder` з встановленим параметром виправлення.
    pub fn justify(mut self, justify: bool) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Створює об'єкт `Text`, обробляючи кожен символ тексту та генеруючи скомпільовані шляхи.
    ///
    /// # Повертає
    ///
    /// * `Result<Text, FontError>` - новий екземпляр `Text` зі скомпільованими шляхами або помилка.
    pub fn build(self) -> Result<Text, TextBuilderError> {
        // Отримуємо значення для побудови тексту або повертаємо помилку, якщо щось не вказано
        let content = self.content.ok_or(TextBuilderError::MissingContent)?;
        let font = self.font.ok_or(TextBuilderError::MissingFont)?;
        let width = self.width.ok_or(TextBuilderError::MissingWidth)?;
        let position = self.position.unwrap_or_else(|| Point::new(0.0, 0.0));
        let scale = self.scale.unwrap_or(1.0);
        let line_height = self.line_height.unwrap_or(1.0);
        let align = self.align.unwrap_or(TextAlign::Left);
        let justify = self.justify.unwrap_or(false);

        let mut glyphs = Vec::new();
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut line_width = 0.0;

        // Обробка кожного символу послідовно, включаючи пробіли
        for char in content.chars() {
            if let Some(glyph) = font.glyph_by_unicode(char as u32) {
                let scaled_glyph = glyph.scale(scale);
                let glyph_width = scaled_glyph.bbox().width();

                // Якщо ширина рядка перевищує максимальну ширину, переносимо його на новий рядок
                if line_width + glyph_width > width {
                    lines.push(line);
                    line = Vec::new();
                    line_width = 0.0;
                }

                // Додаємо гліф до поточного рядка
                line.push(scaled_glyph);
                line_width += glyph_width;
            } else if char == ' ' {
                // Якщо символ — пробіл, додаємо пробіл як окремий гліф із шириною пробілу
                let space_glyph = Glyph::SPACE.scale(scale);
                let space_width = space_glyph.bbox().width();

                // Якщо ширина рядка перевищує максимальну ширину, переносимо його на новий рядок
                if line_width + space_width > width {
                    lines.push(line);
                    line = Vec::new();
                    line_width = 0.0;
                }

                // Додаємо пробіл до поточного рядка
                line.push(space_glyph);
                line_width += space_width;
            }
        }

        // Додаємо останній рядок, якщо залишилися гліфи
        if !line.is_empty() {
            lines.push(line);
        }

        // Обробляємо кожен рядок з вирівнюванням та виправленням по ширині, якщо потрібно
        let mut y_offset = 0.0;
        for line in lines {
            let aligned_line = if justify && line.len() > 1 {
                TextBuilder::justify_line(&line, width)
            } else {
                TextBuilder::align_line(&line, width, align)
            };

            // Зсув для кожного гліфа відповідно до початкової позиції
            let mut x_offset = 0.0;
            for mut glyph in aligned_line {
                let width = glyph.bbox().width();
                glyph = glyph.offset(position.x() + x_offset, position.y() - y_offset);
                glyphs.push(glyph);
                x_offset += width;
            }

            // Зсуваємо позицію для наступного рядка
            y_offset += line_height;
        }

        // Повертаємо об'єкт `Text` з усіма сформованими гліфами
        Ok(Text { glyphs })
    }

    /// Вирівнює рядок символів по ширині, додаючи рівномірний відступ між символами.
    ///
    /// # Аргументи
    ///
    /// * `line` - Вектор зсунутих гліфів для рядка тексту.
    /// * `width` - Ширина, до якої потрібно вирівняти рядок.
    ///
    /// # Повертає
    ///
    /// * `Vec<Glyph>` - Новий вектор з вирівняними гліфами.
    fn justify_line(line: &[Glyph], width: f64) -> Vec<Glyph> {
        // Розрахунок загальної ширини гліфів без відступів
        let line_width: f64 = line.iter().map(|glyph| glyph.bbox().width()).sum();

        // Розрахунок інтервалу між гліфами
        let spaces = line.len() - 1;
        let gap = if spaces > 0 {
            (width - line_width) / spaces as f64
        } else {
            0.0
        };

        // Додаємо відступ до кожного гліфа
        let mut justified_line = Vec::with_capacity(line.len());
        let mut x_offset = 0.0;
        for glyph in line {
            // Застосовуємо зсув та додаємо гліф з новою позицією
            let positioned_glyph = glyph.offset(x_offset, 0.0);
            justified_line.push(positioned_glyph);
            x_offset += glyph.bbox().width() + gap;
        }

        justified_line
    }

    /// Вирівнює рядок гліфів відповідно до заданого типу вирівнювання.
    ///
    /// Функція обчислює ширину рядка гліфів, а потім додає зсув до кожного гліфа
    /// відповідно до обраного вирівнювання (`Left`, `Center`, `Right`).
    ///
    /// # Аргументи
    ///
    /// * `line` - Зріз, що містить гліфи для вирівнювання.
    /// * `width` - Загальна ширина рядка, до якої потрібно вирівняти гліфи.
    /// * `align` - Тип вирівнювання для рядка:
    ///     - `TextAlign::Left`: вирівнювання по лівому краю.
    ///     - `TextAlign::Center`: центрування рядка.
    ///     - `TextAlign::Right`: вирівнювання по правому краю.
    ///
    /// # Повертає
    ///
    /// * `Vec<Glyph>` - Новий вектор з гліфами, які зміщені відповідно до заданого вирівнювання.
    fn align_line(line: &[Glyph], width: f64, align: TextAlign) -> Vec<Glyph> {
        let line_width: f64 = line.iter().map(|g| g.bbox().width()).sum();
        let offset = match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (width - line_width) / 2.0,
            TextAlign::Right => width - line_width,
        };

        line.iter().map(|g| g.offset(offset, 0.0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::drawing::Drawing;
    use crate::text::font::roman::Roman;
    use crate::text::font::variant::Simplex;

    use super::*;
    use env_logger;
    use std::fs::File;
    use std::io::Write;

    fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn test_create_text_with_roman_font_and_save_svg() {
        init_logger();

        // Створення тексту з шрифтом `Roman`
        let roman_font = Roman::new()
            .simplex()
            .expect("Не вдалося ініціалізувати шрифт `Roman`");
        let content = "A Z";

        let text = TextBuilder::default()
            .content(content)
            .scale(0.5)
            .font(roman_font)
            .width(200.0)
            .line_height(10.0)
            .align(TextAlign::Center)
            .build()
            .expect("Не вдалося створити текст");

        // Генерація SVG-документа
        let drawing = Drawing::new(
            (400.0, 200.0),
            text.draw().expect("Не вдалося згенерувати шляхи"),
        );
        let svg_data = drawing.to_svg();

        // Запис SVG у файл
        let mut file = File::create("test_output.svg").expect("Не вдалося створити SVG файл");
        file.write_all(svg_data.as_bytes())
            .expect("Не вдалося записати в SVG файл");

        // Перевірка наявності даних у згенерованому SVG
        assert!(!svg_data.is_empty(), "SVG файл порожній");
    }
}
