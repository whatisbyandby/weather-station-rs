
use defmt::info;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::gpio::Output;
use embassy_rp::spi::{self, Phase, Polarity};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use heapless::Vec;
use postcard::to_vec;
use rfm69::Rfm69;
use rfm69::registers::Registers;

use crate::{EnvironmentReading, Spi0Bus};


pub async fn rfm69_task(spi_bus: Spi0Bus, cs: Output<'static>, chan: &Channel<ThreadModeRawMutex, EnvironmentReading, 2> ) {
	let mut config = spi::Config::default();
    config.frequency = 1_000_000;
    config.phase = Phase::CaptureOnSecondTransition;
    config.polarity = Polarity::IdleHigh;
    let spi_device = SpiDeviceWithConfig::new(&spi_bus, cs, config);
    let mut rfm = Rfm69::new(spi_device);

    let data: [u8; 79] = [
        0x04, 0x01, 0x00, 0x80, 0x10, 0x00, 0xE4, 0xC0, 0x00, 0x41, 0x40, 0x02, 0x92, 0xF5, 0x20,
        0x24, 0x7C, 0x09, 0x1A, 0x40, 0xB0, 0x7B, 0x9B, 0x08, 0xE0, 0xE0, 0x40, 0x80, 0x06, 0x10,
        0x00, 0x00, 0x00, 0x00, 0x02, 0xFF, 0x00, 0x05, 0x80, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x04,
        0x88, 0x2D, 0xD4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x40, 0x00, 0x00, 0x00, 0x8F,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00,
    ];

    rfm.write_many(Registers::OpMode, &data).unwrap();

    rfm.tx_level_high_pwr(16).unwrap();
    let regs = rfm.read_all_regs().unwrap();

    info!("{=[u8]:#04x}", regs);

	loop {
        let env_reading = chan.receive().await;
        let ser_reading: Vec<u8, 50> = to_vec(&env_reading).unwrap();

        let mut packet: Vec<u8, 60> = Vec::new();

        packet.push((ser_reading.len() + 5) as u8).unwrap();
        packet.extend_from_slice(&[255, 255, 0, 0]).unwrap();
        packet.extend_from_slice(ser_reading.as_slice()).unwrap();
        packet.push(0).unwrap();

        rfm.send(packet.as_slice()).unwrap();
    }
}

