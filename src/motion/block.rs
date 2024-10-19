use std::fmt;

use geo::Point;

use super::{instant::Instant, point::PointExtension};

/// Представляє один сегмент руху.
/// Він містить інформацію про початкову швидкість, прискорення та тривалість руху.
#[derive(Clone)]
pub struct Block {
    pub acceleration: f64, // Прискорення, яке застосовується під час цього блоку руху.
    pub duration: f64,     // Тривалість блоку руху (в секундах).
    pub initial_velocity: f64, // Початкова швидкість на початку блоку.
    pub distance: f64,     // Відстань між початковою і кінцевою точками
    pub p1: Point<f64>,    // Початкова точка сегмента.
    pub p2: Point<f64>,    // Кінцева точка сегмента.
}

impl Block {
    /// Створює новий блок руху.
    pub fn new(
        acceleration: f64,
        duration: f64,
        initial_velocity: f64,
        p1: Point<f64>,
        p2: Point<f64>,
    ) -> Self {
        let distance = p1.distance(&p2);
        Self {
            acceleration,
            duration,
            initial_velocity,
            distance,
            p1,
            p2,
        }
    }

    /// Обчислює миттєвий стан об'єкта руху у певний момент часу `t`.
    ///
    /// Цей метод обчислює поточний час, відстань, швидкість, прискорення і позицію об'єкта
    /// на основі рівноприскореного руху.
    ///
    /// # Параметри
    /// - `t`: Час, на якому потрібно обчислити стан. Значення обмежується діапазоном [0, `duration`].
    /// - `dt`: Додатковий приріст часу, який додається до результату.
    /// - `ds`: Додатковий приріст відстані, який додається до результату.
    ///
    /// # Повертає
    /// Миттєвий стан (`Instant`) об'єкта в момент часу `t`, включаючи:
    /// - Відстань, яку пройшов об'єкт до цього моменту.
    /// - Поточну швидкість об'єкта.
    /// - Поточне прискорення.
    /// - Позицію об'єкта на відрізку між початковою і кінцевою точками.
    pub fn instant(&self, t: f64, dt: f64, ds: f64) -> Instant {
        let clamped_t = t.clamp(0.0, self.duration);
        let a = self.acceleration;
        let v = self.initial_velocity + a * clamped_t;
        let s = self.initial_velocity * clamped_t + 0.5 * a * clamped_t * clamped_t;
        let clamped_s = s.clamp(0.0, self.distance);
        let position = self.p1.lerps(&self.p2, clamped_s);
        Instant::new(clamped_t + dt, clamped_s + ds, v, a, position)
    }

    /// Обчислює миттєвий стан об'єкта руху, коли він пройшов певну відстань `s`.
    ///
    /// Цей метод знаходить момент часу, коли об'єкт пройшов відстань `s`,
    /// і викликає метод `instant` для обчислення відповідного стану.
    ///
    /// # Параметри
    /// - `s`: Відстань, яку пройшов об'єкт.
    /// - `dt`: Додатковий приріст часу, який додається до результату.
    /// - `ds`: Додатковий приріст відстані, який додається до результату.
    ///
    /// # Повертає
    /// Миттєвий стан (`Instant`) об'єкта на момент часу, коли об'єкт пройшов відстань `s`.
    pub fn instant_at_distance(&self, s: f64, dt: f64, ds: f64) -> Instant {
        if s <= 0.0 {
            // Якщо відстань менша або дорівнює нулю, повертаємо початковий стан
            return self.instant(0.0, dt, ds);
        }
        if s >= self.distance {
            // Якщо відстань більша або дорівнює повній довжині сегмента, повертаємо кінцевий стан
            return self.instant(self.duration, dt, ds);
        }

        // Обчислюємо кінцеву швидкість при пройденій відстані s
        let vf = (self.initial_velocity.powi(2) + 2.0 * self.acceleration * s).sqrt();

        // Обчислюємо час t за формулою рівноприскореного руху
        let t = (2.0 * s) / (vf + self.initial_velocity);

        // Повертаємо стан на обчисленому часі t
        self.instant(t, dt, ds)
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Блок руху:\n  Прискорення: {}\n  Тривалість: {}\n  Початкова швидкість: {}\n  Відстань: {}\n  Початкова точка: ({}, {})\n  Кінцева точка: ({}, {})",
            self.acceleration,
            self.duration,
            self.initial_velocity,
            self.distance,
            self.p1.x(),
            self.p1.y(),
            self.p2.x(),
            self.p2.y(),
        )
    }
}
