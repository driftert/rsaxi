use std::time::Duration;

use geo::Point;
use log::debug;

use crate::device::{Device, DeviceError, DeviceOptions, StepMode};
use crate::drawing::Drawing;
use crate::motion::block::Block;
use crate::motion::plan::Plan;

/// Константи для налаштування AxiDraw.
const STEPS_PER_MM: f64 = 80.0; // Рідна роздільна здатність: 80 кроків/мм
const MAX_RATE: u32 = 25000; // Максимальна швидкість кроків/с
const MAX_ACCEL: i32 = 50000; // Максимальне прискорення кроків/с²
const CORNER_FACTOR: f64 = 0.001; // Коефіцієнт для обробки кутів у плануванні руху
const PEN_UP_POS: i32 = 60; // Позиція піднятої ручки за замовчуванням
const PEN_UP_SPEED: i32 = 150; // Швидкість підйому ручки за замовчуванням
const PEN_UP_DELAY: i32 = 0; // Затримка після підняття ручки
const PEN_DOWN_POS: i32 = 30; // Позиція опущеної ручки за замовчуванням
const PEN_DOWN_SPEED: i32 = 150; // Швидкість опускання ручки за замовчуванням
const PEN_DOWN_DELAY: i32 = 0; // Затримка після опускання ручки
const SPEED_PENDOWN: f64 = 40.0; // Швидкість малювання за замовчуванням (мм/с)
const SPEED_PENUP: f64 = 150.0; // Швидкість переміщення без малювання (мм/с)
const ACCEL: f64 = 1.0; // Відносне прискорення за замовчуванням

/// Структура, що представляє опції налаштування для AxiDraw.
pub struct Options {
    pub speed_pendown: f64,  // Максимальна швидкість XY при опущеній ручці (мм/с).
    pub speed_penup: f64,    // Максимальна швидкість XY при піднятій ручці (мм/с).
    pub accel: f64,          // Відносна швидкість прискорення/гальмування.
    pub pen_pos_down: i32,   // Положення ручки при опущеній ручці (малювання).
    pub pen_pos_up: i32,     // Положення ручки при піднятій ручці.
    pub pen_rate_lower: i32, // Швидкість опускання механізму підйому ручки.
    pub pen_rate_raise: i32, // Швидкість підняття механізму підйому ручки.
    pub pen_delay_down: i32, // Затримка після опускання ручки (в мілісекундах).
    pub pen_delay_up: i32,   // Затримка після підняття ручки (в мілісекундах).
    pub model: AxiDrawModel, // Вибір моделі апаратного забезпечення AxiDraw.
    pub port: Option<String>, // Вказати USB-порт або AxiDraw для використання.
    pub port_config: Option<String>, // Перевизначити спосіб знаходження USB-портів.
}

impl Default for Options {
    fn default() -> Self {
        Self {
            speed_pendown: SPEED_PENDOWN, // мм/с (рекомендована швидкість малювання)
            speed_penup: SPEED_PENUP,     // мм/с (швидкість переміщення без малювання)
            accel: ACCEL,                 // Відносне прискорення (1.0 = стандартне)
            pen_pos_down: PEN_DOWN_POS,   // Положення серво при опущеній ручці
            pen_pos_up: PEN_UP_POS,       // Положення серво при піднятій ручці
            pen_rate_lower: PEN_DOWN_SPEED, // Швидкість опускання серво
            pen_rate_raise: PEN_UP_SPEED, // Швидкість підняття серво
            pen_delay_down: PEN_DOWN_DELAY, // Затримка після опускання ручки
            pen_delay_up: PEN_UP_DELAY,   // Затримка після підняття ручки
            model: AxiDrawModel::Mini,    // Модель AxiDraw за замовчуванням
            port: None,                   // Автоматичний вибір порту
            port_config: None,            // Стандартна конфігурація порту
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
            pen_up_position: options.pen_pos_up,
            pen_down_position: options.pen_pos_down,
            pen_up_speed: options.pen_rate_raise,
            pen_down_speed: options.pen_rate_lower,
            pen_up_delay: options.pen_delay_up,
            pen_down_delay: options.pen_delay_down,
            step_mode: StepMode::OneSixteenth,
            port_name: options.port.clone(),
        };

        // Ініціалізуємо пристрій
        let mut device = Device::new(device_options)?;
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
        // Ініціалізація пристрою
        self.device.enable_motors(StepMode::OneSixteenth)?;
        self.device.zero_position()?;

        // Ітерація по кожному шляху в малюнку
        for line_string in &drawing.paths.0 {
            if line_string.0.is_empty() {
                continue;
            }

            // Збираємо точки
            let points: Vec<Point<f64>> = line_string
                .0
                .iter()
                .map(|&coord| Point::new(coord.x, coord.y))
                .collect();

            // Генеруємо план руху для шляху
            let acceleration = self.options.accel * 5000.0; // мм/с²
            let max_velocity = self.options.speed_pendown; // мм/с

            let plan = Plan::new(
                points.clone(),
                None,          // Початкові швидкості
                None,          // Максимальні швидкості
                acceleration,  // Прискорення
                max_velocity,  // Максимальна швидкість
                CORNER_FACTOR, // Коефіцієнт для кутів
            )?;

            // Піднімаємо ручку перед переміщенням до початкової точки
            self.device.pen_up()?;

            // Переміщуємося до початкової точки з швидкістю speed_penup
            let start_point = points[0];
            self.move_to(start_point.x(), start_point.y(), self.options.speed_penup)?;

            // Опускаємо ручку для початку малювання
            self.device.pen_down()?;

            // Додаємо затримку після опускання ручки
            if self.options.pen_delay_down > 0 {
                std::thread::sleep(Duration::from_millis(self.options.pen_delay_down as u64));
            }

            for block in &plan.blocks {
                self.execute_block(block)?;
            }

            // Піднімаємо ручку після завершення шляху
            self.device.pen_up()?;

            // Додаємо затримку після підняття ручки
            if self.options.pen_delay_up > 0 {
                std::thread::sleep(Duration::from_millis(self.options.pen_delay_up as u64));
            }
        }

        // Завершення малювання
        self.move_to(0.0, 0.0, self.options.speed_penup)?;
        self.device.disable_motors()?;

        Ok(())
    }

