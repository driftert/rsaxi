use super::{
    block::Block, error::PlanError, instant::Instant, segment::Segment, trapezoid::Trapezoid,
    triangle::Triangle,
};
use crate::motion::util::corner_velocity;
use geo::Point;
use std::{f64::EPSILON, fmt}; // Додаємо обидва рівні логування

/// Структура `Plan` представляє план руху, що складається з кількох сегментів (блоків).
/// Кожен блок містить інформацію про час та пройдену відстань на цьому етапі.
/// Вся траєкторія розбивається на сегменти, кожен з яких описує прискорення,
/// максимальну швидкість та інші параметри руху.
pub struct Plan {
    pub blocks: Vec<Block>,  // Масив блоків руху, що складають план
    pub total_time: f64,     // Загальний час руху
    pub total_distance: f64, // Загальна відстань руху
    pub times: Vec<f64>,     // Масив часових міток для кожного блоку
    pub distances: Vec<f64>, // Масив відстаней для кожного блоку
}

impl Plan {
    /// Створює новий план руху на основі послідовності точок (шляху),
    /// значень прискорення, максимальної швидкості та коефіцієнта повороту.
    ///
    /// # Аргументи:
    /// - `points`: Вектор координат точок `Point<f64>`, через які проходить траєкторія.
    /// - `vs`: Вектор швидкостей в кожній точці.
    /// - `vmaxs`: Вектор максимальних швидкостей для кожного сегмента (може бути порожнім).
    /// - `a`: Прискорення руху.
    /// - `vmax`: Максимально допустима швидкість.
    /// - `cf`: Коефіцієнт для корекції швидкості на поворотах.
    ///
    /// # Повертає:
    /// - `Result<Self, PlanError>`: Успішно створений план або помилка `PlanError`.
    pub fn new(
        points: Vec<Point>,
        vs: Vec<f64>,
        vmaxs: Vec<f64>,
        a: f64,
        vmax: f64,
        cf: f64,
    ) -> Result<Self, PlanError> {
        let eps = EPSILON;

        // Create segments for each consecutive pair of points
        let mut segments: Vec<Segment> = vec![];
        for i in 1..points.len() {
            segments.push(Segment::new(points[i - 1], points[i]));
        }

        // Add a dummy segment at the end for setting final velocity
        let last_point = points[points.len() - 1];
        segments.push(Segment::new(last_point, last_point));

        // Optional per-segment vmax
        let vmaxs = if vmaxs.is_empty() {
            vec![vmax; points.len()]
        } else if vmaxs.len() != points.len() {
            panic!("vmaxs array must be empty or same length as points array");
        } else {
            vmaxs
        };

        // Set max_entry_velocity based on the input velocities or angles between segments
        if vs.is_empty() {
            for i in 1..segments.len() - 1 {
                let (s1_slice, s2_slice) = segments.split_at_mut(i);
                let s1 = &s1_slice[i - 1];
                let s2 = &mut s2_slice[0];
                s2.max_entry_velocity = corner_velocity(s1, s2, vmaxs[i], a, cf);
            }
        } else if vs.len() == points.len() {
            for (i, &v) in vs.iter().enumerate() {
                segments[i].max_entry_velocity = vmaxs[i].min(v);
            }
        } else {
            panic!("vs array must be empty or same length as points array");
        }

        // Loop over segments
        let mut i: usize = 0;
        while i < segments.len() - 1 {
            let (current_segments, next_segments) = segments.split_at_mut(i + 1);
            let segment = &mut current_segments[i];
            let next_segment = &mut next_segments[0];
            let s = segment.length;
            let vi = segment.entry_velocity;
            let vmax = vmaxs[i];
            let vexit = vmax.min(next_segment.max_entry_velocity);
            let p1 = segment.p1;
            let p2 = segment.p2;

            let mut blocks: Vec<Block> = vec![];

            // Determine which profile to use for this segment
            let m = Triangle::triangular_profile(s, vi, vexit, a, p1, p2);
            if m.s1 < -eps {
                // Too fast! Update max_entry_velocity and backtrack
                segment.max_entry_velocity = (vexit * vexit + 2.0 * a * s).sqrt();
                if i > 0 {
                    i -= 1;
                }
            } else if m.s2 < 0.0 {
                // Accelerate
                let vf = (vi * vi + 2.0 * a * s).sqrt();
                let t = (vf - vi) / a;
                blocks.push(Block::new(a, t, vi, p1, p2));
                next_segment.entry_velocity = vf;
                i += 1;
            } else if m.vmax > vmax {
                // Accelerate, cruise, decelerate
                let z = Trapezoid::trapezoidal_profile(s, vi, vmax, vexit, a, p1, p2);
                blocks.push(Block::new(a, z.t1, vi, z.p1, z.p2));
                blocks.push(Block::new(0.0, z.t2, vmax, z.p2, z.p3));
                blocks.push(Block::new(-a, z.t3, vmax, z.p3, z.p4));
                next_segment.entry_velocity = vexit;
                i += 1;
            } else {
                // Accelerate, decelerate
                blocks.push(Block::new(a, m.t1, vi, m.p1, m.p2));
                blocks.push(Block::new(-a, m.t2, m.vmax, m.p2, m.p3));
                next_segment.entry_velocity = vexit;
                i += 1;
            }
            segment.blocks = blocks;
        }

        // Concatenate all blocks
        let mut all_blocks: Vec<Block> = vec![];
        for segment in &segments {
            for block in &segment.blocks {
                if block.duration > eps {
                    all_blocks.push(block.clone());
                }
            }
        }

        // Compute starting time and position for each block
        let mut ts = vec![0.0; all_blocks.len()];
        let mut ss = vec![0.0; all_blocks.len()];
        let mut t = 0.0;
        let mut s = 0.0;
        for (i, block) in all_blocks.iter().enumerate() {
            ts[i] = t;
            ss[i] = s;
            t += block.duration;
            s += block.distance;
        }

        Ok(Plan {
            blocks: all_blocks,
            total_time: t,
            total_distance: s,
            times: ts,
            distances: ss,
        })
    }

