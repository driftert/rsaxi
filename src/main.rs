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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Налаштування опцій Axidraw з вибраною моделлю
    let options = Options {
        model: AxiDrawModel::Mini,
        ..Options::default()
    };

    // Ініціалізація Axidraw з вказаними опціями
    let mut axidraw = Axidraw::new(options)?;

    // Отримання розмірів моделі
    let model_width = axidraw.options.model.width();
    let model_height = axidraw.options.model.height();

    // Визначення меж малюнка на основі розмірів моделі з додаванням відступів
    let margin = 10.0; // Відступ у мм
    let drawing_bounds = (model_width - 2.0 * margin, model_height - 2.0 * margin);

    // Створення квадратного малюнка в межах робочої області
    let square = Drawing::new(
        drawing_bounds, // Межі: ширина і висота в мм
        MultiLineString(vec![vec![
            Point::new(margin, margin),
            Point::new(margin, drawing_bounds.1 - margin),
            Point::new(drawing_bounds.0 - margin, drawing_bounds.1 - margin),
            Point::new(drawing_bounds.0 - margin, margin),
            Point::new(margin, margin),
        ]
        .into()]),
    );

    // Виконання малювання
    axidraw.draw(&square)?;

    Ok(())
}