    /// Виконує один блок руху, обчислюючи необхідні кроки та надсилаючи команди до пристрою.
    ///
    /// # Параметри
    /// - `block`: Блок руху, який потрібно виконати.
    ///
    /// # Повертає
    /// - `Result<(), DeviceError>`: `Ok(())` у разі успішного виконання або `DeviceError` у разі помилки.
    pub fn execute_block(&mut self, block: &Block) -> Result<(), DeviceError> {
        // Обчислюємо різницю в позиціях (дельта) у міліметрах
        let delta_x_mm = block.p2.x() - block.p1.x();
        let delta_y_mm = block.p2.y() - block.p1.y();

        // Конвертуємо дельту з міліметрів у кроки, округлюючи до найближчого цілого числа
        let delta_x_steps = (delta_x_mm * STEPS_PER_MM).round() as i32;
        let delta_y_steps = (delta_y_mm * STEPS_PER_MM).round() as i32;

        // Конвертуємо тривалість блоку з секунд у мілісекунди, округлюючи до найближчого цілого числа
        let duration_ms = (block.duration * 1000.0).round() as u64;

        // Якщо немає кроків для переміщення по обидвом осям, пропускаємо відправку команди
        if delta_x_steps == 0 && delta_y_steps == 0 {
            return Ok(());
        }

        // Створюємо об'єкт Duration з мілісекунд
        let duration = Duration::from_millis(duration_ms);

        // Надсилаємо команду руху до пристрою, передаючи тривалість та кількість кроків для кожної осі
        self.device
            .stepper_move(duration, delta_x_steps, Some(delta_y_steps))?;

        // Логуємо деталі виконаного блоку для налагодження
        debug!(
            "Виконано блок: Тривалість={}мс, Кроки=({}, {})",
            duration_ms, delta_x_steps, delta_y_steps
        );

        Ok(())
    }

    /// Переміщує ручку до вказаних координат (x, y) з вказаною швидкістю.
    ///
    /// # Параметри
    /// - `x`: Координата X в одиницях пристрою (мм).
    /// - `y`: Координата Y в одиницях пристрою (мм).
    /// - `speed`: Швидкість переміщення (мм/с).
    ///
    /// # Повертає
    /// - `Result<(), DeviceError>`: Повертає `Ok(())`, якщо переміщення успішне, або `DeviceError` у разі помилки.
    fn move_to(&mut self, x: f64, y: f64, speed: f64) -> Result<(), DeviceError> {
        // Перетворюємо координати в кроки
        let x_steps = (x * STEPS_PER_MM).round() as i32;
        let y_steps = (y * STEPS_PER_MM).round() as i32;

        // Перетворюємо швидкість в кроки/с
        let rate = (speed * STEPS_PER_MM).round() as u32;

        // Обмежуємо швидкість відповідно до можливостей пристрою
        let rate = rate.min(MAX_RATE);

        // Переміщуємося до абсолютної позиції
        self.device.home(rate, Some(x_steps), Some(y_steps))?;

        Ok(())
    }
}
