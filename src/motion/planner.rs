use geo::Point;

use super::{error::PlanError, plan::Plan};

/// Структура `Planner` відповідає за планування руху для AxiDraw.
/// Вона використовує профіль швидкості та контроль інструменту для обчислення шляхів руху.
pub struct Planner {
    max_velocity: f64,  // Максимальна швидкість, яку може досягти під час руху.
    acceleration: f64,  // Максимальне прискорення, яке може бути застосоване під час руху.
    corner_factor: f64, // Фактор для корекції швидкості на поворотах.
}

impl Planner {
    /// Створює новий `Planner` з заданими параметрами.
    ///
    /// # Параметри:
    /// - `max_velocity`: Максимальна швидкість руху.
    /// - `max_acceleration`: Максимальне прискорення руху.
    /// - `corner_factor`: Фактор для корекції швидкості на поворотах.
    pub fn new(max_velocity: f64, acceleration: f64, corner_factor: f64) -> Self {
        Self {
            max_velocity,
            acceleration,
            corner_factor,
        }
    }

    /// Створює новий план руху на основі наданих точок.
    ///
    /// # Параметри:
    /// - `points`: Вектор точок `Point<f64>`, що визначає шлях руху.
    ///
    /// # Повертає:
    /// - `Result<Plan, PlanError>`: Результат плану руху або помилка.
    pub fn plan(&self, points: Vec<Point<f64>>) -> Result<Plan, PlanError> {
        Plan::new(
            points,
            vec![],
            vec![],
            self.acceleration,
            self.max_velocity,
            self.corner_factor,
        )
    }
}
