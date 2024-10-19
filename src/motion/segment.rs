use std::fmt;

use geo::Point;

use super::{block::Block, point::PointExtension};

/// Структура `Segment` представляє геометричний сегмент між двома точками.
/// Вона включає в себе довжину сегмента, напрямний вектор, вхідну швидкість, блоки руху.
pub struct Segment {
    pub p1: Point<f64>,          // Початкова точка сегмента.
    pub p2: Point<f64>,          // Кінцева точка сегмента.
    pub vector: Point<f64>,      // Напрямний вектор між p1 і p2.
    pub length: f64,             // Довжина сегмента.
    pub max_entry_velocity: f64, // Максимальна вхідна швидкість для цього сегмента.
    pub entry_velocity: f64,     // Початкова швидкість входу в сегмент.
    pub blocks: Vec<Block>,      // Блоки руху для цього сегмента.
}

impl Segment {
    /// Створює новий сегмент між двома точками `p1` і `p2`.
    ///
    /// # Параметри
    /// - `p1`: Початкова точка сегмента.
    /// - `p2`: Кінцева точка сегмента.
    ///
    /// # Повертає
    /// Повертає новий екземпляр `Segment` з обчисленою довжиною та напрямним вектором.
    ///
    /// # Зауваження
    /// Якщо точки `p1` і `p2` збігаються, довжина сегмента буде дорівнювати 0, а напрямний вектор буде `(0.0, 0.0)`.
    pub fn new(p1: Point<f64>, p2: Point<f64>) -> Self {
        let length = p1.distance(&p2);
        let vector = (p2 - p1).normalize();

        Self {
            p1,
            p2,
            vector,
            length,
            max_entry_velocity: 0.0,
            entry_velocity: 0.0,
            blocks: vec![],
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Сегмент:\n  Початкова точка: ({:.2}, {:.2})\n  Кінцева точка: ({:.2}, {:.2})\n  Довжина: {:.2}\n  Напрямний вектор: ({:.2}, {:.2})\n  Максимальна вхідна швидкість: {:.2}\n  Початкова швидкість входу: {:.2}\n  Кількість блоків: {}",
            self.p1.x(), self.p1.y(),
            self.p2.x(), self.p2.y(),
            self.length,
            self.vector.x(), self.vector.y(),
            self.max_entry_velocity,
            self.entry_velocity,
            self.blocks.len()
        )
    }
}
