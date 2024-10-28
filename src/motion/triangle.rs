use super::point::PointExtension;
use geo::Point;

/// Структура представляє трикутний профіль руху, який складається з двох етапів:
/// прискорення і уповільнення.
pub struct Triangle {
    pub s1: f64,        // Відстань, пройдена під час прискорення.
    pub s2: f64,        // Відстань, пройдена під час уповільнення.
    pub t1: f64,        // Час прискорення.
    pub t2: f64,        // Час уповільнення.
    pub vmax: f64,      // Максимальна досягнута швидкість.
    pub p1: Point<f64>, // Початкова точка руху.
    pub p2: Point<f64>, // Точка після фази прискорення.
    pub p3: Point<f64>, // Кінцева точка руху.
}

impl Triangle {
    /// Обчислює трикутний профіль руху: прискорення та уповільнення.
    ///
    /// # Параметри
    /// - `s`: Загальна відстань руху.
    /// - `vi`: Початкова швидкість.
    /// - `vf`: Кінцева швидкість.
    /// - `a`: Прискорення (однакове для прискорення і уповільнення).
    /// - `p1`: Початкова точка руху.
    /// - `p3`: Кінцева точка руху.
    ///
    /// # Повертає
    /// Повертає структуру `Triangle`, яка містить параметри трикутного профілю: відстані, часи
    /// та проміжні точки руху.
    pub fn triangular_profile(
        s: f64,
        vi: f64,
        vf: f64,
        a: f64,
        p1: Point<f64>,
        p3: Point<f64>,
    ) -> Self {
        let s1 = (2.0 * a * s + vf * vf - vi * vi) / (4.0 * a);
        let s2 = s - s1;
        let vmax = (vi * vi + 2.0 * a * s1).sqrt();
        let t1 = (vmax - vi) / a;
        let t2 = (vf - vmax) / -a;
        let p2 = p1.lerps(&p3, s1 / s);

        Self {
            s1,
            s2,
            t1,
            t2,
            vmax,
            p1,
            p2,
            p3,
        }
    }
}