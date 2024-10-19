use super::{
    block::Block, error::PlanError, instant::Instant, segment::Segment, trapezoid::Trapezoid,
    triangle::Triangle,
};
use crate::motion::util::corner_velocity;
use geo::Point;
use log::{debug, info};
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
        mut vmaxs: Vec<f64>,
        a: f64,
        vmax: f64,
        cf: f64,
    ) -> Result<Self, PlanError> {
        let mut segments = Vec::new();
        info!("Початок створення сегментів руху для траєкторії.");
        debug!("Точки маршруту: {:?}, Прискорення: {}, Максимальна швидкість: {}, Коефіцієнт повороту: {}", points, a, vmax, cf);

        // Створюємо сегменти між сусідніми точками
        for i in 1..points.len() {
            let segment = Segment::new(points[i - 1].clone(), points[i].clone());
            if segment.length > EPSILON {
                info!(
                    "Додано сегмент довжиною {:.2} мм між точками {:?}",
                    segment.length,
                    (points[i - 1], points[i])
                );
                debug!("Сегмент {} має довжину: {:.2}", i, segment.length);
                segments.push(segment);
            }
        }

        // Остання точка для завершення траєкторії
        let last_point = points[points.len() - 1].clone();
        segments.push(Segment::new(last_point.clone(), last_point.clone()));
        info!("Додано останній сегмент для завершення траєкторії.");
        debug!("Кількість сегментів: {}", segments.len());

        // Якщо vmaxs порожній, заповнюємо його значенням vmax для кожної точки
        if vmaxs.is_empty() {
            vmaxs = vec![vmax; points.len()];
            info!("Використано максимальну швидкість за замовчуванням для всіх точок.");
        } else if vmaxs.len() != points.len() {
            panic!("Масив максимальних швидкостей має бути порожнім або такого ж розміру, як масив точок.");
        }

        // Заповнення вектору вхідних швидкостей для сегментів
        if vs.is_empty() {
            segments[0].max_entry_velocity = vmaxs[0];
            for i in 1..segments.len() - 1 {
                let (left, right) = segments.split_at_mut(i);
                let s1 = &left[i - 1];
                let s2 = &mut right[0];
                s2.max_entry_velocity = corner_velocity(s1, s2, vmaxs[i], a, cf);
                info!("Розраховано вхідну швидкість для сегмента {} з урахуванням коефіцієнта повороту.", i);
                debug!(
                    "Вхідна швидкість для сегменту {} з vmax: {}, прискоренням: {}, коефіцієнтом повороту: {} дорівнює {}",
                    i, vmaxs[i], a, cf, s2.max_entry_velocity
                );
            }
        } else if vs.len() == points.len() {
            for i in 0..vs.len() {
                segments[i].max_entry_velocity = vs[i].min(vmaxs[i]);
                debug!(
                    "Встановлено max_entry_velocity для сегмента {} = {}",
                    i, segments[i].max_entry_velocity
                );
            }
        } else {
            panic!("Масив швидкостей має бути або порожнім, або того ж розміру, що і масив точок.");
        }

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
            let m = Triangle::triangular_profile(s, vi, vexit, a, p1, p2);

            if m.s1 < -EPSILON {
                info!("Швидкість для сегменту {} перевищує допустиму. Проводиться зменшення швидкості.", current_index);
                left[current_index].max_entry_velocity = (vexit.powi(2) + 2.0 * a * s).sqrt();
                i -= 1;
                debug!(
                    "Повернення до сегменту {}: зменшено max_entry_velocity до {}",
                    current_index, left[current_index].max_entry_velocity
                );
            } else if m.s2 <= 0.0 {
                info!(
                    "Сегмент {}: профіль прискорення без гальмування.",
                    current_index
                );
                let vf = (vi.powi(2) + 2.0 * a * s).sqrt();
                let t = (vf - vi) / a;
                let block = Block::new(a, t, vi, p1, p2);
                block_list.push(block);
                next_segment.entry_velocity = vf;
                i += 1;
                debug!(
                    "Сегмент {}: vf = {}, вхідна швидкість наступного сегменту = {}",
                    current_index, vf, next_segment.entry_velocity
                );
            } else if m.vmax > vmax_segment + EPSILON {
                info!("Сегмент {}: профіль трапеціїдального типу.", current_index);
                let z = Trapezoid::trapezoidal_profile(s, vi, vmax_segment, vexit, a, p1, p2);
                let block1 = Block::new(a, z.t1, vi, z.p1, z.p2);
                let block2 = Block::new(0.0, z.t2, vmax_segment, z.p2, z.p3);
                let block3 = Block::new(-a, z.t3, vmax_segment, z.p3, z.p4);
                block_list.push(block1);
                block_list.push(block2);
                block_list.push(block3);
                next_segment.entry_velocity = vexit;
                i += 1;
                debug!(
                    "Сегмент {}: трапецідальний профіль, вхідна швидкість наступного сегменту = {}",
                    current_index, next_segment.entry_velocity
                );
            } else {
                info!(
                    "Сегмент {}: комбінований профіль прискорення та гальмування.",
                    current_index
                );
                let block1 = Block::new(a, m.t1, vi, m.p1, m.p2);
                let block2 = Block::new(-a, m.t2, m.vmax, m.p2, m.p3);
                block_list.push(block1);
                block_list.push(block2);
                next_segment.entry_velocity = vexit;
                i += 1;
                debug!(
                    "Сегмент {}: комбінований профіль, вхідна швидкість наступного сегменту = {}",
                    current_index, next_segment.entry_velocity
                );
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

        info!(
            "План руху успішно створений. Загальний час: {:.2} секунд, загальна відстань: {:.2} мм",
            t, s
        );
        debug!(
            "Загальна кількість блоків: {}, загальний час: {:.2}, загальна відстань: {:.2}",
            all_blocks.len(),
            t,
            s
        );

        Ok(Self {
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
