use crate::device::{Device, DeviceError};
use crate::drawing::Drawing;

/// Представляє модель машини AxiDraw.
#[derive(Debug, Clone, Copy)]
pub enum AxiDrawModel {
    V3,   // "AxiDraw V3", ширина: 215.9 мм, висота: 279.4 мм
    V3A3, // "AxiDraw V3/A3", ширина: 279.4 мм, висота: 431.8 мм
    SEA3, // "AxiDraw SE/A3", ширина: 279.4 мм, висота: 431.8 мм
    Mini, // "MiniKit2", ширина: 152.4 мм, висота: 101.6 мм
}

impl AxiDrawModel {
    pub fn name(&self) -> &'static str {
        match self {
            AxiDrawModel::V3 => "AxiDraw V3",
            AxiDrawModel::V3A3 => "AxiDraw V3/A3",
            AxiDrawModel::SEA3 => "AxiDraw SE/A3",
            AxiDrawModel::Mini => "MiniKit2",
        }
    }

    pub fn width(&self) -> f64 {
        match self {
            AxiDrawModel::V3 => 215.9,
            AxiDrawModel::V3A3 | AxiDrawModel::SEA3 => 279.4,
            AxiDrawModel::Mini => 152.4,
        }
    }

    pub fn height(&self) -> f64 {
        match self {
            AxiDrawModel::V3 => 279.4,
            AxiDrawModel::V3A3 | AxiDrawModel::SEA3 => 431.8,
            AxiDrawModel::Mini => 101.6,
        }
    }
}

/// Структура для керування AxiDraw.
pub struct Axidraw {
    pub device: Device,
    pub model: AxiDrawModel,
}

impl Axidraw {
    /// Створює новий екземпляр `Axidraw` з вибраною моделлю.
    /// При створенні також ініціалізується `Device`.
    pub fn new(model: AxiDrawModel) -> Result<Self, DeviceError> {
        let device = Device::new()?;
        Ok(Self { device, model })
    }

    /// Метод для малювання, який приймає `Drawing`.
    pub fn draw(&mut self, drawing: &Drawing) -> Result<(), DeviceError> {
        Ok(())
    }
}
