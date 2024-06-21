mod file;

pub use file::FileCamera;

use crate::webrtc::Sample;

#[derive(Debug, thiserror::Error)]
pub enum CameraError {
	#[error("Camera was disconnected")]
	Disconnected,
}

pub trait Camera {
	/// duration get's written by the caller
	fn next_sample(&mut self) -> Result<Sample, CameraError>;
}
