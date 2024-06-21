mod twcc_interceptor;

use twcc_interceptor::TwccInterceptor;

use crate::camera::Camera;

use std::thread;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::time::{Instant, Duration};
use std::path::Path;
use std::sync::Arc;
use std::error::Error as StdError;

use tokio::sync::{Notify, mpsc};
use tokio::runtime::Handle as RtHandle;

use webrtc::api::interceptor_registry::{
	configure_nack, configure_rtcp_reports, configure_twcc
};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::interceptor::registry::Registry;
use webrtc::media::io::h264_reader::H264Reader;
use webrtc::media::io::ogg_reader::OggReader;
pub use webrtc::media::Sample;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
pub use webrtc::peer_connection::sdp::session_description::{
	RTCSessionDescription as Description
};
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::{
	TrackLocalStaticSample
};
use webrtc::data_channel::RTCDataChannel;
use webrtc::track::track_local::TrackLocal;
use webrtc::stats::StatsReportType;


#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("webrtc error")]
	WebrtcError(#[from] webrtc::Error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
	Connected,
	Disconnected
}

pub struct Webrtc {}

impl Webrtc {
	pub fn new() -> Self {
		Self {}
	}

	pub async fn create_connection(
		&self,
		desc: Description,
		camera: Box<dyn Camera + Send>
	) -> Result<Connection, Error> {
		let mut m = MediaEngine::default();

		/// cannot fail, see webrtc impl
		m.register_default_codecs().unwrap();

		let mut registry = Registry::new();

		registry = configure_nack(registry, &mut m);
		registry = configure_rtcp_reports(registry);
		registry = configure_twcc(registry, &mut m)?;
		registry.add(Box::new(TwccInterceptor::builder()));

		let api = APIBuilder::new()
			.with_media_engine(m)
			.with_interceptor_registry(registry)
			.build();

		let config = RTCConfiguration {
			ice_servers: vec![RTCIceServer {
				urls: vec!["stun:stun.l.google.com:19302".to_owned()],
				..Default::default()
			}],
			..Default::default()
		};

		let peer_connection = Arc::new(api.new_peer_connection(config).await?);

		let video_track = Arc::new(TrackLocalStaticSample::new(
			RTCRtpCodecCapability {
				mime_type: MIME_TYPE_H264.to_owned(),
				..Default::default()
			},
			"video".to_owned(),
			"webrtc-rs".to_owned()
		));

		let rtp_sender = peer_connection.add_track(
			Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>
		).await?;

		let (state_tx, state_rx) = mpsc::channel(5);

		let camera_peer_connection = peer_connection.clone();
		let rt_handle = RtHandle::current();
		thread::spawn(move || {
			camera_thread(
				camera,
				video_track,
				camera_peer_connection,
				state_rx,
				rt_handle
			);
		});

		// keep the rtp stream going
		tokio::spawn(async move {
			let mut rtcp_buf = vec![0u8; 1500];
			while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
		});

		// set the handler for ICE connection
		peer_connection.on_ice_connection_state_change(Box::new(
			move |connection_state: RTCIceConnectionState| {
				eprintln!("ice connection state changed {:?}", connection_state);
				// if connection_state == RTCIceConnectionState::Connected {
				// 	notify_tx.notify_waiters();
				// }

				Box::pin(async {})
			}
		));

		peer_connection.on_ice_candidate(Box::new(
			move |c: Option<RTCIceCandidate>| {
				eprintln!("ice candidate update {:?}", c);

				Box::pin(async {})
			}
		));

		peer_connection.on_peer_connection_state_change(
			Box::new(move |s: RTCPeerConnectionState| {
				eprintln!("peer connection state change {:?}", s);

				match s {
					RTCPeerConnectionState::Connected => {
						let _ = state_tx.try_send(State::Connected);
					},
					RTCPeerConnectionState::Disconnected |
					RTCPeerConnectionState::Failed |
					RTCPeerConnectionState::Closed => {
						let _ = state_tx.try_send(State::Disconnected);
					},
					_ => {}
				}

				Box::pin(async {})
			})
		);

		peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
			let d_label = d.label().to_owned();
			let d_id = d.id();
			println!("New DataChannel {} {}", d_label, d_id);

			Box::pin(async {})
		}));

		peer_connection.set_remote_description(desc).await?;

		let answer = peer_connection.create_answer(None).await?;

		peer_connection.set_local_description(answer).await?;

		Ok(Connection { peer_connection })
	}
}

