use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use std::error::Error as StdError;

use tokio::sync::Notify;
use tokio::time::Duration;

use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::io::h264_reader::H264Reader;
use webrtc::media::io::ogg_reader::OggReader;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::{
	TrackLocalStaticSample
};
use webrtc::track::track_local::TrackLocal;
use webrtc::Error;

#[tokio::main]
async fn main() {


	let mut m = MediaEngine::default();

	m.register_default_codecs().unwrap();

	let mut registry = Registry::new();

	registry = register_default_interceptors(registry, &mut m).unwrap();

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

	let peer_connection = Arc::new(
		api.new_peer_connection(config).await.unwrap()
	);

	let notify_tx = Arc::new(Notify::new());
	let notify_video = notify_tx.clone();

	let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
	let video_done_tx = done_tx.clone();

	// video
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
	).await.unwrap();

	// Read incoming RTCP packets
	// Before these packets are returned they are processed by interceptors.
	// For things like NACK this needs to be called.
	tokio::spawn(async move {
		let mut rtcp_buf = vec![0u8; 1500];
		while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
	});

	tokio::spawn(async move {
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

		let _ = video_done_tx.try_send(());
	});


	// set the handler for ICE connection
	peer_connection.on_ice_connection_state_change(Box::new(
		move |connection_state: RTCIceConnectionState| {
			eprintln!("connection state changed {:?}", connection_state);
			if connection_state == RTCIceConnectionState::Connected {
				notify_tx.notify_waiters();
			}

			Box::pin(async {})
		}
	));

	peer_connection.on_peer_connection_state_change(
		Box::new(move |s: RTCPeerConnectionState| {
			eprintln!("state change {:?}", s);

			if s == RTCPeerConnectionState::Failed {
				eprintln!("peer conection failed");
				let _ = done_tx.try_send(());
			}

			Box::pin(async {})
		})
	);

	eprintln!("read line");

	// wait for the offer to be pasted
	let line = fs::read_to_string("./session.txt").unwrap();
	let desc_data = decode(line.as_str()).unwrap();
	eprintln!("desc {:?}", desc_data);
	let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)
		.unwrap();

	peer_connection.set_remote_description(offer).await.unwrap();

	let answer = peer_connection.create_answer(None).await.unwrap();

	let mut gather_complete = peer_connection.gathering_complete_promise().await;

	peer_connection.set_local_description(answer).await.unwrap();

	let _ = gather_complete.recv().await;

	if let Some(local_desc) = peer_connection.local_description().await {
		let json_str = serde_json::to_string(&local_desc).unwrap();
		let b64 = encode(&json_str);
		eprintln!("b64 {:?}", b64);
	} else {
		eprintln!("generate local_description failed!");
	}

	eprintln!("precc ctrl-c to stop");
	tokio::select! {
		_ = done_rx.recv() => {
			eprintln!("received done signal!");
		},
		_ = tokio::signal::ctrl_c() => {
			println!("ctrl + c");
		}
	}

	peer_connection.close().await.unwrap();
}

pub fn decode(s: &str) -> Result<String, Box<dyn StdError>> {
	let b = base64::decode(s)?;
	let s = String::from_utf8(b)?;
	Ok(s)
}

pub fn encode(b: &str) -> String {
	base64::encode(b)
}