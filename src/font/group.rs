use phf::phf_map;

// Include generated maps
include!(concat!(env!("OUT_DIR"), "/occidental_fonts.rs"));
include!(concat!(env!("OUT_DIR"), "/occidental_unicode_map.rs"));
include!(concat!(env!("OUT_DIR"), "/oriental_fonts.rs"));

/// Типи груп шрифтів.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FontGroupType {
    Occidental, // Західна група шрифтів: містить символи латинського алфавіту та інші західні символи.
    Oriental, // Східна група шрифтів: містить символи східних мов, таких як китайська, японська, корейська.
}

/// Група шрифтів, яка містить інформацію про тип групи, відповідні файли шрифтів та мапу відповідностей Unicode.
#[derive(Debug, Clone)]
pub struct FontGroup {
    pub group_type: FontGroupType, // Тип групи шрифтів (Occidental або Oriental).
    pub fonts: &'static phf::Map<u32, &'static str>, // Статичні змінні, які містять мапи гліфів.
    pub unicode_map: &'static phf::Map<u32, u32>, // Мапа відповідностей Unicode для цієї групи шрифтів.
}

/// Порожня мапа для випадків, коли відповідностей немає.
static EMPTY_UNICODE_MAP: phf::Map<u32, u32> = phf_map! {};

/// Статичні екземпляри для груп Occidental та Oriental.
pub static OCCIDENTAL_FONT_GROUP: FontGroup = FontGroup {
    group_type: FontGroupType::Occidental,
    fonts: &OCCIDENTAL_HERSHEY_FONTS,
    unicode_map: &HERSHEY_OCCIDENTAL_UNICODE_MAP,
};

pub static ORIENTAL_FONT_GROUP: FontGroup = FontGroup {
    group_type: FontGroupType::Oriental,
    fonts: &ORIENTAL_HERSHEY_FONTS,
    unicode_map: &EMPTY_UNICODE_MAP,
};
