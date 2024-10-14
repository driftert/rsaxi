use log::{debug, error, info};
use serialport::{available_ports, DataBits, Parity, SerialPort, SerialPortType, StopBits};
use std::time::Duration;
use thiserror::Error;

// Константи для дефолтних значень позицій та швидкостей ручки
const PEN_UP_POS: i32 = 60; // Максимальна позиція підйому ручки за замовчуванням
const PEN_UP_SPEED: i32 = 150; // Швидкість підйому ручки за замовчуванням
const PEN_UP_DELAY: i32 = 0;

const PEN_DOWN_POS: i32 = 40; // Мінімальна позиція опускання ручки за замовчуванням
const PEN_DOWN_SPEED: i32 = 150; // Швидкість опускання ручки за замовчуванням
const PEN_DOWN_DELAY: i32 = 0;

/// Тип для обробки помилок, які можуть виникнути під час роботи з пристроєм
#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Помилка підключення: {0}")]
    ConnectionError(String),

    #[error("Помилка команди: {0}")]
    CommandError(String),

    #[error("Невірне значення для параметру: {parameter}, значення: {value}")]
    InvalidValue { parameter: String, value: String },

    #[error("Некоректна відповідь: {0}")]
    InvalidResponse(String),
}

/// Режими кроків для моторів (глобальний режим)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepMode {
    Disable = 0,      // Вимкнути мотори
    OneSixteenth = 1, // 1/16 крок
    OneEighth = 2,    // 1/8 крок
    OneQuarter = 3,   // 1/4 крок
    OneHalf = 4,      // 1/2 крок
    FullStep = 5,     // Повний крок
}

/// Стан мотора (чи виконується команда, чи рухається мотор, чи FIFO порожня)
#[derive(Debug, Clone, Copy)]
pub struct MotorStatus {
    pub executing_command: bool, // Чи виконується команда
    pub moving: bool,            // Чи рухається мотор
    pub fifo_empty: bool,        // Чи FIFO порожня
}

/// Структура Device для керування підключенням до пристрою через серійний порт
/// Ця структура дозволяє керувати різними аспектами пристрою, включаючи серійний зв'язок
/// та глобальні налаштування для управління положенням і швидкістю руху ручки.
pub struct Device {
    pub port: Box<dyn SerialPort>, // Серійний порт
    pub connected: bool,           // Прапорець для відстеження стану підключення

    // Глобальні конфігураційні параметри для управління ручкою
    pub pen_up_position: i32,   // Максимальна позиція підйому ручки
    pub pen_up_speed: i32,      // Швидкість підйому ручки
    pub pen_up_delay: i32,      // Затримка при підйомі
    pub pen_down_position: i32, // Мінімальна позиція опускання ручки
    pub pen_down_speed: i32,    // Швидкість опускання ручки
    pub pen_down_delay: i32,    // Затримка при опусканні

    // Стан ручки
    pub is_lowered: bool, // Стан ручки: true — опущена, false — піднята

    // Стан моторів
    step_mode: StepMode, // Глобальний режим кроку для обох моторів
    motor1_enabled: bool,
    motor2_enabled: bool,
}

impl Device {
    /// Конструктор для створення екземпляра `Device`, що автоматично підключається до знайденого порту
    ///
    /// Викликає методи для пошуку доступного порту та підключення до нього.
    /// Також виконує конфігурацію глобальних налаштувань для керування ручкою.
    ///
    /// # Повертає:
    /// - `Result<Self, DeviceError>`: Повертає екземпляр структури Device або помилку в разі невдачі.
    pub fn new() -> Result<Self, DeviceError> {
        let port_name = Device::find_port()?; // Пошук порту для підключення
        let port = Device::connect(&port_name)?; // Підключення до знайденого порту

        // Створення нового екземпляра `Device` з налаштуванням конфігурації для управління ручкою
        let mut device = Self {
            port,
            connected: true,
            pen_up_position: PEN_UP_POS,
            pen_up_speed: PEN_UP_SPEED,
            pen_up_delay: PEN_UP_DELAY,
            pen_down_position: PEN_DOWN_POS,
            pen_down_speed: PEN_DOWN_SPEED,
            pen_down_delay: PEN_DOWN_DELAY,
            is_lowered: false,
            step_mode: StepMode::OneSixteenth,
            motor1_enabled: false,
            motor2_enabled: false,
        };

        // Виклик методу для налаштування пристрою з використанням глобальних параметрів
        device.configure()?;
        device.is_lowered = device.query_pen_state()?;

        // Зчитуємо поточний стан моторів
        let (motor1_enabled, motor2_enabled, step_mode) = device.query_enable_motors()?;

        device.motor1_enabled = motor1_enabled;
        device.motor2_enabled = motor2_enabled;
        device.step_mode = step_mode;

        Ok(device)
    }

