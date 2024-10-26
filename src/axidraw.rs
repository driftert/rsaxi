use std::f64::EPSILON;

use geo::Point;
use log::{debug, info};

use crate::device::{Device, DeviceError, DeviceOptions, StepMode};
use crate::drawing::Drawing;
use crate::motion::plan::Plan;
use crate::motion::point::PointExtension;

/// Константи для налаштування AxiDraw.
const TIMESLICE_MS: i32 = 100;
const MICROSTEPPING_MODE: u32 = 1;
const PEN_UP_POSITION: i32 = 60; // Позиція піднятої ручки за замовчуванням
const PEN_UP_SPEED: i32 = 150; // Швидкість підйому ручки за замовчуванням
const PEN_UP_DELAY: i32 = 0; // Затримка після підняття ручки
const PEN_DOWN_POSITION: i32 = 30; // Позиція опущеної ручки за замовчуванням
const PEN_DOWN_SPEED: i32 = 150; // Швидкість опускання ручки за замовчуванням
const PEN_DOWN_DELAY: i32 = 0; // Затримка після опускання ручки
const ACCELERATION: f64 = 16.0; // Прискорення за замовчуванням
const MAX_VELOCITY: f64 = 20.0; // Швидкість малювання за замовчуванням
const CORNER_FACTOR: f64 = 0.001; // Коефіцієнт для обробки кутів у плануванні руху

/// Структура, що представляє опції налаштування для AxiDraw.
pub struct Options {
    pub steps_per_unit: i32,
    pub pen_up_position: i32,        // Положення ручки при піднятій ручці.
    pub pen_up_speed: i32,           // Швидкість підняття механізму підйому ручки.
    pub pen_up_delay: i32,           // Затримка після підняття ручки (в мілісекундах).
    pub pen_down_position: i32,      // Положення ручки при опущеній ручці (малювання).
    pub pen_down_speed: i32,         // Швидкість опускання механізму підйому ручки.
    pub pen_down_delay: i32,         // Затримка після опускання ручки (в мілісекундах).
    pub acceleration: f64,           // Швидкість прискорення/гальмування..
    pub max_velocity: f64,           // Швидкість малювання за замовчуванням.
    pub corner_factor: f64,          // Коефіцієнт для обробки кутів у плануванні руху.
    pub model: AxiDrawModel,         // Вибір моделі апаратного забезпечення AxiDraw.
    pub port: Option<String>,        // Вказати USB-порт або AxiDraw для використання.
    pub port_config: Option<String>, // Перевизначити спосіб знаходження USB-портів.
}

impl Default for Options {
    fn default() -> Self {
        // Обчислюємо дільник кроків на основі режиму мікрокрокування
        let step_divider = 2i32.pow(MICROSTEPPING_MODE - 1);

        // Обчислюємо кількість кроків на одиницю (мм) з урахуванням дільника
        let steps_per_unit = 80 / step_divider;

        Self {
            steps_per_unit,
            pen_up_position: PEN_UP_POSITION,
            pen_up_speed: PEN_UP_SPEED,
            pen_up_delay: PEN_UP_DELAY,
            pen_down_position: PEN_DOWN_POSITION,
            pen_down_speed: PEN_DOWN_SPEED,
            pen_down_delay: PEN_DOWN_DELAY,
            acceleration: ACCELERATION,
            max_velocity: MAX_VELOCITY,
            corner_factor: CORNER_FACTOR,
            model: AxiDrawModel::Mini, // Модель AxiDraw за замовчуванням
            port: None,                // Автоматичний вибір порту
            port_config: None,         // Стандартна конфігурація порту
        }
    }
}

/// Представляє модель машини AxiDraw.
#[derive(Debug, Clone, Copy)]
pub enum AxiDrawModel {
    V3,   // "AxiDraw V3", ширина: 215.9 мм, висота: 279.4 мм
    V3A3, // "AxiDraw V3/A3", ширина: 279.4 мм, висота: 431.8 мм
    SEA3, // "AxiDraw SE/A3", ширина: 279.4 мм, висота: 431.8 мм
    Mini, // "AxiDraw Mini", ширина: 160 мм, висота: 101 мм
}

impl AxiDrawModel {
    /// Повертає назву моделі.
    pub fn name(&self) -> &'static str {
        match self {
            AxiDrawModel::V3 => "AxiDraw V3",
            AxiDrawModel::V3A3 => "AxiDraw V3/A3",
            AxiDrawModel::SEA3 => "AxiDraw SE/A3",
            AxiDrawModel::Mini => "AxiDraw Mini",
        }
    }

    /// Повертає ширину робочої області моделі в міліметрах.
    pub fn width(&self) -> f64 {
        match self {
            AxiDrawModel::V3 => 215.9,
            AxiDrawModel::V3A3 | AxiDrawModel::SEA3 => 279.4,
            AxiDrawModel::Mini => 160.0,
        }
    }

    /// Повертає висоту робочої області моделі в міліметрах.
    pub fn height(&self) -> f64 {
        match self {
            AxiDrawModel::V3 => 279.4,
            AxiDrawModel::V3A3 | AxiDrawModel::SEA3 => 431.8,
            AxiDrawModel::Mini => 101.0,
        }
    }
}

