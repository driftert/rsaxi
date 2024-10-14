use anyhow::Result;
use env_logger::Env;
use font::{script::Script, variant::Simplex};
use geo::Point;

// Імпортуємо модулі
mod axidraw;
mod device;
mod drawing;
mod font;
mod motion;
mod text;

// Імпортуємо необхідні компоненти з модулів
use drawing::{Drawable, Drawing};
use text::Text;

fn main() -> Result<()> {
    // Ініціалізуємо логер з налаштуваннями за замовчуванням (рівень info)
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Крок 1: Ініціалізуємо Roman шрифт
    log::info!("Ініціалізація Roman шрифту...");
    let roman = Script::new();

    // Крок 2: Створюємо Plain варіант Roman шрифту
    log::info!("Створення Plain варіанту Roman шрифту...");
    let simplex_font = roman.simplex()?;

    // Крок 3: Створюємо екземпляр Text за допомогою TextBuilder
    log::info!("Створення об'єкта Text...");
    let text = Text::builder()
        .content("Hershey")
        .font(simplex_font)
        .position(Point::new(50.0, 50.0)) // Початкова позиція (x=50.0, y=50.0)
        .scale(1.0) // Масштаб
        .build()?;

    // Крок 4: Ініціалізуємо новий Drawing з відповідними межами
    let bounds = (300.0, 200.0); // Приклад меж (ширина, висота)
    let mut drawing = Drawing::new(bounds, text.draw()?);

    // Крок 6: Конвертуємо Drawing у SVG
    log::info!("Конвертація Drawing у SVG...");
    let svg_content = drawing.to_svg();

    // Крок 7: Записуємо SVG контент у файл
    log::info!("Запис SVG у файл 'output.svg'...");
    std::fs::write("output.svg", svg_content)?;

    log::info!("Файл SVG 'output.svg' успішно створено.");

    Ok(())
}