    /// Виконання конфігурації для налаштування глобальних параметрів ручки
    ///
    /// Цей метод налаштовує мінімальні та максимальні положення ручки, а також швидкість її підйому та опускання.
    /// Викликається в конструкторі для автоматичної ініціалізації після підключення.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok у випадку успіху або помилку при невдачі.
    fn configure(&mut self) -> Result<(), DeviceError> {
        let servo_min = 7500;
        let servo_max = 28000;

        // Розрахунок позиції підйому ручки (пропорційно до діапазону значень сервоприводу)
        let pen_up_position = (self.pen_up_position as f32 / 100.0)
            * (servo_max - servo_min) as f32
            + servo_min as f32;
        let pen_up_position = pen_up_position as i32;

        // Розрахунок позиції опускання ручки (пропорційно до діапазону значень сервоприводу)
        let pen_down_position = (self.pen_down_position as f32 / 100.0)
            * (servo_max - servo_min) as f32
            + servo_min as f32;
        let pen_down_position = pen_down_position as i32;

        // Відправка команд для конфігурації позицій і швидкостей
        self.command(&format!("SC,4,{}", pen_up_position))?;
        self.command(&format!("SC,5,{}", pen_down_position))?;
        self.command(&format!("SC,11,{}", self.pen_up_speed * 5))?;
        self.command(&format!("SC,12,{}", self.pen_down_speed * 5))?;

        Ok(())
    }

    /// Пошук доступного серійного порту
    ///
    /// Цей метод шукає серійний порт, підключений до пристрою EiBotBoard, використовуючи інформацію про USB.
    /// Якщо пристрій не знайдено, повертається помилка.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Назву порту або помилку, якщо пристрій не знайдено.
    fn find_port() -> Result<String, DeviceError> {
        info!("Пошук серійного порту...");
        let ports = available_ports().map_err(|e| {
            DeviceError::ConnectionError(format!("Помилка при отриманні списку портів: {:?}", e))
        })?;

        // Пошук порту, що відповідає певному пристрою
        ports
            .into_iter()
            .find_map(|port| {
                if let SerialPortType::UsbPort(info) = &port.port_type {
                    if let Some(product) = &info.product {
                        if product.starts_with("EiBotBoard") {
                            info!("Знайдено пристрій продукту: {}", product);
                            return Some(port.port_name.clone());
                        }
                    }
                }
                None
            })
            .ok_or_else(|| {
                DeviceError::ConnectionError("Не знайдено відповідного порту".to_string())
            })
    }

    /// Підключення до пристрою через серійний порт
    ///
    /// Метод встановлює з'єднання з пристроєм, використовуючи назву порту, і налаштовує серійний порт для зв'язку.
    /// Також надсилається команда для ініціалізації DTR (Data Terminal Ready).
    ///
    /// # Параметри:
    /// - `port_name`: Назва порту, до якого потрібно підключитися.
    ///
    /// # Повертає:
    /// - `Result<Box<dyn SerialPort>, DeviceError>`: Повертає відкритий серійний порт або помилку у випадку невдачі.
    fn connect(port_name: &str) -> Result<Box<dyn SerialPort>, DeviceError> {
        info!("Підключення до пристрою: {} ...", port_name);

        // Створення і конфігурація серійного порту
        let mut port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(100))
            .parity(Parity::None)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .open()
            .map_err(|e| {
                DeviceError::ConnectionError(format!("Не вдалося відкрити порт: {}", e))
            })?;

        // Ініціалізація порту
        port.write_data_terminal_ready(true).map_err(|e| {
            DeviceError::CommandError(format!("Помилка ініціалізації порту: {}", e))
        })?;

