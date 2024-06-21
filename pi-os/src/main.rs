mod camera;
mod webrtc;

use crate::webrtc::Description;
use camera::FileCamera;

use std::fs::{self, File};

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
	env_logger::init();

	let offer = fs::read_to_string("./session.txt").unwrap();
	let offer = base64::decode(offer.as_str()).unwrap();
	let offer = serde_json::from_slice::<Description>(&offer).unwrap();

	let webrtc = crate::webrtc::Webrtc::new();
	let camera = FileCamera::new("./h264.h264");
	let con = webrtc
		.create_connection(offer, Box::new(camera))
		.await
		.unwrap();

	eprintln!("connection created");

	let answer = con.description().await;
	let answer = serde_json::to_string(&answer).unwrap();
	let answer = base64::encode(answer.as_str());

	eprintln!("answer {:?}", answer);

	sleep(Duration::from_secs(25)).await;

	con.close().await;
}
