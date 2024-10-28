use geo::{coord, LineString, MultiLineString, Point, Rect};
use log::{debug, error, info};

use crate::text::font::error::FontError;

/// Представляє окремий гліф (символ) шрифту Hershey як набір шляхів.
#[derive(Debug, Clone)]
pub struct Glyph {
    pub charcode: Option<u32>,       // Unicode код символа.
    pub paths: MultiLineString<f64>, // Шляхи, що визначають гліф.
    pub xmin: f64,                   // Мінімальне значення x для гліфа.
    pub xmax: f64,                   // Максимальне значення x для гліфа.
    pub ymin: f64,                   // Мінімальне значення y для гліфа.
    pub ymax: f64,                   // Максимальне значення y для гліфа.
}

impl Glyph {
    pub const SPACE: Glyph = Glyph {
        charcode: Some(32),
        paths: MultiLineString(vec![]), // Порожній масив для пробілу
        xmin: -0.5,
        xmax: 0.5,
        ymin: -0.5,
        ymax: 0.5,
    };

    /// Створює новий гліф із символом, шляхами та опційним Unicode кодом.
    ///
    /// # Аргументи
    ///
    /// * `paths` - шляхи, що визначають гліф.
    /// * `charcode` - опційний Unicode код для цього гліфа.
    /// * `xmin` - мінімальне значення x для гліфа.
    /// * `xmax` - максимальне значення x для гліфа.
    /// * `ymin` - мінімальне значення y для гліфа.
    /// * `ymax` - максимальне значення y для гліфа.
    ///
    /// # Повертає
    ///
    /// * `Self` - новий екземпляр гліфа.
    pub fn new(
        paths: MultiLineString<f64>,
        charcode: Option<u32>,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) -> Self {
        Glyph {
            charcode,
            paths,
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }

    /// Повертає обмежувальну рамку гліфа.
    ///
    /// # Повертає
    ///
    /// * `Rect<f64>` - Прямокутник, що представляє обмежувальну рамку гліфа.
    pub fn bbox(&self) -> Rect<f64> {
        // Створюємо обмежувальну рамку, використовуючи координати xmin, ymin, xmax, ymax
        let rect = Rect::new(
            coord! { x: self.xmin, y: self.ymin },
            coord! { x: self.xmax, y: self.ymax },
        );
        rect
    }

    /// Парсить окремий гліф з рядка і застосовує мапу Unicode для відповідної групи шрифтів.
    ///
    /// # Аргументи
    ///
    /// * `glyph` - рядок, що містить інформацію про гліф.
    /// * `cmap` - мапа відповідностей Hershey кодів та Unicode кодів для даної групи.
    ///
    /// # Повертає
    ///
    /// * `Result<Self, FontError>` - новий гліф або помилка парсингу.
    pub fn from_line(glyph: &str, cmap: &phf::Map<u32, u32>) -> Result<Self, FontError> {
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
        let charcode = cmap.get(&character).copied();
        debug!(
            "Для Hershey коду '{}' знайдено Unicode код: {:?}",
            character, charcode
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

        info!("Ліва межа: {}, Права межа: {}", left_margin, right_margin);

        // Обчислюємо xmin та xmax
        let xmin = left_margin as f64;
        let xmax = right_margin as f64;

        // Обчислюємо ymin та ymax
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;

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
                let point = Point::new((x_coordinate as f64) - xmin, y_coordinate as f64);

                ymin = ymin.min(point.y());
                ymax = ymax.max(point.y());

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

        // Повертаємо новий гліф зі списком шляхів, межами та опційним кодом символа.
        Ok(Glyph::new(
            MultiLineString(paths),
            charcode,
            xmin,
            xmax,
            ymin,
            ymax,
        ))
    }
}