/// Структура для керування AxiDraw.
pub struct Axidraw {
    pub device: Device,
    pub options: Options,
}

impl Axidraw {
    /// Створює новий екземпляр `Axidraw` з налаштуваннями опцій.
    ///
    /// # Параметри
    /// - `options`: Об'єкт `Options`, що містить налаштування для AxiDraw.
    ///
    /// # Повертає
    /// - `Result<Self, DeviceError>`: Повертає `Ok(Axidraw)` при успішному створенні або `DeviceError` у разі помилки.
    pub fn new(options: Options) -> Result<Self, DeviceError> {
        let device_options = DeviceOptions {
            steps_per_unit: options.steps_per_unit,
            pen_up_position: options.pen_up_position,
            pen_up_speed: options.pen_up_speed,
            pen_up_delay: options.pen_up_delay,
            pen_down_position: options.pen_down_position,
            pen_down_speed: options.pen_down_speed,
            pen_down_delay: options.pen_down_delay,
            step_mode: StepMode::OneSixteenth,
            port_name: options.port.clone(),
            port_config: options.port_config.clone(),
        };

        // Ініціалізуємо пристрій
        let device = Device::new(device_options)?;
        Ok(Self { device, options })
    }

    /// Метод для малювання, який приймає `Drawing`.
    ///
    /// # Параметри
    /// - `drawing`: Об'єкт `Drawing`, що містить шляхи для малювання.
    ///
    /// # Повертає
    /// - `Result<(), anyhow::Error>`: Повертає `Ok(())`, якщо малювання успішне, або помилку в разі невдачі.
    pub fn draw(&mut self, drawing: &Drawing) -> Result<(), anyhow::Error> {
        // Логування інформації про малюнок
        info!("Кількість шляхів: {}", drawing.paths.0.len());
        info!("Межі малюнка: {:?}", drawing.bounds);

        // Піднімаємо перо перед початком малювання
        self.device.zero_position()?;
        self.device.pen_up()?;

        // Ініціалізація змінної для відстеження останньої точки
        let mut last_position = Point::new(0.0, 0.0);

        // Ітерація по кожному шляху в MultiLineString
        for (i, line_string) in drawing.paths.0.iter().enumerate() {
            if line_string.0.is_empty() {
                continue;
            }

            // Отримуємо першу точку поточного шляху
            let start_coord = line_string.0[0];
            let start_point = Point::new(start_coord.x, start_coord.y);

            // Переміщуємося до початкової точки з піднятим пером
            self.run_path(vec![last_position, start_point])?;

            // Опускаємо перо для початку малювання після досягнення початкової точки
            self.device.pen_down()?;

            // Малюємо шлях
            let draw_path: Vec<Point<f64>> = line_string
                .0
                .iter()
                .map(|coord| Point::new(coord.x, coord.y))
                .collect(); // Конвертуємо всі координати на Point

            // Отримуємо останню точку перед передачею в run_path
            let last_point = *draw_path.last().unwrap();

            // Виконуємо малювання по точках
            self.run_path(draw_path)?;

            // Оновлюємо останню позицію до кінцевої точки поточного шляху
            last_position = last_point;

            // Перевіряємо, чи є наступний шлях
            if let Some(next_path) = drawing.paths.0.get(i + 1) {
                // Отримуємо першу точку наступного шляху
                let next_coord = next_path.0[0];
                let next_point = Point::new(next_coord.x, next_coord.y);

                // Порівнюємо останню точку поточного шляху з першою точкою наступного шляху
                if last_position.distance(&next_point) > EPSILON {
                    // Піднімаємо перо після завершення шляху тільки якщо наступна точка далеко
                    self.device.pen_up()?;
                } else {
                    debug!("Наступна точка близько, не підіймаємо перо.");
                }
            }
        }

        // Обчислюємо кількість кроків для повернення на початкову позицію (0, 0)
        let steps_per_unit = self.options.steps_per_unit as f64;

        // Обчислюємо частоту кроків на основі максимальної швидкості
        let max_velocity = self.options.max_velocity;
        let step_frequency = (max_velocity * steps_per_unit).round() as u32;

        // Обмежуємо частоту кроків до дозволеного діапазону
        let step_frequency = step_frequency.clamp(2, 25000);

        // Повертаємося до початкової позиції (0, 0) з обчисленими кроками і частотою
        self.device.pen_up()?;

        // Виконуємо команду home
        self.device.home(step_frequency, None, None)?;

        Ok(())
    }

