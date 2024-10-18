use super::{
    block::Block, error::PlanError, instant::Instant, point::PointExtension, segment::Segment,
    trapezoid::Trapezoid, triangle::Triangle,
};
use crate::motion::util::corner_velocity;
use geo::Point;

/// Структура `Plan` представляє план руху, що складається з блоків.
/// Вона включає час, пройдену відстань, блоки та масиви часу і відстані для кожного блоку.
pub struct Plan {
    pub blocks: Vec<Block>,  // Блоки руху
    pub total_time: f64,     // Загальний час руху
    pub total_distance: f64, // Загальна пройдена відстань
    pub times: Vec<f64>,     // Час кожного блоку
    pub distances: Vec<f64>, // Відстань для кожного блоку
}

impl Plan {
    /// Створює новий план руху на основі точок, прискорення, максимальної швидкості та коефіцієнта повороту.
    ///
    /// # Параметри:
    /// - `points`: Вектор точок `Point<f64>`, що визначає шлях руху.
    /// - `acceleration`: Прискорення для сегментів руху.
    /// - `max_velocity`: Максимальна швидкість для кожного сегмента.
    /// - `corner_factor`: Коефіцієнт повороту (corner factor), що враховує зміну швидкості на кутах.
    ///
    /// # Повертає:
    /// - `Result<Self, PlanError>`: Створений `Plan` або помилка.
    pub fn new(
        points: Vec<Point<f64>>,
        acceleration: f64,
        max_velocity: f64,
        corner_factor: f64,
    ) -> Result<Self, PlanError> {
        const EPSILON: f64 = 1e-9;

        // Крок 1: Видалення дублікатів точок
        let deduped_points = Self::dedup_points(points, EPSILON);

        // Крок 2: Обробка випадку з одною точкою
        if deduped_points.len() == 1 {
            let single_point = deduped_points[0];
            let block = Block::new(0.0, 0.0, 0.0, single_point, single_point);
            return Ok(Self {
                blocks: vec![block],
                total_time: 0.0,
                total_distance: 0.0,
                times: vec![0.0],
                distances: vec![0.0],
            });
        }

        // Крок 3: Створення сегментів
        let mut segments: Vec<Segment> = deduped_points
            .windows(2)
            .map(|pair| Segment::new(pair[0], pair[1]))
            .collect();

        // Крок 4: Обчислення максимальних вхідних швидкостей для кутів
        for i in 1..segments.len() {
            let (prev_segments, current_segments) = segments.split_at_mut(i);
            let prev_segment = &prev_segments[i - 1];
            let current_segment = &mut current_segments[0];

            // Обчислюємо максимальну вхідну швидкість тільки коли це потрібно
            current_segment.max_entry_velocity = corner_velocity(
                prev_segment,
                current_segment,
                max_velocity,
                acceleration,
                corner_factor,
            );
        }

        // Крок 5: Додавання дампінгового сегмента для кінцевої точки
        let last_point = deduped_points.last().unwrap().clone();
        segments.push(Segment::new(last_point, last_point));

        // Використовуємо isize для дозволу від'ємних значень при backtracking
        let mut i: isize = 0;
        while (i as usize) < segments.len() - 1 {
            let current_index = i as usize;

            // Розділяємо вектор на дві частини: до поточного індексу + 1 і після
            let (left, right) = segments.split_at_mut(current_index + 1);

            let segment = &left[current_index];
            let next_segment = &mut right[0];

            let s = segment.length;
            let vi = segment.entry_velocity;
            let vmax_segment = segment.max_entry_velocity;
            let vexit = next_segment.max_entry_velocity.min(vmax_segment);
            let p1 = segment.p1;
            let p2 = segment.p2;
            let mut block_list: Vec<Block> = Vec::new();

            // Визначення профілю руху: трикутний або трапецідальний
            let m = Triangle::triangular_profile(s, vi, vexit, acceleration, p1, p2);

            if m.s1 < -EPSILON {
                // Сегмент занадто швидкий! Оновлюємо max_entry_velocity і backtrack
                left[current_index].max_entry_velocity =
                    (vexit.powi(2) + 2.0 * acceleration * s).sqrt();

                if i > 0 {
                    i -= 1;
                } else {
                    // Неможливо відкотитися далі; вийти з циклу
                    break;
                }
                continue;
            } else if m.s2 <= 0.0 {
                // Прискорення без декреселерації
                let vf = (vi.powi(2) + 2.0 * acceleration * s).sqrt();
                let t = (vf - vi) / acceleration;
                let block = Block::new(acceleration, t, vi, p1, p2);
                block_list.push(block);
                // Оновлюємо entry_velocity наступного сегмента
                next_segment.entry_velocity = vf;
                i += 1;
            } else if m.vmax > vmax_segment + EPSILON {
                // Трапецідальний профіль: прискорення, круїз, декреселерація
                let z = Trapezoid::trapezoidal_profile(
                    s,
                    vi,
                    vmax_segment,
                    vexit,
                    acceleration,
                    p1,
                    p2,
                );
                let block1 = Block::new(acceleration, z.t1, vi, z.p1, z.p2);
                let block2 = Block::new(0.0, z.t2, vmax_segment, z.p2, z.p3);
                let block3 = Block::new(-acceleration, z.t3, vmax_segment, z.p3, z.p4);
                block_list.push(block1);
                block_list.push(block2);
                block_list.push(block3);
                // Оновлюємо entry_velocity наступного сегмента
                next_segment.entry_velocity = vexit;
                i += 1;
            } else {
                // Комбіноване прискорення та декреселерація
                let block1 = Block::new(acceleration, m.t1, vi, m.p1, m.p2);
                let block2 = Block::new(-acceleration, m.t2, m.vmax, m.p2, m.p3);
                block_list.push(block1);
                block_list.push(block2);
                // Оновлюємо entry_velocity наступного сегмента
                next_segment.entry_velocity = vexit;
                i += 1;
            }

            // Присвоєння блоків сегменту
            left[current_index].blocks = block_list;
        }

        // Конкатенація всіх блоків з сегментів
        let mut all_blocks: Vec<Block> = Vec::new();
        for segment in &segments {
            for block in &segment.blocks {
                if block.duration > EPSILON {
                    all_blocks.push(block.clone());
                }
            }
        }

        // Обчислення початкового часу та відстані для кожного блоку
        let mut ts: Vec<f64> = Vec::with_capacity(all_blocks.len());
        let mut ss: Vec<f64> = Vec::with_capacity(all_blocks.len());
        let mut t = 0.0;
        let mut s = 0.0;

        for block in &all_blocks {
            ts.push(t);
            ss.push(s);
            t += block.duration;
            s += block.distance;
        }

        Ok(Self {
            blocks: all_blocks,
            total_time: t,
            total_distance: s,
            times: ts,
            distances: ss,
        })
    }

    /// Видаляє послідовні дублікатні точки, які знаходяться в межах заданого порогу epsilon.
    ///
    /// # Параметри:
    /// - `points`: Вектор точок для видалення дублікатів.
    /// - `epsilon`: Мінімальна відстань між точками, щоб вважати їх різними.
    ///
    /// # Повертає:
    /// - `Vec<Point<f64>>`: Вектор точок без послідовних дублікатів.
    fn dedup_points(points: Vec<Point<f64>>, epsilon: f64) -> Vec<Point<f64>> {
        if points.is_empty() {
            return Vec::new();
        }

        let mut deduped = Vec::new();
        deduped.push(points[0].clone());

        for point in points.iter().skip(1) {
            let last = deduped.last().unwrap();
            let distance = last.distance(point);
            if distance > epsilon {
                deduped.push(point.clone());
            }
        }

        deduped
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
