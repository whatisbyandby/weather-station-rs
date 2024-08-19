use defmt::*;
use embassy_rp::pwm::Pwm;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Ticker};

#[derive(Debug)]
pub enum WindDirection {
	N,
	NNE ,
	NE,
	ENE,
	E,
	ESE,
	SE,
	SSE,
	S,
	SSW,
	SW,
	WSW,
	W,
	WNW,
	NW,
	NNW,
}


#[derive(Debug)]
pub struct WindReading {
	pub wind_speed: f32,
	pub wind_direction: WindDirection
}

pub async fn wind_sensor_task(wind_sensor_pwm: Pwm<'static>, wind_channel: &Channel<ThreadModeRawMutex, WindReading, 14>) {
	let mut ticker = Ticker::every(Duration::from_secs(5));

	let wind_channel_sender = wind_channel.sender();

	let mut start = Instant::now();

	loop {
		
		debug!("This is the wind sensor loop");
		ticker.next().await;
		let wind_speed_count = wind_sensor_pwm.counter();
		wind_sensor_pwm.set_counter(0);
		let end = Instant::now();
		let duration = end.duration_since(start);

		start = end;

		let wind_speed = ((wind_speed_count * 1000) as f32 / (duration.as_millis()) as f32) * 2.4;
		wind_channel_sender.send(
			WindReading {
				wind_speed,
				wind_direction: WindDirection::E
			}
		).await;
	}
}
