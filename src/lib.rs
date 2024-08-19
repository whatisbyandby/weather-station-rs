#![no_std]

use core::cell::RefCell;

use embassy_rp::{peripherals::SPI0, spi::{Blocking, Spi}};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use serde::{Deserialize, Serialize};

pub mod radio;
pub mod wind_sensor;

pub type Spi0Bus = Mutex<NoopRawMutex, RefCell<Spi<'static, SPI0, Blocking>>>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EnvironmentReading {
    pub current_temperature: f32,
    pub current_humidity: f32,
    pub current_pressure: f32,
    pub current_wind_speed: f32,
    pub current_wind_gust: f32,
    pub current_wind_direction: f32,
    pub current_rainfall: f32,
    pub battery_level: f32,
}