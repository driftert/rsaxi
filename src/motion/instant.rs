use std::fmt;

use geo::Point;

/// Структура представляє стан руху в певний момент часу.
/// Вона включає час, пройдену відстань, швидкість, прискорення та положення в просторі.
pub struct Instant {
    pub time_elapsed: f64,      // Час, що минув з початку руху
    pub distance_traveled: f64, // Пройдена відстань з початку руху
    pub velocity: f64,          // Миттєва швидкість
    pub acceleration: f64,      // Миттєве прискорення
    pub position: Point<f64>,   // Миттєва позиція в просторі (точка)
}

impl Instant {
    /// Створює новий екземпляр `Instant`.
    ///
    /// # Параметри:
    /// - `time_elapsed`: Час, що минув з початку руху.
    /// - `distance_traveled`: Пройдена відстань з початку руху.
    /// - `velocity`: Миттєва швидкість.
    /// - `acceleration`: Миттєве прискорення.
    /// - `position`: Миттєва позиція в просторі.
    pub fn new(
        time_elapsed: f64,
        distance_traveled: f64,
        velocity: f64,
        acceleration: f64,
        position: Point<f64>,
    ) -> Self {
        Self {
            time_elapsed,
            distance_traveled,
            velocity,
            acceleration,
            position,
        }
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Момент {{ час: {:.2}, пройдена відстань: {:.2}, швидкість: {:.2}, прискорення: {:.2}, позиція: ({:.2}, {:.2}) }}",
            self.time_elapsed,
            self.distance_traveled,
            self.velocity,
            self.acceleration,
            self.position.x(),
            self.position.y()
        )
    }
}
