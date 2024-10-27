use anyhow::Result;
use axidraw::{AxiDrawModel, Axidraw, Options};
use clap::{Arg, Command};
use env_logger::Env;
use log::{error, info};

// Імпортуємо модулі
mod axidraw;
mod device;
mod drawing;
mod motion;
mod text;

fn main() -> Result<()> {
    // Ініціалізація логування з рівнем за замовчуванням "info"
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Налаштування CLI за допомогою clap
    let matches = Command::new("rsaxi")
        .version("0.1.0")
        .author("Taras Koval <tkoval83@icloud.com>")
        .about("Командний інтерфейс для налаштування і керування AxiDraw")
        .arg(
            Arg::new("steps_per_unit")
                .long("steps_per_unit")
                .help("Кроки на одиницю (наприклад, мм)")
                .value_name("STEPS")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_up_position")
                .long("pen_up_position")
                .help("Положення піднятої ручки")
                .value_name("HEIGHT")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_up_speed")
                .long("pen_up_speed")
                .help("Швидкість підйому ручки")
                .value_name("SPEED")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_up_delay")
                .long("pen_up_delay")
                .help("Затримку після підйому ручки (в мілісекундах)")
                .value_name("DELAY")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_down_position")
                .long("pen_down_position")
                .help("Положення опущеної ручки")
                .value_name("HEIGHT")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_down_speed")
                .long("pen_down_speed")
                .help("Швидкість опускання ручки")
                .value_name("SPEED")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("pen_down_delay")
                .long("pen_down_delay")
                .help("Затримку після опускання ручки (в мілісекундах)")
                .value_name("DELAY")
                .required(false)
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("acceleration")
                .long("acceleration")
                .help("Прискорення")
                .value_name("ACCELERATION")
                .required(false)
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(
            Arg::new("max_velocity")
                .long("max_velocity")
                .help("Максимальну швидкість")
                .value_name("VELOCITY")
                .required(false)
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(
            Arg::new("corner_factor")
                .long("corner_factor")
                .help("Коефіцієнт для обробки кутів")
                .value_name("FACTOR")
                .required(false)
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(
            Arg::new("model")
                .long("model")
                .help("Модель AxiDraw: v3, v3a3, sea3, або mini")
                .value_name("MODEL")
                .required(false)
                .value_parser(["v3", "v3a3", "sea3", "mini"]),
        )
        .get_matches();

    // Ініціалізація стандартних опцій
    let mut options = Options::default();

    // Перевизначення опцій на основі введення CLI
    if let Some(steps_per_unit) = matches.get_one::<i32>("steps_per_unit") {
        options.steps_per_unit = *steps_per_unit;
    }
    if let Some(pen_up_position) = matches.get_one::<i32>("pen_up_position") {
        options.pen_up_position = *pen_up_position;
    }
    if let Some(pen_up_speed) = matches.get_one::<i32>("pen_up_speed") {
        options.pen_up_speed = *pen_up_speed;
    }
    if let Some(pen_up_delay) = matches.get_one::<i32>("pen_up_delay") {
        options.pen_up_delay = *pen_up_delay;
    }
    if let Some(pen_down_position) = matches.get_one::<i32>("pen_down_position") {
        options.pen_down_position = *pen_down_position;
    }
    if let Some(pen_down_speed) = matches.get_one::<i32>("pen_down_speed") {
        options.pen_down_speed = *pen_down_speed;
    }
    if let Some(pen_down_delay) = matches.get_one::<i32>("pen_down_delay") {
        options.pen_down_delay = *pen_down_delay;
    }
    if let Some(acceleration) = matches.get_one::<f64>("acceleration") {
        options.acceleration = *acceleration;
    }
    if let Some(max_velocity) = matches.get_one::<f64>("max_velocity") {
        options.max_velocity = *max_velocity;
    }
    if let Some(corner_factor) = matches.get_one::<f64>("corner_factor") {
        options.corner_factor = *corner_factor;
    }
    if let Some(model) = matches.get_one::<String>("model") {
        options.model = match model.as_str() {
            "v3" => AxiDrawModel::V3,
            "v3a3" => AxiDrawModel::V3A3,
            "sea3" => AxiDrawModel::SEA3,
            "mini" => AxiDrawModel::Mini,
            _ => unreachable!(),
        };
    }

    // Ініціалізація AxiDraw з модифікованими опціями
    let mut axidraw = Axidraw::new(options)?;

    // Приклад використання: підняти ручку для перевірки застосування опцій
    if let Err(e) = axidraw.device.pen_up() {
        error!("Помилка підняття ручки: {}", e);
        std::process::exit(1);
    }

    info!("CLI конфігурація успішно застосована!");

    Ok(())
}
