use thiserror::Error;

/// Перелік можливих помилок, що виникають при роботі зі шрифтами.
#[derive(Debug, Error)]
pub enum FontError {
    /// Помилка при парсингу окремого гліфу.
    #[error("Помилка парсингу гліфу: {glyph}. Причина: {message}")]
    GlyphParsingError {
        glyph: String,   // Символ гліфа, який викликав помилку.
        message: String, // Повідомлення про причину помилки парсингу.
    },

    /// Помилка при створенні шрифту з файлу офсетів.
    #[error("Помилка під час створення шрифту з файлу: {file}. Причина: {source}")]
    FontCreationError {
        file: String, // Назва файлу офсетів, який викликав помилку.
        #[source] // Додаткова інформація про помилку створення шрифту.
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Загальна помилка шрифту.
    #[error("Загальна помилка шрифту: {0}")]
    GenericError(String),
}