pub struct Connection {
	peer_connection: Arc<RTCPeerConnection>
}

impl Connection {
	pub async fn description(&self) -> Description {
		self.peer_connection.local_description().await.unwrap()
	}

	pub async fn close(&self) {
		self.peer_connection.close().await.unwrap();
	}
}

// every x frames
const GATHER_STATS_EVERY: usize = 15;

fn camera_thread(
	mut camera: Box<dyn Camera>,
	track: Arc<TrackLocalStaticSample>,
	peer_connection: Arc<RTCPeerConnection>,
	mut state_rx: mpsc::Receiver<State>,
	rt: RtHandle
) {
	// let's wait until the connection is established
	match state_rx.blocking_recv() {
		Some(State::Connected) => {},
		Some(State::Disconnected) |
		None => return
	};


	let mut gather_stats_in = 0;
	let mut stats;
	let mut ticks = FrameTicks::new();

	// let's try to get some information
	loop {
		if gather_stats_in == 0 {
			stats = rt.block_on(peer_connection.get_stats());
			// for (key, stat) in stats.reports {
			// 	// let stat = match stat {
			// 	// 	StatsReportType::CandidatePair(stat) |
			// 	// 	StatsReportType::LocalCandidate(stat) |
			// 	// 	StatsReportType::RemoteCandidate(stat)
			// 	// }
			// 	if let StatsReportType::CandidatePair(stat) = stat {
			// 		eprintln!("{:?} {:?} {:?}", key, stat.available_incoming_bitrate, stat.available_outgoing_bitrate);
			// 	}
			// }
			gather_stats_in = GATHER_STATS_EVERY;
		} else {
			gather_stats_in -= 1;
		}

		// wait until we should send another frame
		let frames_skipped = ticks.tick();
		if frames_skipped > 0 {
			eprintln!("skipped frames {:?}", frames_skipped);
		}

		// check if the connection already closed
		match state_rx.try_recv() {
			Ok(State::Connected) => unreachable!(),
			Ok(State::Disconnected) |
			Err(mpsc::error::TryRecvError::Disconnected) => return,
			Err(mpsc::error::TryRecvError::Empty) => {}
		}

		let starts_sample_time = Instant::now();
		let mut sample = match camera.next_sample() {
			Ok(s) => s,
			Err(e) => {
				// should we close the track??
				eprintln!("camera closed {:?}", e);
				return
			}
		};
		eprintln!(
			"took {:?}ms to get the next sample",
			starts_sample_time.elapsed().as_millis()
		);

		sample.duration = FRAME_DURATION;

		if let Err(e) = rt.block_on(track.write_sample(&sample)) {
			eprintln!("could not write sample {:?}", e);
			return
		}
	}
}


const FRAME_DURATION: Duration = Duration::from_millis(1000 / 30);

struct FrameTicks {
	last_tick: Instant
}

impl FrameTicks {
	fn new() -> Self {
		Self {
			last_tick: Instant::now()
		}
	}

	/// wait's until the next tick
	/// returns the number of tick that already passed
	fn tick(&mut self) -> usize {
		let mut elapsed = self.last_tick.elapsed();
		if elapsed < FRAME_DURATION {
			thread::sleep(FRAME_DURATION - elapsed);
			self.last_tick = Instant::now();
			return 0
		}

		let ticks_passed = (
			elapsed.as_secs_f64() / FRAME_DURATION.as_secs_f64()
		).floor() as u32;
		elapsed -= FRAME_DURATION * ticks_passed;
		thread::sleep(FRAME_DURATION - elapsed);
		self.last_tick = Instant::now();

		ticks_passed as usize
	}
}