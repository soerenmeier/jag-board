use super::{Camera, CameraError};
use crate::webrtc::Sample;

use std::fs::File;
use std::io::BufReader;

use webrtc::media::io::h264_reader::H264Reader;

pub struct FileCamera {
	reader: H264Reader<BufReader<File>>,
}

impl FileCamera {
	pub fn new(path: &str) -> Self {
		let file = File::open("./h264.h264").unwrap();
		let reader = H264Reader::new(BufReader::new(file));

		Self { reader }
	}
}

impl Camera for FileCamera {
	fn next_sample(&mut self) -> Result<Sample, CameraError> {
		let nal = match self.reader.next_nal() {
			Ok(n) => n,
			Err(e) => {
				eprintln!("failed to read nal probably file streamed {:?}", e);
				return Err(CameraError::Disconnected);
			}
		};

		Ok(Sample {
			data: nal.data.freeze(),
			..Default::default()
		})
	}
}

/*
let file = File::open("./h264.h264").unwrap();
let reader = BufReader::new(file);
let mut h264 = H264Reader::new(reader);

notify_video.notified().await;

eprintln!("play from disk");

let mut ticker = tokio::time::interval(Duration::from_millis(33));
loop {
	let nal = match h264.next_nal() {
		Ok(nal) => nal,
		Err(e) => {
			eprintln!("All video frames parsed and sent: {:?}", e);
			break
		}
	};

	video_track.write_sample(&Sample {
		data: nal.data.freeze(),
		duration: Duration::from_secs(1),
		..Default::default()
	}).await.unwrap();

	let _ = ticker.tick().await;
}
*/
