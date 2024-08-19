#![no_std]
#![no_main]

use core::cell::RefCell;

use embassy_rp::i2c;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::pwm::{Config, InputMode, Pwm};
use embassy_rp::spi::{self, Phase, Polarity, Spi};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::{Delay, Timer};
use bme280::i2c::BME280;
use heapless::Vec;

use {defmt_rtt as _, panic_probe as _};

use weather_station_rs::{EnvironmentReading, Spi0Bus};
use weather_station_rs::radio::rfm69_task;
use weather_station_rs::wind_sensor::{wind_sensor_task, WindReading};

static RADIO_CHANNEL: Channel<ThreadModeRawMutex, EnvironmentReading, 2> = Channel::new();
static WIND_CHANNEL: Channel<ThreadModeRawMutex, WindReading, 14> = Channel::new();

#[embassy_executor::task]
async fn wind_sensor(wind_sensor_pwm: Pwm<'static>) {
    wind_sensor_task(wind_sensor_pwm, &WIND_CHANNEL).await;
}

#[embassy_executor::task]
async fn radio_task(spi_bus: Spi0Bus, cs: Output<'static>) {
    rfm69_task(spi_bus, cs, &RADIO_CHANNEL).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // =================== PWM Setup ======================================

    let cfg: Config = Default::default();
    let pwm = Pwm::new_input(p.PWM_SLICE3, p.PIN_23, Pull::None, InputMode::RisingEdge, cfg);

    // =================== I2C Setup ======================================
    let sda = p.PIN_4;
    let scl = p.PIN_5;

    let i2c = i2c::I2c::new_blocking(p.I2C0, scl, sda, i2c::Config::default());

    let mut bme280 = BME280::new_secondary(i2c);

    // =============== SPI Setup ============================================
    let miso = p.PIN_16;
    let mosi = p.PIN_19;
    let clk = p.PIN_18;
    let rfm69_cs = p.PIN_17;

    // Shared SPI bus
    let mut rfm69_config = spi::Config::default();
    rfm69_config.frequency = 1_000_000;
    rfm69_config.phase = Phase::CaptureOnSecondTransition;
    rfm69_config.polarity = Polarity::IdleHigh;
    let spi = Spi::new_blocking(p.SPI0, clk, mosi, miso, rfm69_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    // Chip select pins for the SPI devices
    let cs = Output::new(rfm69_cs, Level::High);

    spawner.must_spawn(wind_sensor(pwm));
    spawner.must_spawn(radio_task(spi_bus, cs));

    let mut delay = Delay;

    bme280.init(&mut delay).unwrap();

    let control = RADIO_CHANNEL.sender();
    loop {
        
        let measurement = bme280.measure(&mut delay).unwrap();
        
        Timer::after_secs(60).await;
        WIND_CHANNEL.ready_to_receive().await;
        let mut wind_readings: Vec<WindReading, 24> = Vec::new();
        while !WIND_CHANNEL.is_empty() {
            let new_reading = WIND_CHANNEL.receive().await;
            wind_readings.push(new_reading).unwrap();
        }

        info!("Num Wind Readings: {}", wind_readings.len());

        let env_reading = EnvironmentReading {
            current_temperature: measurement.temperature,
            current_humidity: measurement.humidity,
            current_pressure: measurement.pressure,
            current_wind_speed: 0.0,
            current_wind_gust: 0.0,
            current_wind_direction: 0.0,
            current_rainfall: 0.0,
            battery_level: 0.0,
        };
        control.send(env_reading).await;
    }
}
