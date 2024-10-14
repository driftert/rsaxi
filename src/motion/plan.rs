use crate::motion::util::corner_velocity;

use super::{
    block::Block, error::PlanError, segment::Segment, trapezoid::Trapezoid, triangle::Triangle,
};
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
    /// Створює новий план руху на основі точок, швидкостей і прискорення.
    ///
    /// # Параметри:
    /// - `points`: Вектор точок `Point<f64>`, що визначає шлях руху.
    /// - `vs`: Опціональний вектор початкових швидкостей для кожної точки.
    /// - `vmaxs`: Опціональний вектор максимальних швидкостей для кожної точки.
    /// - `a`: Прискорення для сегментів руху.
    /// - `vmax`: Максимальна швидкість для кожного сегмента (якщо `vmaxs` не надано).
    /// - `cf`: Коефіцієнт повороту (corner factor), що враховує зміну швидкості на кутах.
    ///
    /// # Повертає:
    /// - Створює екземпляр `Plan`, що містить блоки руху з розрахованим часом і відстанню для кожного блоку.
    ///
    /// # Зауваження:
    /// - Якщо `vmaxs` не надано, для кожного сегмента використовується значення `vmax`.
    pub fn new(
        points: Vec<Point<f64>>,
        vs: Option<Vec<f64>>,
        vmaxs: Option<Vec<f64>>,
        a: f64,
        vmax: f64,
        cf: f64,
    ) -> Result<Self, PlanError> {
        const EPSILON: f64 = 1e-9;

        // Перевірка на мінімальну кількість точок
        if points.len() < 2 {
            return Err(PlanError::InsufficientPoints);
        }

        // Кількість сегментів
        let num_segments = points.len() - 1;

        // Створення сегментів для кожної пари точок
        let mut segments: Vec<Segment> = Vec::with_capacity(num_segments + 1);
        for i in 0..num_segments {
            let p1 = points[i];
            let p2 = points[i + 1];
            let mut segment = Segment::new(p1, p2);
            segments.push(segment);
        }

        // Додавання дампінгового сегмента для встановлення кінцевої швидкості
        let last_point = points[num_segments];
        let dummy_segment = Segment::new(last_point, last_point);
        segments.push(dummy_segment);

        // Ініціалізація vmaxs
        let mut vmaxs = match vmaxs {
            Some(vs) => {
                if vs.len() != points.len() {
                    return Err(PlanError::VelocityMismatch);
                }
                vs
            }
            None => vec![vmax; points.len()],
        };

        // Ініціалізація vs (початкові швидкості)
        let mut vs = match vs {
            Some(vs_values) => {
                if vs_values.len() != points.len() {
                    return Err(PlanError::VelocityMismatch);
                }
                vs_values
            }
            None => vec![0.0; points.len()],
        };

        // Присвоєння початкових та максимальних швидкостей сегментам
        for i in 0..segments.len() {
            segments[i].entry_velocity = vs[i];
            segments[i].max_entry_velocity = vmaxs[i];
        }

        // Обчислення max_entry_velocity для кута між сегментами, якщо vs не надано
        if vs.is_empty() {
            for i in 1..segments.len() - 1 {
                let s1 = &segments[i - 1];
                let s2 = &segments[i];
                segments[i].max_entry_velocity = corner_velocity(s1, s2, vmaxs[i], a, cf);
            }
        }

        // Використовуємо isize для дозволу від'ємних значень при backtracking
        let mut i: isize = 0;
        while (i as usize) < segments.len() - 1 {
            let current_index = i as usize;
            let segment = &segments[current_index];
            let next_segment = &segments[current_index + 1];

            let s = segment.length;
            let vi = segment.entry_velocity;
            let vmax_segment = segments[current_index].max_entry_velocity;
            let vexit = segments[current_index + 1]
                .max_entry_velocity
                .min(vmax_segment);
            let p1 = segment.p1;
            let p2 = segment.p2;
            let mut block_list: Vec<Block> = Vec::new();

            // Визначення профілю руху: трикутний або трапецідальний
            let m = Triangle::triangular_profile(s, vi, vexit, a, p1, p2);

            if m.s1 < -EPSILON {
                // Сегмент занадто швидкий! Оновлюємо max_entry_velocity і backtrack
                segments[current_index].max_entry_velocity = (vexit.powi(2) + 2.0 * a * s).sqrt();
                i -= 1;
                if i < 0 {
                    // Якщо i < 0, неможливо backtrack далі
                    break;
                }
                continue;
            } else if m.s2 < 0.0 {
                // Прискорення
                let vf = (vi.powi(2) + 2.0 * a * s).sqrt();
                let t = (vf - vi) / a;
                let block = Block::new(a, t, vi, p1, p2);
                block_list.push(block);
                // Оновлюємо entry_velocity наступного сегмента
                let next_segment_mut = &mut segments[current_index + 1];
                next_segment_mut.entry_velocity = vf;
                i += 1;
            } else if m.vmax > vmax_segment + EPSILON {
                // Прискорення, круїз, декреселерація
                let z = Trapezoid::trapezoidal_profile(s, vi, vmax_segment, vexit, a, p1, p2);
                let block1 = Block::new(a, z.t1, vi, z.p1, z.p2);
                let block2 = Block::new(0.0, z.t2, vmax_segment, z.p2, z.p3);
                let block3 = Block::new(-a, z.t3, vmax_segment, z.p3, z.p4);
                block_list.push(block1);
                block_list.push(block2);
                block_list.push(block3);
                // Оновлюємо entry_velocity наступного сегмента
                let next_segment_mut = &mut segments[current_index + 1];
                next_segment_mut.entry_velocity = vexit;
                i += 1;
            } else {
                // Прискорення, декреселерація
                let block1 = Block::new(a, m.t1, vi, m.p1, m.p2);
                let block2 = Block::new(-a, m.t2, m.vmax, m.p2, m.p3);
                block_list.push(block1);
                block_list.push(block2);
                // Оновлюємо entry_velocity наступного сегмента
                let next_segment_mut = &mut segments[current_index + 1];
                next_segment_mut.entry_velocity = vexit;
                i += 1;
            }

            // Присвоєння блоків сегменту
            let segment_mut = &mut segments[current_index];
            segment_mut.blocks = block_list;
        }

        // Конкатенація всіх блоків з усіх сегментів
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
}
