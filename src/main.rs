use anyhow::Result;
use axidraw::{AxiDrawModel, Axidraw, Options};
use env_logger::Env;
use font::{roman::Roman, variant::Simplex};
use geo::{Coord, LineString, MultiLineString};

// Імпортуємо модулі
mod axidraw;
mod device;
mod drawing;
mod font;
mod motion;
mod text;

// Імпортуємо необхідні компоненти з модулів
use drawing::Drawing;
use text::Text;

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
    let roman_simplex = Roman::new().simplex()?;
    let text = Text::builder()
        .content("H")
        .font(roman_simplex)
        .scale(1.0)
        .spacing(0.5)
        .build()?;

    // Створення малюнка з двома лініями
    let lines = MultiLineString(vec![
        LineString(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 50.0, y: 50.0 }]),
        LineString(vec![Coord { x: 50.0, y: 50.0 }, Coord { x: 100.0, y: 0.0 }]),
    ]);

    // Створення малюнка з використанням багатолінійної геометрії
    let drawing = Drawing::new((100.0, 100.0), lines);

    // Виконання малювання
    axidraw.draw(&drawing)?;

    Ok(())
}
