use std::ops::AddAssign;

use anyhow::Result;
use geo::MultiLineString;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

/// Трейт, що представляє об'єкт, який можна малювати.
pub trait Drawable {
    /// Генерує геометричні шляхи, що представляють об'єкт для малювання.
    ///
    /// # Повертає
    ///
    /// * `Result<MultiLineString<f64>>` - Геометричні шляхи або помилка.
    fn draw(&self) -> Result<MultiLineString<f64>>;
}

/// Структура для представлення малюнка, який складається з набору шляхів.
#[derive(Debug, Clone)]
pub struct Drawing {
    pub paths: MultiLineString<f64>, // Набір шляхів, що складають малюнок.
    pub bounds: (f64, f64),          // Межі малюнка (ширина, висота).
}

impl Drawing {
    /// Створює новий малюнок із вказаними межами та шляхами.
    ///
    /// # Аргументи
    /// * `bounds` - кортеж (ширина, висота) для малюнка.
    /// * `paths` - шляхи, що складають малюнок.
    ///
    /// # Повертає
    /// * Новий екземпляр `Drawing`.
    pub fn new(bounds: (f64, f64), paths: MultiLineString<f64>) -> Self {
        Drawing { paths, bounds }
    }

    /// Генерує SVG-документ із поточного малюнка та повертає його у вигляді рядка.
    ///
    /// # Повертає
    /// Серіалізований в стрічку документ SVG.
    pub fn to_svg(&self) -> String {
        // Логування початку процесу конвертації
        log::info!(
            "Генерація SVG-документа з межами: ширина = {}, висота = {}",
            self.bounds.0,
            self.bounds.1
        );

        // Ініціалізуємо SVG Path дані
        let mut data = Data::new();

        // Проходимо по кожному LineString у MultiLineString
        for line in &self.paths.0 {
            if let Some(first_point) = line.0.first() {
                // Переміщуємося до першої точки шляху
                data = data.move_to((first_point.x, first_point.y));
                // Малюємо лінії до всіх наступних точок
                for point in line.0.iter().skip(1) {
                    data = data.line_to((point.x, point.y));
                }
            }
        }

        // Створюємо SVG Path елемент з згенерованими даними
        let path = Path::new()
            .set("fill", "none") // Без заливки
            .set("stroke", "black") // Чорний колір лінії
            .set("stroke-width", 1) // Товщина лінії
            .set("d", data); // Геометричні дані шляху

        // Створюємо документ SVG з визначеним viewBox
        let document = Document::new()
            .set("viewBox", (0, 0, self.bounds.0, self.bounds.1)) // Встановлюємо viewBox відповідно до меж
            .add(path); // Додаємо шлях до документа

        // Серіалізуємо SVG документ у рядок
        let svg_string = document.to_string();

        // Логування завершення процесу конвертації
        log::info!("SVG-документ успішно згенеровано.");

        svg_string
    }
}

// Реалізація трейту AddAssign для Drawing з об'єктами, що реалізують Drawable
impl<T: Drawable> AddAssign<T> for Drawing {
    fn add_assign(&mut self, drawable: T) {
        match drawable.draw() {
            Ok(drawable_paths) => {
                self.paths.0.extend(drawable_paths.0);
            }
            Err(e) => {
                // Логування помилки
                log::error!("Помилка при додаванні Drawable: {:?}", e);
            }
        }
    }
}
