use std::f64::EPSILON;

use super::segment::Segment;

/// Обчислює максимальну швидкість на куті між двома сегментами.
///
/// # Параметри
/// - `s1`: Перший сегмент.
/// - `s2`: Другий сегмент.
/// - `vmax`: Максимальна швидкість для сегмента.
/// - `a`: Прискорення.
/// - `cf`: Кутовий коефіцієнт (corner factor).
///
/// # Повертає
/// Максимальна швидкість на куті між двома сегментами.
pub fn corner_velocity(s1: &Segment, s2: &Segment, vmax: f64, a: f64, cf: f64) -> f64 {
    let cosine = -s1.vector.dot(s2.vector);

    // Перевірка на майже паралельні сегменти (кут близький до 180 градусів)
    if (cosine - 1.0).abs() < EPSILON {
        return 0.0;
    }

    // Обчислюємо синус кута
    let sine = ((1.0 - cosine) / 2.0).sqrt();

    // Перевірка на майже перпендикулярні сегменти (кут близький до 90 градусів)
    if (sine - 1.0).abs() < EPSILON {
        return vmax;
    }

    // Обчислюємо максимальну швидкість на основі кута і коефіцієнта cf
    let v = ((a * cf * sine) / (1.0 - sine)).sqrt();

    // Повертаємо мінімум між обчисленою швидкістю і максимально дозволеною швидкістю
    v.min(vmax)
}