        info!("Підключено до пристрою: {}", port_name);
        Ok(port)
    }

    /// Відправлення команди до пристрою і зчитування повної відповіді
    ///
    /// Цей метод відправляє команду через серійний порт і чекає на відповідь від пристрою.
    /// Якщо команда завершується успішно, пристрій надсилає підтвердження у вигляді "OK".
    ///
    /// # Параметри:
    /// - `cmd`: Команда, яку потрібно надіслати.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Відповідь від пристрою або помилку, якщо команда не виконується.
    pub fn command(&mut self, cmd: &str) -> Result<String, DeviceError> {
        // Локальна функція для зчитування буфера відповіді
        fn read_full_response(port: &mut dyn SerialPort) -> Result<String, DeviceError> {
            let mut response = Vec::new();
            loop {
                let mut buffer = [0; 256];
                match port.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }
                        response.extend_from_slice(&buffer[..bytes_read]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        break;
                    }
                    Err(e) => {
                        return Err(DeviceError::CommandError(format!(
                            "Помилка читання відповіді: {}",
                            e
                        )));
                    }
                }
            }
            let response_str = String::from_utf8_lossy(&response).to_string();
            debug!("Отримано відповідь: {}", response_str);
            Ok(response_str)
        }

        // Перевірка, чи команда порожня
        if cmd.is_empty() {
            return Err(DeviceError::CommandError(
                "Команда не може бути порожньою".to_string(),
            ));
        }

        // Перевірка, чи команда містить недопустимі символи
        if !cmd.is_ascii() || cmd.contains(|c| c == '\n' || c == '\r') {
            return Err(DeviceError::CommandError(
                "Команда містить недозволені символи".to_string(),
            ));
        }

        // Формування повної команди з додаванням символа `\r`
        let full_cmd = format!("{}\r", cmd.trim());
        if full_cmd.len() > 256 {
            return Err(DeviceError::CommandError(
                "Команда перевищує максимальну довжину 256 байт".to_string(),
            ));
        }

        // Деякі команди не очікують "OK" у відповіді
        let no_ok_commands = ["V", "I", "A", "MR", "PI", "QM"];

        // Перевірка стану підключення
        if self.connected != true {
            return Err(DeviceError::ConnectionError(
                "Пристрій не підключений".to_string(),
            ));
        }

        info!("Відправлення команди: {}", full_cmd.trim_end());
        self.port.write_all(full_cmd.as_bytes()).map_err(|e| {
            DeviceError::CommandError(format!("Помилка відправлення команди: {}", e))
        })?;

        let response = read_full_response(self.port.as_mut())?;

        // Якщо команда не входить до списку `no_ok_commands`, перевіряємо наявність "OK" у відповіді
        if !no_ok_commands
            .iter()
            .any(|&c| cmd.to_uppercase().starts_with(c))
        {
            if response.ends_with("OK\r\n") {
                let trimmed_response = response.trim_end_matches("OK\r\n").to_string();
                Ok(trimmed_response)
            } else {
                Err(DeviceError::CommandError(
                    "Відповідь не містить очікуваного OK".to_string(),
                ))
            }
        } else {
            Ok(response)
        }
    }

    /// Відправлення команди ReBoot (RB) для перезавантаження пристрою
    ///
    /// Цей метод надсилає команду перезавантаження пристрою і очікує підтвердження успішного виконання.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok або помилку, якщо перезавантаження не вдалося.
    pub fn reboot(&mut self) -> Result<(), DeviceError> {
        self.command("RB").map(|_| {
            info!("Пристрій перезавантажується...");
        })
    }

    ///
    /// Цей метод надсилає команду для скидання пристрою до його початкового стану. Після виконання команди пристрій
    /// повинен повернутися до базової конфігурації та бути готовим до прийняття нових команд.
    ///
    /// # Повертає:
    /// - `Result<(, DeviceError>`: Повертає Ok або помилку в разі невдачі.
    pub fn reset(&mut self) -> Result<(), DeviceError> {
        self.command("R").map(|_| {
            info!("Пристрій скинуто.");
        })
    }

    /// Встановлення псевдоніму на пристрої EBB
    ///
    /// Цей метод дозволяє встановити псевдонім (ім'я) для пристрою. Псевдонім може бути використаний для ідентифікації
    /// пристрою при підключенні або відображенні в системі.
    ///
    /// # Параметри:
    /// - `nickname`: Псевдонім, який потрібно встановити. Має бути не більше 16 символів і складатися з ASCII символів.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok або помилку в разі, якщо псевдонім недійсний або команда не виконується.
    pub fn nickname(&mut self, nickname: &str) -> Result<(), DeviceError> {
        if nickname.len() > 16 {
            return Err(DeviceError::CommandError(
                "Псевдонім повинен містити не більше 16 символів".to_string(),
            ));
        }

        if !nickname.is_ascii() {
            return Err(DeviceError::CommandError(
                "Псевдонім повинен містити лише ASCII символи".to_string(),
            ));
        }

        let cmd = format!("ST,{}", nickname);
        self.command(&cmd).map(|_| {
            info!("Псевдонім встановлено.");
        })
    }

    /// Запит версії прошивки EBB
    ///
    /// Цей метод надсилає команду для запиту версії прошивки пристрою. Відповідь містить номер версії, який
    /// можна використовувати для ідентифікації поточної прошивки на пристрої.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає рядок із версією прошивки або помилку в разі невдачі.
    pub fn version(&mut self) -> Result<String, DeviceError> {
        let response = self.command("V")?;
        info!("Версія прошивки: {}", response.trim());
        Ok(response)
    }

    /// Читання значення піну на певному порту
    ///
    /// Цей метод дозволяє прочитати стан одного з пінів на порту (наприклад, порти A, B, C, D, E). Команда повертає
    /// значення, яке вказує на те, чи встановлений пін у високий (1) або низький (0) стан.
    ///
    /// # Параметри:
    /// - `port`: Символ, який представляє порт (A, B, C, D, E).
    /// - `pin`: Номер піна на порті (значення від 0 до 7).
    ///
    /// # Повертає:
    /// - `Result<bool, DeviceError>`: Повертає `true`, якщо пін встановлений у високий стан, і `false`, якщо в низький, або помилку в разі невдачі.
    ///
    /// # Обмеження:
    /// - Якщо передано недійсний порт або пін (порт не з A-E або пін поза діапазоном 0-7), метод повертає помилку.
    pub fn read_pin(&mut self, port: char, pin: u8) -> Result<bool, DeviceError> {
        if !['A', 'B', 'C', 'D', 'E'].contains(&port) {
            return Err(DeviceError::CommandError(format!(
                "Недійсний порт: {}. Доступні порти: A, B, C, D, E",
                port
            )));
        }

        if pin > 7 {
            return Err(DeviceError::CommandError(format!(
                "Недійсний пін: {}. Допустимі значення: 0-7",
                pin
            )));
        }

        let cmd = format!("PI,{},{}", port, pin);
        let response = self.command(&cmd)?;
        debug!("Відповідь на PI: {}", response.trim());

        let value = response.trim().split(',').last().unwrap_or("0").trim();
        match value {
            "1" => Ok(true),
            "0" => Ok(false),
            _ => Err(DeviceError::InvalidResponse(format!(
                "Некоректна відповідь для піну: {}",
                response
            ))),
        }
    }

    /// Метод для налаштування напрямку піну (PD)
    ///
    /// Цей метод дозволяє встановити напрямок для певного піну на вказаному порту.
    /// Напрямок може бути або вхід (1), або вихід (0).
    ///
    /// # Параметри:
    /// - `port`: Символ, що представляє порт (A, B, C, D, E).
    /// - `pin`: Номер піну на порті (значення від 0 до 7).
    /// - `direction`: Напрямок для піну: 0 для виходу або 1 для входу.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok у випадку успіху або помилку в разі невдачі.
    ///
    /// # Обмеження:
    /// - Порт має бути A-E, а пін повинен знаходитися в діапазоні від 0 до 7.
    /// - Напрямок має бути або 0 (вихід), або 1 (вхід).
    pub fn pin_direction(&mut self, port: char, pin: u8, direction: u8) -> Result<(), DeviceError> {
        // Перевіряємо, чи передано дійсний порт
        if !['A', 'B', 'C', 'D', 'E'].contains(&port) {
            return Err(DeviceError::CommandError(format!(
                "Недійсний порт: {}. Доступні порти: A, B, C, D, E",
                port
            )));
        }

        // Перевіряємо, чи номер піну є в межах діапазону (0-7)
        if pin > 7 {
            return Err(DeviceError::CommandError(format!(
                "Недійсний пін: {}. Допустимі значення: 0-7",
                pin
            )));
        }

        // Перевіряємо, чи напрямок є дійсним (0 - вихід, 1 - вхід)
        if direction > 1 {
            return Err(DeviceError::CommandError(format!(
                "Недійсний напрямок: {}. Доступні значення: 0 (вихід), 1 (вхід)",
                direction
            )));
        }

        // Формуємо команду "PD,Port,Pin,Direction"
        let cmd = format!("PD,{}, {},{}", port, pin, direction);

        self.command(&cmd)?;
        Ok(())
    }

    /// Метод для опускання ручки
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok або помилку в разі невдачі.
    pub fn pen_down(&mut self) -> Result<(), DeviceError> {
        let delta = (self.pen_up_position - self.pen_down_position).abs();
        let duration = (1000 * delta) / self.pen_down_speed;
        let delay = duration + self.pen_down_delay;
        self.pen_state(0, Some(Duration::from_millis(delay as u64)), None)
    }

    /// Метод для підйому ручки
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok або помилку в разі невдачі.
    pub fn pen_up(&mut self) -> Result<(), DeviceError> {
        let delta = (self.pen_up_position - self.pen_down_position).abs();
        let duration = (1000 * delta) / self.pen_up_speed;
        let delay = duration + self.pen_up_delay;
        self.pen_state(1, Some(Duration::from_millis(delay as u64)), None)
    }

    /// Метод для перемикання стану ручки (TP)
    ///
    /// Цей метод перемикає стан ручки з піднятої в опущену і навпаки. Також можна задати опціональну затримку.
    ///
    /// # Параметри:
    /// - `duration`: (опціонально) Затримка виконання операції.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку в разі невдачі.
    pub fn pen_toggle(&mut self, duration: Option<Duration>) -> Result<String, DeviceError> {
        let mut cmd = "TP".to_string();

        if let Some(dur) = duration {
            let duration_in_ms = dur.as_millis();
            if duration_in_ms < 1 || duration_in_ms > 65535 {
                return Err(DeviceError::InvalidValue {
                    parameter: "duration".to_string(),
                    value: duration_in_ms.to_string(),
                });
            }
            cmd.push_str(&format!(",{}", duration_in_ms));
        }

        let response = self.command(&cmd)?;

        // Перемикаємо поточний стан ручки
        self.is_lowered = !self.is_lowered;

        Ok(response)
    }

    /// Метод для встановлення стану ручки (SP)
    ///
    /// Цей метод використовується для встановлення ручки в опущений (0) або піднятий (1) стан.
    /// Також можна задати затримку виконання та пін порту B, який контролюється.
    ///
    /// # Параметри:
    /// - `value`: Значення 0 для опускання або 1 для підйому ручки.
    /// - `duration`: (опціонально) Затримка перед виконанням операції.
    /// - `portb_pin`: (опціонально) Номер піна на порту B (0-7), який можна використовувати для керування.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає Ok або помилку в разі невдачі.
    pub fn pen_state(
        &mut self,
        value: u8,                  // 0 — опустити, 1 — підняти ручку
        duration: Option<Duration>, // (необов'язково) затримка
        portb_pin: Option<u8>,      // (необов'язково) номер піна на порту B (0-7)
    ) -> Result<(), DeviceError> {
        // Перевіряємо, чи значення value є дійсним (0 або 1)
        if value != 0 && value != 1 {
            error!("Недійсне значення для value: {}", value);
            return Err(DeviceError::InvalidValue {
                parameter: "value".to_string(),
                value: value.to_string(),
            });
        }

        // Перевіряємо, чи значення portb_pin є дійсним (0-7), якщо воно задано
        if let Some(pin) = portb_pin {
            if pin > 7 {
                error!("Недійсне значення для portb_pin: {}", pin);
                return Err(DeviceError::InvalidValue {
                    parameter: "portb_pin".to_string(),
                    value: pin.to_string(),
                });
            }
        }

        // Формуємо команду для пристрою "SP,Value[,Duration[,PortB_Pin]]"
        let mut cmd = format!("SP,{}", value);

        // Додаємо Duration, якщо він заданий
        if let Some(dur) = duration {
            let duration_in_ms = dur.as_millis();
            // Перевіряємо, чи Duration не перевищує 65535 мс
            if duration_in_ms < 1 || duration_in_ms > u16::MAX.into() {
                error!(
                    "Тривалість повинна бути в діапазоні 1-65535 мс: {} мс",
                    duration_in_ms
                );
                return Err(DeviceError::InvalidValue {
                    parameter: "duration".to_string(),
                    value: duration_in_ms.to_string(),
                });
            }
            cmd.push_str(&format!(",{}", duration_in_ms));
        } else {
            // Якщо тривалість не вказана, використовується значення 0 мс за замовчуванням
            cmd.push_str(",0");
        }

        // Додаємо PortB_Pin, якщо він заданий
        if let Some(pin) = portb_pin {
            cmd.push_str(&format!(",{}", pin));
        }

        // Надсилаємо команду
        self.command(&cmd)?;

        // Оновлюємо стан ручки після успішної команди
        self.is_lowered = value == 0; // Ручка опущена, якщо value = 0, і піднята, якщо value = 1

        Ok(())
    }

    /// Метод для запиту поточного стану ручки (QP)
    ///
    /// Цей метод запитує поточний стан ручки. Він повертає, чи ручка знаходиться у піднятому або опущеному стані.
    ///
    /// # Повертає:
    /// - `Result<bool, DeviceError>`: Повертає `true`, якщо ручка опущена, і `false`, якщо піднята, або помилку в разі невдачі.
    pub fn query_pen_state(&mut self) -> Result<bool, DeviceError> {
        let cmd = "QP"; // Команда для запиту стану
        debug!("Відправлення команди: {}", cmd);

        let response = self.command(cmd)?; // Відправляємо команду на пристрій
        debug!("Отримано відповідь: {}", response.trim());

        // Парсимо відповідь, очікуємо 0 (опущена) або 1 (піднята)
        let pen_status = response.trim().chars().next().ok_or_else(|| {
            error!("Некоректна відповідь від пристрою: порожня відповідь");
            DeviceError::InvalidResponse("Порожня відповідь від пристрою".to_string())
        })?;

        // Повертаємо стан ручки на основі відповіді
        match pen_status {
            '0' => Ok(true),  // Ручка опущена
            '1' => Ok(false), // Ручка піднята
            _ => {
                error!("Невідома відповідь від пристрою: {}", response.trim());
                Err(DeviceError::InvalidResponse(format!(
                    "Невідома відповідь від пристрою: {}",
                    response
                )))
            }
        }
    }

    /// Обнулити глобальні позиції кроків моторів 1 і 2
    ///
    /// Ця команда скидає (обнуляє) глобальні позиції моторів 1 та 2.
    /// Після цього крокові двигуни будуть вважати поточну позицію як (0, 0).
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Успішне виконання або помилка у разі невдачі.
    pub fn zero_position(&mut self) -> Result<(), DeviceError> {
        let cmd = "CS"; // Команда для обнулення глобальних позицій
        debug!("Відправлення команди: {}", cmd);

        self.command(cmd)?; // Виконуємо команду на пристрої
        info!("Команда CS виконана успішно");

        Ok(())
    }

    /// Виконує низькорівневу команду руху (LM) для керування осями
    ///
    /// Ця команда дозволяє керувати кроковими двигунами обох осей з можливістю
    /// задання початкової швидкості, кількості кроків і прискорення для кожної осі окремо.
    /// Команда також дозволяє обнулити акумулятор кроків для кожної осі перед початком руху.
    ///
    /// # Параметри:
    /// - `rate1`: Початкова швидкість для осі 1. Це беззнакове ціле число в діапазоні від 0 до 2147483647.
    /// - `steps1`: Кількість кроків для осі 1. Це знакове ціле число, де знак визначає напрямок руху.
    /// - `accel1`: Прискорення для осі 1. Це знакове ціле число, яке додається до швидкості на кожному кроці.
    /// - `rate2`: Початкова швидкість для осі 2. Це беззнакове ціле число в діапазоні від 0 до 2147483647.
    /// - `steps2`: Кількість кроків для осі 2. Це знакове ціле число, де знак визначає напрямок руху.
    /// - `accel2`: Прискорення для осі 2. Це знакове ціле число, яке додається до швидкості на кожному кроці.
    /// - `clear`: (опціонально) Число в діапазоні від 0 до 3. Якщо передано значення 1, то акумулятор для осі 1 обнуляється. Якщо 2 — для осі 2, якщо 3 — для обох осей.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку в разі невдачі.
    ///
    /// # Обмеження:
    /// - Принаймні одна з осей повинна мати ненульову кількість кроків або ненульові швидкість/прискорення, інакше рух не відбудеться.
    pub fn low_level_move(
        &mut self,
        rate1: u32,        // Початкова швидкість для осі 1
        steps1: i32,       // Кількість кроків для осі 1
        accel1: i32,       // Прискорення для осі 1
        rate2: u32,        // Початкова швидкість для осі 2
        steps2: i32,       // Кількість кроків для осі 2
        accel2: i32,       // Прискорення для осі 2
        clear: Option<u8>, // Очищення акумулятора: 0 - не очищати, 1 - очистити для осі 1, 2 - для осі 2, 3 - для обох
    ) -> Result<String, DeviceError> {
        // Валідація значення параметра `clear`
        if let Some(c) = clear {
            if c > 3 {
                return Err(DeviceError::CommandError(
                    "Недійсне значення для параметра Clear".to_string(),
                ));
            }
        }

        // Формуємо команду "LM,Rate1,Steps1,Accel1,Rate2,Steps2,Accel2[,Clear]"
        let clear_param = clear.map_or("".to_string(), |c| format!(",{}", c));
        let cmd = format!(
            "LM,{},{},{},{},{},{}{}",
            rate1, steps1, accel1, rate2, steps2, accel2, clear_param
        );

        debug!("Відправлення команди: {}", cmd);
        let response = self.command(&cmd)?;
        info!("Команда LM виконана успішно: {}", response.trim());

        Ok(response)
    }

    /// Команда для повернення моторів до "домашньої" позиції або до абсолютної позиції (HM).
    ///
    /// Ця команда рухає мотори з поточної позиції до позиції (0, 0) або до нової абсолютної позиції,
    /// зазначеної відносно домашньої. Це єдина команда, яка дозволяє вказати абсолютну позицію для руху.
    ///
    /// # Параметри:
    /// - `step_frequency`: Частота кроків у діапазоні від 2 до 25000, що представляє швидкість руху.
    /// - `position1`: (опціонально) Абсолютна позиція для мотора 1, у діапазоні ±4,294,967.
    /// - `position2`: (опціонально) Абсолютна позиція для мотора 2, у діапазоні ±4,294,967.
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає відповідь від пристрою або помилку в разі невдачі.
    pub fn home(
        &mut self,
        step_frequency: u32,    // Частота кроків, кроків на секунду
        position1: Option<i32>, // Абсолютна позиція для мотора 1 (опціонально)
        position2: Option<i32>, // Абсолютна позиція для мотора 2 (опціонально)
    ) -> Result<(), DeviceError> {
        if step_frequency < 2 || step_frequency > 25000 {
            return Err(DeviceError::CommandError(
                "Частота кроків повинна бути в діапазоні від 2 до 25000".to_string(),
            ));
        }

        let mut cmd = format!("HM,{}", step_frequency);

        // Додаємо абсолютні позиції для моторів, якщо вони вказані
        if let Some(pos1) = position1 {
            if pos1.abs() > 4_294_967 {
                return Err(DeviceError::CommandError(
                    "Позиція мотора 1 повинна бути в діапазоні ±4,294,967".to_string(),
                ));
            }
            cmd.push_str(&format!(",{}", pos1));
        }

        if let Some(pos2) = position2 {
            if pos2.abs() > 4_294_967 {
                return Err(DeviceError::CommandError(
                    "Позиція мотора 2 повинна бути в діапазоні ±4,294,967".to_string(),
                ));
            }
            cmd.push_str(&format!(",{}", pos2));
        }

        debug!("Відправлення команди: {}", cmd);
        self.command(&cmd)?;
        info!("Команда HM виконана успішно");

        Ok(())
    }

    /// Команда для прямолінійного руху або затримки (SM).
    ///
    /// Ця команда дозволяє керувати двигунами для виконання прямолінійного руху з постійною швидкістю.
    /// Якщо обидва значення `AxisSteps1` і `AxisSteps2` дорівнюють нулю, команда просто додасть затримку.
    ///
    /// # Параметри:
    /// - `duration`: Тривалість руху або затримки у мілісекундах (діапазон 1 - 16777215).
    /// - `axis_steps1`: Кількість кроків для осі 1 (діапазон -16777215 до +16777215).
    /// - `axis_steps2`: (опціонально) Кількість кроків для осі 2 (діапазон -16777215 до +16777215).
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає успіх або помилку у разі невдачі.
    pub fn stepper_move(
        &mut self,
        duration: Duration,       // Тривалість у мілісекундах
        axis_steps1: i32,         // Кількість кроків для осі 1
        axis_steps2: Option<i32>, // Опціонально: кількість кроків для осі 2
    ) -> Result<(), DeviceError> {
        // Конвертуємо тривалість у мілісекунди
        let duration_ms = duration.as_millis();

        // Перевіряємо діапазон значень для тривалості і кроків
        if duration_ms < 1 || duration_ms > 16777215 {
            return Err(DeviceError::CommandError(
                "Тривалість повинна бути в діапазоні 1 - 16777215 мс".to_string(),
            ));
        }

        if axis_steps1.abs() > 16777215 {
            return Err(DeviceError::CommandError(
                "Кроки для осі 1 повинні бути в діапазоні -16777215 до +16777215".to_string(),
            ));
        }

        if let Some(steps2) = axis_steps2 {
            if steps2.abs() > 16777215 {
                return Err(DeviceError::CommandError(
                    "Кроки для осі 2 повинні бути в діапазоні -16777215 до +16777215".to_string(),
                ));
            }
        }

        // Формуємо команду "SM,Duration,AxisSteps1[,AxisSteps2]"
        let cmd = if let Some(steps2) = axis_steps2 {
            format!("SM,{},{},{}", duration_ms, axis_steps1, steps2)
        } else {
            format!("SM,{},{}", duration_ms, axis_steps1)
        };

        debug!("Відправлення команди: {}", cmd);
        self.command(&cmd)?;
        info!("Команда SM виконана успішно");

        Ok(())
    }

    /// Виконує команду руху для змішаних осей (XM).
    ///
    /// Ця команда дозволяє керувати осями A і B для систем зі змішаною геометрією, такими як CoreXY або H-Bot.
    /// Команда перетворює кроки по осях A і B в еквівалентні значення для осей 1 і 2 за допомогою формул:
    /// - AxisSteps1 = AxisStepsA + AxisStepsB
    /// - AxisSteps2 = AxisStepsA - AxisStepsB
    ///
    /// Якщо обидва кроки осей A і B дорівнюють нулю, виконується затримка на тривалість `duration`.
    ///
    /// # Параметри:
    /// - `duration`: Тривалість руху у мілісекундах у форматі `Duration` (діапазон 1-16777215 мс).
    /// - `axis_steps_a`: Кількість кроків для осі A (діапазон -16777215 до +16777215).
    /// - `axis_steps_b`: Кількість кроків для осі B (діапазон -16777215 до +16777215).
    ///
    /// # Повертає:
    /// - `Result<(), DeviceError>`: Повертає успіх або помилку у разі невдачі.
    pub fn stepper_move_mixed(
        &mut self,
        duration: Duration, // Тривалість у форматі `Duration`
        axis_steps_a: i32,  // Кількість кроків для осі A
        axis_steps_b: i32,  // Кількість кроків для осі B
    ) -> Result<(), DeviceError> {
        // Конвертуємо тривалість у мілісекунди
        let duration_ms = duration.as_millis();

        // Перевіряємо діапазон значень для тривалості і кроків
        if duration_ms < 1 || duration_ms > 16777215 {
            return Err(DeviceError::CommandError(
                "Тривалість повинна бути в діапазоні 1 - 16777215 мс".to_string(),
            ));
        }

        if axis_steps_a.abs() > 16777215 || axis_steps_b.abs() > 16777215 {
            return Err(DeviceError::CommandError(
                "Кроки повинні бути в діапазоні -16777215 до +16777215 для обох осей A і B"
                    .to_string(),
            ));
        }

        // Формуємо команду "XM,Duration,AxisStepsA,AxisStepsB"
        let cmd = format!("XM,{},{},{}", duration_ms, axis_steps_a, axis_steps_b);

        debug!("Відправлення команди: {}", cmd);
        self.command(&cmd)?;
        info!("Команда XM виконана успішно");

        Ok(())
    }

    /// Увімкнення/вимкнення моторів з встановленням глобального режиму кроків.
    ///
    /// # Параметри:
    /// - `motor1_enable`: Опціональний параметр для увімкнення/вимкнення мотора 1.
    /// - `motor2_enable`: Опціональний параметр для увімкнення/вимкнення мотора 2.
    /// - `step_mode`: Опціональний режим кроків.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку.
    fn set_motors(
        &mut self,
        motor1_enable: Option<bool>,
        motor2_enable: Option<bool>,
        step_mode: Option<StepMode>,
    ) -> Result<String, DeviceError> {
        let m1_enable = motor1_enable.unwrap_or(self.motor1_enabled);
        let m2_enable = motor2_enable.unwrap_or(self.motor2_enabled);
        let mode = step_mode.unwrap_or(self.step_mode);

        let enable1 = if m1_enable { mode as u8 } else { 0 };
        let enable2 = if m2_enable { 1 } else { 0 }; // Enable2 не змінює step_mode

        let cmd = format!("EM,{},{}", enable1, enable2);

        debug!("Відправлення команди: {}", cmd);
        let response = self.command(&cmd)?;
        info!("Команда EM виконана: {}", response.trim());

        self.motor1_enabled = m1_enable;
        self.motor2_enabled = m2_enable;
        self.step_mode = mode;

        Ok(response)
    }

    /// Вимкнення моторів.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку.
    pub fn disable_motors(&mut self) -> Result<String, DeviceError> {
        self.set_motors(Some(false), Some(false), None)
    }

    /// Увімкнення моторів з встановленням глобального режиму кроку.
    ///
    /// # Параметри:
    /// - `step_mode`: Режим кроку для обох моторів.
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку.
    pub fn enable_motors(&mut self, step_mode: StepMode) -> Result<String, DeviceError> {
        self.set_motors(Some(true), Some(true), Some(step_mode))
    }

    /// Негайна зупинка моторів.
    ///
    /// # Параметри:
    /// - `disable_motors`: Вимкнути мотори після зупинки (true) або залишити увімкненими (false).
    ///
    /// # Повертає:
    /// - `Result<String, DeviceError>`: Повертає відповідь від пристрою або помилку.
    pub fn abort_motors(&mut self, disable_motors: bool) -> Result<String, DeviceError> {
        let cmd = if disable_motors {
            "ES,1".to_string()
        } else {
            "ES".to_string()
        };

        debug!("Відправлення команди: {}", cmd);
        let response = self.command(&cmd)?;
        info!("Команда ES виконана: {}", response.trim());

        if disable_motors {
            self.motor1_enabled = false;
            self.motor2_enabled = false;
            self.step_mode = StepMode::Disable;
        }

        Ok(response)
    }

    /// Читання стану моторів і глобального режиму кроку.
    ///
    /// # Повертає:
    /// - `Result<(bool, bool, StepMode), DeviceError>`: Повертає стан моторів 1 і 2, а також глобальний режим кроку.
    fn query_enable_motors(&mut self) -> Result<(bool, bool, StepMode), DeviceError> {
        let motor1_enabled = self.read_pin('E', 0)?;
        let motor2_enabled = self.read_pin('C', 1)?;

        let ms1_active = self.read_pin('E', 2)?;
        let ms2_active = self.read_pin('E', 1)?;
        let ms3_active = self.read_pin('A', 6)?;

        let step_mode = match (ms1_active, ms2_active, ms3_active) {
            (true, true, true) => StepMode::OneSixteenth,
            (true, true, false) => StepMode::OneEighth,
            (false, true, false) => StepMode::OneQuarter,
            (true, false, false) => StepMode::OneHalf,
            (false, false, false) => StepMode::FullStep,
            _ => {
                error!("Некоректна комбінація станів MS пінів");
                return Err(DeviceError::InvalidResponse(
                    "Некоректна комбінація станів MS пінів".to_string(),
                ));
            }
        };

        Ok((motor1_enabled, motor2_enabled, step_mode))
    }

    /// Запитуємо стан обох моторів (QM).
    ///
    /// # Повертає:
    /// - `Result<(MotorStatus, MotorStatus), DeviceError>`: Повертає статус моторів або помилку.
    pub fn motor_status(&mut self) -> Result<(MotorStatus, MotorStatus), DeviceError> {
        let cmd = "QM";

        debug!("Відправлення команди: {}", cmd);
        let response = self.command(cmd)?;
        debug!("Отримано відповідь: {}", response.trim());

        let parts: Vec<&str> = response.trim().split(',').collect();

        if parts.len() == 5 && parts[0] == "QM" {
            let command_status = parts[1].trim().parse::<u8>().unwrap_or(0) != 0;
            let motor1_moving = parts[2].trim().parse::<u8>().unwrap_or(0) != 0;
            let motor2_moving = parts[3].trim().parse::<u8>().unwrap_or(0) != 0;
            let fifo_empty = parts[4].trim().parse::<u8>().unwrap_or(1) == 0;

            let motor1_status = MotorStatus {
                executing_command: command_status,
                moving: motor1_moving,
                fifo_empty,
            };

            let motor2_status = MotorStatus {
                executing_command: command_status,
                moving: motor2_moving,
                fifo_empty,
            };

            Ok((motor1_status, motor2_status))
        } else {
            error!("Некоректна відповідь від QM: {}", response.trim());
            Err(DeviceError::InvalidResponse(
                "Некоректна відповідь від QM".to_string(),
            ))
        }
    }

    /// Зчитує глобальні позиції кроків моторів 1 та 2 (QS).
    ///
    /// Цей метод відправляє команду `QS` на пристрій, яка повертає поточні глобальні позиції кроків
    /// для моторів 1 і 2. Кожна з цих позицій є 32-бітним цілим числом і відображає поточне положення осі.
    ///
    /// # Повертає:
    /// - `Result<(i32, i32), DeviceError>`: Повертає кортеж з двох значень (позиція мотора 1, позиція мотора 2),
    /// або помилку в разі невдачі.
    pub fn read_position(&mut self) -> Result<(i32, i32), DeviceError> {
        let cmd = "QS";
        debug!("Відправлення команди: {}", cmd);

        let response = self.command(cmd)?;
        debug!("Отримано відповідь від QS: {}", response.trim());

        let positions: Vec<&str> = response.trim().split(',').collect();
        if positions.len() != 2 {
            return Err(DeviceError::InvalidResponse(
                "Некоректна відповідь від QS: очікувалося два значення".to_string(),
            ));
        }

        let motor1_position = positions[0].parse::<i32>().map_err(|_| {
            DeviceError::InvalidResponse(format!(
                "Некоректне значення для позиції мотора 1: {}",
                positions[0]
            ))
        })?;

        let motor2_position = positions[1].parse::<i32>().map_err(|_| {
            DeviceError::InvalidResponse(format!(
                "Некоректне значення для позиції мотора 2: {}",
                positions[1]
            ))
        })?;

        Ok((motor1_position, motor2_position))
    }

    /// Відключення пристрою
    pub fn disconnect(&mut self) {
        if self.connected {
            info!("Закриваємо порт...");
            self.connected = false;
        }
    }
}

/// Автоматичне відключення пристрою при його знищенні
impl Drop for Device {
    fn drop(&mut self) {
        self.disconnect();
        info!("Пристрій відключено");
    }
}