    /// Повертає пристрій до "домашньої" позиції (0, 0).
    ///
    /// Ця функція використовує внутрішній виклик функції `go_to`, щоб перемістити
    /// пристрій до координат (0, 0), які зазвичай відповідають початковій позиції
    /// пристрою. Це аналог команди повернення до початку робочої області.
    ///
    /// # Повертає:
    /// - `Result<(), anyhow::Error>`: Повертає `Ok`, якщо переміщення виконане успішно,
    /// або помилку у випадку невдачі.
    fn home(&mut self) -> Result<(), anyhow::Error> {
        self.goto(0.0, 0.0)
    }

    /// Виконує відносне переміщення на вказані відстані по осях X та Y.
    ///
    /// # Параметри
    /// - `dx`: Відносна відстань по осі X.
    /// - `dy`: Відносна відстань по осі Y.
    ///
    /// # Повертає
    /// - `Result<(), anyhow::Error>`: Повертає Ok або помилку у випадку невдачі.
    fn move_to(&mut self, dx: f64, dy: f64) -> Result<(), anyhow::Error> {
        // Формування шляху для відносного переміщення від (0, 0) до (dx, dy)
        let path = vec![
            Point::new(0.0, 0.0), // Початкова точка
            Point::new(dx, dy),   // Цільова точка
        ];

        // Виконуємо переміщення за сформованим шляхом
        self.run_path(path)
    }

    /// Виконує план руху, надсилаючи команди до пристрою для кожного блоку.
    ///
    /// # Параметри
    /// - `plan`: Об'єкт `Plan`, що містить блоки руху.
    ///
    /// # Повертає
    /// - `Result<(), anyhow::Error>`: Повертає Ok або помилку у випадку невдачі.
    fn run_plan(&mut self, plan: &Plan) -> Result<(), anyhow::Error> {
        let step_ms = TIMESLICE_MS;
        let step_s = step_ms as f64 / 1000.0;
        let mut t = 0.0;

        while t < plan.total_time {
            // Отримуємо стани на початку та в кінці кроку
            let i1 = plan
                .instant(t)
                .ok_or_else(|| anyhow::anyhow!("Не вдалося отримати стан на t"))?;
            let i2 = plan
                .instant(t + step_s)
                .ok_or_else(|| anyhow::anyhow!("Не вдалося отримати стан на t + step_s"))?;

            // Обчислюємо зміну позиції
            let delta = i2.position - i1.position;

            // Конвертуємо зміну в кроки двигуна
            let sx = (delta.x() * self.options.steps_per_unit as f64).round();
            let sy = (delta.y() * self.options.steps_per_unit as f64).round();

            // Виконуємо команду руху (XM - змішана геометрія для осей A та B)
            self.device
                .stepper_move_mixed(step_ms as u32, sx as i32, sy as i32)?;

            // Збільшуємо час
            t += step_s;
        }

        Ok(())
    }

    /// Виконує переміщення до абсолютних координат (x, y).
    ///
    /// # Параметри
    /// - `x`: Абсолютна координата по осі X.
    /// - `y`: Абсолютна координата по осі Y.
    ///
    /// # Повертає
    /// - `Result<(), anyhow::Error>`: Повертає Ok або помилку у випадку невдачі.
    fn goto(&mut self, x: f64, y: f64) -> Result<(), anyhow::Error> {
        // Зчитуємо поточні позиції кроків моторів
        let (motor1_steps, motor2_steps) = self.device.read_position()?;

        // Конвертуємо позиції кроків у координати x і y
        let steps_per_unit = self.options.steps_per_unit as f64;
        let a = motor1_steps as f64 / steps_per_unit;
        let b = motor2_steps as f64 / steps_per_unit;

        let current_y = (a - b) / 2.0;
        let current_x = current_y + b;

        // Формуємо шлях від поточної позиції до нової позиції (x, y)
        let path = vec![
            Point::new(current_x, current_y), // Поточна позиція
            Point::new(x, y),                 // Нова позиція
        ];

        // Виконуємо переміщення по цьому шляху
        self.run_path(path)
    }

    /// Виконує переміщення за заданим шляхом.
    ///
    /// # Параметри
    /// - `path`: Вектор точок `Point<f64>`, які визначають шлях руху.
    ///
    /// # Повертає
    /// - `Result<(), anyhow::Error>`: Повертає Ok або помилку у випадку невдачі.
    fn run_path(&mut self, path: Vec<Point<f64>>) -> Result<(), anyhow::Error> {
        // Генеруємо план руху на основі шляху
        let plan = Plan::new(
            path,
            vec![],
            vec![],
            self.options.acceleration,
            self.options.max_velocity,
            self.options.corner_factor,
        )?;

        debug!("{}", plan);

        self.run_plan(&plan)
    }
}
