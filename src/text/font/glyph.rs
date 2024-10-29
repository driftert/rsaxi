use geo::{coord, AffineOps, AffineTransform, LineString, MultiLineString, Point, Rect};
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

    /// Зміщує гліф на задані відстані по осях X та Y.
    ///
    /// # Аргументи
    ///
    /// * `dx` - Відстань зміщення по осі X.
    /// * `dy` - Відстань зміщення по осі Y.
    ///
    /// # Повертає
    ///
    /// * `Self` - Новий екземпляр гліфа зі зміщеними шляхами та оновленими межами.
    pub fn offset(&self, dx: f64, dy: f64) -> Self {
        // Створюємо афінну трансформацію для зміщення
        let transform = AffineTransform::translate(dx, dy);

        // Застосовуємо трансформацію до всіх шляхів
        let new_paths = self.paths.affine_transform(&transform);

        // Повертаємо новий гліф зі зміщеними шляхами та оновленими межами
        Glyph {
            charcode: self.charcode,
            paths: new_paths,
            xmin: self.xmin + dx,
            xmax: self.xmax + dx,
            ymin: self.ymin + dy,
            ymax: self.ymax + dy,
        }
    }

    /// Масштує гліф на заданий коефіцієнт, зберігаючи пропорції відносно (0, 0).
    ///
    /// # Аргументи
    ///
    /// * `factor` - Коефіцієнт масштабу.
    ///
    /// # Повертає
    ///
    /// * `Self` - Новий екземпляр гліфа з масштабованими шляхами та оновленими межами.
    pub fn scale(&self, factor: f64) -> Self {
        // Визначаємо точку орієнтації для масштабування (0, 0)
        let origin = coord! { x: 0.0, y: 0.0 };

        // Створюємо афінну трансформацію для масштабування з origin як точкою орієнтації
        let scaling = AffineTransform::scale(factor, factor, origin);

        // Застосовуємо трансформацію до всіх шляхів
        let new_paths = self.paths.affine_transform(&scaling);

        // Оновлюємо обмежувальну рамку
        let new_xmin = self.xmin * factor;
        let new_xmax = self.xmax * factor;
        let new_ymin = self.ymin * factor;
        let new_ymax = self.ymax * factor;

        // Повертаємо новий гліф зі масштабованими шляхами та оновленими межами
        Glyph {
            charcode: self.charcode,
            paths: new_paths,
            xmin: new_xmin,
            xmax: new_xmax,
            ymin: new_ymin,
            ymax: new_ymax,
        }
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
                let point = Point::new(x_coordinate as f64, y_coordinate as f64);

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

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger;
    use phf::phf_map;

    fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn test_parse_glyph_a() {
        init_logger();

        // Визначаємо мапу для тесту
        static TEST_CMAP: phf::Map<u32, u32> = phf_map! {
            8u32 => 72u32, // Hershey код 8 -> 'H'
        };

        // Приклад рядка гліфа для букви 'H'
        let glyph_line = "    8  9MWOMOV RUMUV ROQUQ";

        // Парсимо гліф
        let glyph = Glyph::from_line(glyph_line, &TEST_CMAP).expect("Парсинг гліфа не вдався");
        print!("{:?}", glyph);

        // Перевіряємо основні параметри гліфа
        assert_eq!(glyph.charcode, Some(72));
        assert!(!glyph.paths.0.is_empty());

        // Визначаємо очікувані шляхи
        let expected_paths = vec![
            LineString::from(vec![Point::new(-3.0, -5.0), Point::new(-3.0, 4.0)]),
            LineString::from(vec![Point::new(3.0, -5.0), Point::new(3.0, 4.0)]),
            LineString::from(vec![Point::new(-3.0, -1.0), Point::new(3.0, -1.0)]),
        ];

        // Перевіряємо кількість шляхів
        assert_eq!(glyph.paths.0.len(), expected_paths.len());

        // Перевіряємо кожен шлях окремо
        for (parsed, expected) in glyph.paths.0.iter().zip(expected_paths.iter()) {
            assert_eq!(
                parsed.0.len(),
                expected.0.len(),
                "Кількість точок у шляхах не співпадає"
            );
            for (p, e) in parsed.0.iter().zip(expected.0.iter()) {
                assert_eq!(p.x, e.x, "Координата X точки не співпадає");
                assert_eq!(p.y, e.y, "Координата Y точки не співпадає");
            }
        }
    }

    #[test]
    fn test_parse_glyph_a_with_x_offset() {
        init_logger();

        // Define the character map for the test
        static TEST_CMAP: phf::Map<u32, u32> = phf_map! {
            8u32 => 72u32, // Hershey code 8 -> 'H'
        };

        // Sample glyph line for the letter 'H'
        let glyph_line = "    8  9MWOMOV RUMUV ROQUQ";

        // Parse the glyph
        let glyph = Glyph::from_line(glyph_line, &TEST_CMAP).expect("Failed to parse glyph");
        println!("{:?}", glyph);

        // Verify basic glyph parameters
        assert_eq!(glyph.charcode, Some(72));
        assert!(!glyph.paths.0.is_empty());

        // Define expected paths before offset
        let expected_paths_before_offset = vec![
            LineString::from(vec![Point::new(-3.0, -5.0), Point::new(-3.0, 4.0)]),
            LineString::from(vec![Point::new(3.0, -5.0), Point::new(3.0, 4.0)]),
            LineString::from(vec![Point::new(-3.0, -1.0), Point::new(3.0, -1.0)]),
        ];

        // Verify paths before offset
        assert_eq!(glyph.paths.0.len(), expected_paths_before_offset.len());
        for (parsed, expected) in glyph
            .paths
            .0
            .iter()
            .zip(expected_paths_before_offset.iter())
        {
            assert_eq!(
                parsed.0.len(),
                expected.0.len(),
                "Number of points in path does not match"
            );
            for (p, e) in parsed.0.iter().zip(expected.0.iter()) {
                assert_eq!(
                    p.x, e.x,
                    "X coordinate of point does not match before offset"
                );
                assert_eq!(
                    p.y, e.y,
                    "Y coordinate of point does not match before offset"
                );
            }
        }

        // Define offset values (only on x-axis)
        let dx = 2.0;
        let dy = 0.0;

        // Apply the offset
        let offset_glyph = glyph.offset(dx, dy);
        println!("Offset: {:?}", offset_glyph);

        // Define expected paths after x offset
        let expected_paths_after_x_offset = vec![
            LineString::from(vec![Point::new(-1.0, -5.0), Point::new(-1.0, 4.0)]),
            LineString::from(vec![Point::new(5.0, -5.0), Point::new(5.0, 4.0)]),
            LineString::from(vec![Point::new(-1.0, -1.0), Point::new(5.0, -1.0)]),
        ];

        // Verify paths after x offset
        assert_eq!(
            offset_glyph.paths.0.len(),
            expected_paths_after_x_offset.len()
        );
        for (parsed, expected) in offset_glyph
            .paths
            .0
            .iter()
            .zip(expected_paths_after_x_offset.iter())
        {
            assert_eq!(
                parsed.0.len(),
                expected.0.len(),
                "Number of points in path does not match after x offset"
            );
            for (p, e) in parsed.0.iter().zip(expected.0.iter()) {
                assert_eq!(
                    p.x, e.x,
                    "X coordinate of point does not match after x offset"
                );
                assert_eq!(
                    p.y, e.y,
                    "Y coordinate of point should remain unchanged after x offset"
                );
            }
        }

        // Verify that only the x values of bounding box are offset
        assert_eq!(
            offset_glyph.xmin,
            glyph.xmin + dx,
            "xmin does not match after x offset"
        );
        assert_eq!(
            offset_glyph.xmax,
            glyph.xmax + dx,
            "xmax does not match after x offset"
        );
        assert_eq!(
            offset_glyph.ymin, glyph.ymin,
            "ymin should remain unchanged after x offset"
        );
        assert_eq!(
            offset_glyph.ymax, glyph.ymax,
            "ymax should remain unchanged after x offset"
        );
    }

    #[test]
    fn test_scale_glyph_a() {
        init_logger();

        // Визначаємо мапу для тесту
        static TEST_CMAP: phf::Map<u32, u32> = phf_map! {
            8u32 => 72u32, // Hershey код 8 -> 'H'
        };

        // Приклад рядка гліфа для букви 'H'
        let glyph_line = "    8  9MWOMOV RUMUV ROQUQ";

        // Парсимо гліф
        let glyph = Glyph::from_line(glyph_line, &TEST_CMAP).expect("Не вдалося розпарсити гліф");
        println!("Оригінальний гліф: {:?}", glyph);

        // Перевіряємо основні параметри гліфа
        assert_eq!(glyph.charcode, Some(72));
        assert!(!glyph.paths.0.is_empty());

        // Коефіцієнт масштабу
        let scale_factor = 2.0;

        // Застосовуємо масштаб
        let scaled_glyph = glyph.scale(scale_factor);
        println!("Масштабований гліф: {:?}", scaled_glyph);

        // Перевіряємо оновлені шляхи
        let expected_scaled_paths = vec![
            LineString::from(vec![Point::new(-6.0, -10.0), Point::new(-6.0, 8.0)]),
            LineString::from(vec![Point::new(6.0, -10.0), Point::new(6.0, 8.0)]),
            LineString::from(vec![Point::new(-6.0, -2.0), Point::new(6.0, -2.0)]),
        ];

        assert_eq!(scaled_glyph.paths.0.len(), expected_scaled_paths.len());
        for (scaled_path, expected_path) in scaled_glyph
            .paths
            .0
            .iter()
            .zip(expected_scaled_paths.iter())
        {
            assert_eq!(
                scaled_path.0.len(),
                expected_path.0.len(),
                "Кількість точок у шляхах не співпадає після масштабу"
            );
            for (scaled_point, expected_point) in scaled_path.0.iter().zip(expected_path.0.iter()) {
                assert_eq!(
                    scaled_point.x, expected_point.x,
                    "Координата X точки не співпадає після масштабу"
                );
                assert_eq!(
                    scaled_point.y, expected_point.y,
                    "Координата Y точки не співпадає після масштабу"
                );
            }
        }

        // Перевіряємо обмежувальну рамку після масштабу
        assert_eq!(
            scaled_glyph.xmin,
            glyph.xmin * scale_factor,
            "xmin не співпадає після масштабу"
        );
        assert_eq!(
            scaled_glyph.xmax,
            glyph.xmax * scale_factor,
            "xmax не співпадає після масштабу"
        );
        assert_eq!(
            scaled_glyph.ymin,
            glyph.ymin * scale_factor,
            "ymin не співпадає після масштабу"
        );
        assert_eq!(
            scaled_glyph.ymax,
            glyph.ymax * scale_factor,
            "ymax не співпадає після масштабу"
        );
    }
}
