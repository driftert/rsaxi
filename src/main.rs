use anyhow::Result;
use axidraw::{AxiDrawModel, Axidraw, Options};
use env_logger::Env;
use geo::{MultiLineString, Point};

// Імпортуємо модулі
mod axidraw;
mod device;
mod drawing;
mod font;
mod motion;
mod text;

// Імпортуємо необхідні компоненти з модулів
use drawing::Drawing;

fn main() -> Result<()> {
    // Ініціалізація логування з рівнем за замовчуванням "info"
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // Налаштування опцій Axidraw з вибраною моделлю
    let options = Options {
        model: AxiDrawModel::Mini,
        ..Options::default()
    };

    // Ініціалізація Axidraw з вказаними опціями
    let mut axidraw = Axidraw::new(options)?;

    // Створення квадратного малюнка розміром 5x5 мм, без відступів
    let square = Drawing::new(
        (100.0, 100.0),
        MultiLineString(vec![vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ]
        .into()]),
    );

    // Виконання малювання
    axidraw.draw(&square)?;

    Ok(())
}