    /// Повертає стан руху в певний момент часу.
    ///
    /// # Параметри:
    /// - `t`: Час, що минув з початку руху.
    ///
    /// # Повертає:
    /// - `Option<Instant>`: Структура `Instant`, якщо час знаходиться в межах плану, інакше `None`.
    pub fn instant(&self, t: f64) -> Option<Instant> {
        // Обмежуємо час до діапазону [0, total_time]
        let clamped_t = t.clamp(0.0, self.total_time);

        // Використовуємо бінарний пошук для знаходження першого індексу, де times[i] > clamped_t
        let index = match self.times.binary_search_by(|&x| {
            if clamped_t < x {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };

        // Визначаємо індекс блоку
        let block_index = if index == 0 { 0 } else { index - 1 };

        // Отримуємо блок та обчислюємо Instant
        self.blocks.get(block_index).map(|block| {
            let t_in_block = clamped_t - self.times[block_index];
            let dt = self.times[block_index];
            let ds = self.distances[block_index];
            block.instant(t_in_block, dt, ds)
        })
    }

    /// Повертає стан руху на певній пройденій відстані.
    ///
    /// # Параметри:
    /// - `s`: Пройдена відстань з початку руху.
    ///
    /// # Повертає:
    /// - `Option<Instant>`: Структура `Instant`, якщо відстань знаходиться в межах плану, інакше `None`.
    pub fn instant_at_distance(&self, s: f64) -> Option<Instant> {
        // Обмежуємо відстань до діапазону [0, total_distance]
        let clamped_s = s.clamp(0.0, self.total_distance);

        // Використовуємо бінарний пошук для знаходження першого індексу, де distances[i] > clamped_s
        let index = match self.distances.binary_search_by(|&x| {
            if clamped_s < x {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };

        // Визначаємо індекс блоку
        let block_index = if index == 0 { 0 } else { index - 1 };

        // Отримуємо блок та обчислюємо Instant
        self.blocks.get(block_index).map(|block| {
            let s_in_block = clamped_s - self.distances[block_index];
            let dt = self.times[block_index];
            let ds = self.distances[block_index];
            block.instant_at_distance(s_in_block, dt, ds)
        })
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "План:")?;
        writeln!(f, "  Загальний час: {:.2} секунд", self.total_time)?;
        writeln!(f, "  Загальна відстань: {:.2} мм", self.total_distance)?;
        writeln!(f, "  Кількість блоків: {}", self.blocks.len())?;

        for (_, block) in self.blocks.iter().enumerate() {
            writeln!(f, "{}", block)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::Point;

    #[test]
    fn test_plan_with_three_points_investigate_velocity() {
        // Ініціалізуємо логгер
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        // Визначаємо три точки для плану
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(50.0, 50.0);
        let p3 = Point::new(100.0, 0.0);

        let points = vec![p1, p2, p3];

        // Використовуємо пусті вектори для швидкостей
        let vs = vec![];
        let vmaxs = vec![];

        // Налаштовуємо прискорення і максимальну швидкість
        let acceleration = 20.0;
        let max_velocity = 100.0;
        let corner_factor = 1.0;

        // Створюємо новий план руху
        let plan_result = Plan::new(points, vs, vmaxs, acceleration, max_velocity, corner_factor);

        // Перевіряємо, чи план створено успішно
        assert!(plan_result.is_ok());

        let plan = plan_result.unwrap();
        println!("{}", plan);
    }
}
