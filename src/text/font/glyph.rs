use geo::{LineString, MultiLineString, Point};
use log::{debug, error, info};

use crate::text::font::error::FontError;

/// Представляє окремий гліф (символ) шрифту Hershey як набір шляхів.
#[derive(Debug, Clone)]
pub struct Glyph {
    pub code: u32,                   // Код, який представляє цей гліф.
    pub paths: MultiLineString<f64>, // Шляхи, що визначають гліф.
    pub width: f64,                  // Ширина гліфа.
    pub unicode_code: Option<u32>,   // Опційний Unicode код для цього гліфа.
}

impl Glyph {
    /// Створює новий гліф із символом, шляхами та опційним Unicode кодом.
    ///
    /// # Аргументи
    ///
    /// * `code` - код гліфа.
    /// * `paths` - шляхи, що визначають гліф.
    /// * `width` - ширина гліфа.
    /// * `unicode_code` - опційний Unicode код для цього гліфа.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр гліфа.
    pub fn new(
        code: u32,
        paths: MultiLineString<f64>,
        width: f64,
        unicode_code: Option<u32>,
    ) -> Self {
        Glyph {
            code,
            paths,
            width,
            unicode_code,
        }
    }

    /// Парсить окремий гліф з рядка і застосовує мапу Unicode для відповідної групи шрифтів.
    ///
    /// # Аргументи
    ///
    /// * `glyph` - рядок, що містить інформацію про гліф.
    /// * `unicode_map` - мапа відповідностей Hershey кодів та Unicode кодів для даної групи.
    ///
    /// # Повертає
    ///
    /// * `Result<Self, FontError>` - новий гліф або помилка парсингу.
    pub fn from_line(glyph: &str, unicode_map: &phf::Map<u32, u32>) -> Result<Self, FontError> {
        debug!("Парсимо гліф із рядка: {}", glyph);

        // Перевіряємо мінімальну довжину рядка, необхідну для парсингу.
        if glyph.len() < 10 {
            error!("Помилка парсингу: рядок занадто короткий для гліфа.");
            return Err(FontError::GlyphParsingError {
                glyph: glyph.to_string(),
                message: "Рядок занадто короткий для парсингу гліфа".to_string(),
            });
        }

        // Парсимо номер гліфа (перші п'ять символів).
        let character_str = &glyph[0..5].trim();
        debug!("Парсимо символ з рядка: '{}'", character_str);

        // Парсимо значення символа як число (u32)
        let character = character_str.parse::<u32>().map_err(|_| {
            error!(
                "Неможливо парсити символ гліфа '{}' як число.",
                character_str
            );
            FontError::GlyphParsingError {
                glyph: glyph.to_string(),
                message: "Неможливо парсити символ гліфа як число".to_string(),
            }
        })?;

        info!("Гліф для символа '{}' успішно оброблено.", character);

        // Перевіряємо, чи існує відповідний Unicode код для цього Hershey коду
        let unicode_code = unicode_map.get(&character).copied();
        debug!(
            "Для Hershey коду '{}' знайдено Unicode код: {:?}",
            character, unicode_code
        );

        // Парсимо кількість вершин (символи з 5 до 8).
        let num_vertices_str = &glyph[5..8].trim();
        let num_vertices = num_vertices_str.parse::<usize>().map_err(|_| {
            error!("Неможливо парсити кількість вершин '{}'.", num_vertices_str);
            FontError::GlyphParsingError {
                glyph: glyph.to_string(),
                message: "Неможливо парсити кількість вершин".to_string(),
            }
        })?;

        debug!("Кількість вершин: {}", num_vertices);

        // Парсимо ліву межу (символ на позиції 8) і праву межу (символ на позиції 9).
        let left_margin = (glyph.chars().nth(8).ok_or(FontError::GlyphParsingError {
            glyph: glyph.to_string(),
            message: "Неможливо отримати символ лівої межі".to_string(),
        })? as i32)
            - ('R' as i32);

        let right_margin = (glyph.chars().nth(9).ok_or(FontError::GlyphParsingError {
            glyph: glyph.to_string(),
            message: "Неможливо отримати символ правої межі".to_string(),
        })? as i32)
            - ('R' as i32);

        let glyph_width = (right_margin - left_margin) as f64;

        info!(
            "Ліва межа: {}, Права межа: {}, Ширина гліфа: {}",
            left_margin, right_margin, glyph_width
        );

        // Парсимо координати точок і шляхи.
        let mut paths = Vec::new(); // Містить шляхи для поточного гліфа.
        let mut current_points = Vec::new(); // Містить точки для поточного шляху.
        let mut is_pen_down = true; // Визначає, чи активний режим малювання (перо опущене).
        let mut current_index = 10; // Початкова позиція для парсингу координат.

        // Проходимо по рядку, витягуючи координати пар точок.
        while current_index + 1 < glyph.len() {
            let x_char = glyph
                .chars()
                .nth(current_index)
                .ok_or(FontError::GlyphParsingError {
                    glyph: glyph.to_string(),
                    message: "Неможливо отримати координату x".to_string(),
                })?;
            let y_char =
                glyph
                    .chars()
                    .nth(current_index + 1)
                    .ok_or(FontError::GlyphParsingError {
                        glyph: glyph.to_string(),
                        message: "Неможливо отримати координату y".to_string(),
                    })?;

            // Якщо зустрічаємо " R", це означає підйом пера.
            if x_char == ' ' && y_char == 'R' {
                if !current_points.is_empty() {
                    info!(
                        "Закінчення шляху: кількість точок = {}",
                        current_points.len()
                    );
                    paths.push(LineString::from(
                        current_points.iter().cloned().collect::<Vec<Point<f64>>>(),
                    ));
                    current_points.clear(); // Очищаємо точки для нового шляху.
                }
                is_pen_down = false;
                info!("Ручка піднята.");
            } else {
                debug!(
                    "Обчислюємо координати відносно 'R': x = {}, y = {}",
                    x_char, y_char
                );
                let x_coordinate = (x_char as i32) - ('R' as i32);
                let y_coordinate = (y_char as i32) - ('R' as i32);
                let point = Point::new(x_coordinate as f64, y_coordinate as f64);

                if is_pen_down {
                    debug!("Додана точка: x = {}, y = {}", x_coordinate, y_coordinate);
                    current_points.push(point);
                } else {
                    is_pen_down = true;
                    info!("Ручка опущена. Початок нового шляху.");
                    current_points.push(point);
                }
            }

            // Переміщуємо індекс для наступної пари координат.
            current_index += 2;
        }

        // Якщо є залишкові точки після завершення рядка, додаємо їх як шлях.
        if !current_points.is_empty() {
            paths.push(LineString::from(
                current_points.iter().cloned().collect::<Vec<Point<f64>>>(),
            ));
            debug!(
                "Додано завершальний шлях з кількістю точок = {}",
                current_points.len()
            );
        }

        info!("Гліф успішно парсено для символа '{}'.", character);

        // Повертаємо новий гліф зі списком шляхів та шириною.
        Ok(Glyph::new(
            character,
            MultiLineString(paths), // Використовуємо конструктор без From
            glyph_width,
            unicode_code,
        ))
    }
}
