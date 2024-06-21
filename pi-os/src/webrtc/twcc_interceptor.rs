use std::sync::Arc;

use async_trait::async_trait;

use webrtc::interceptor::stream_info::StreamInfo;
use webrtc::interceptor::{
	Attributes, Error, Interceptor, InterceptorBuilder, RTCPReader, RTCPWriter,
	RTPReader, RTPWriter,
};

#[derive(Debug)]
pub struct TwccInterceptorBuilder;

impl InterceptorBuilder for TwccInterceptorBuilder {
	fn build(
		&self,
		id: &str,
	) -> Result<Arc<dyn Interceptor + Send + Sync>, Error> {
		Ok(Arc::new(TwccInterceptor))
	}
}

#[derive(Debug)]
pub struct TwccInterceptor;

impl TwccInterceptor {
	pub fn builder() -> TwccInterceptorBuilder {
		TwccInterceptorBuilder {}
	}
}

#[async_trait]
impl Interceptor for TwccInterceptor {
	async fn bind_rtcp_reader(
		&self,
		reader: Arc<dyn RTCPReader + Send + Sync>,
	) -> Arc<dyn RTCPReader + Send + Sync> {
		Arc::new(TwccInterceptorRtcpReader {
			parent_reader: reader,
		})
	}

	async fn bind_rtcp_writer(
		&self,
		writer: Arc<dyn RTCPWriter + Send + Sync>,
	) -> Arc<dyn RTCPWriter + Send + Sync> {
		writer
	}

	async fn bind_local_stream(
		&self,
		_info: &StreamInfo,
		writer: Arc<dyn RTPWriter + Send + Sync>,
	) -> Arc<dyn RTPWriter + Send + Sync> {
		eprintln!("bind local stream {:?}", _info);
		writer
	}

	async fn unbind_local_stream(&self, _info: &StreamInfo) {}

	async fn bind_remote_stream(
		&self,
		_info: &StreamInfo,
		reader: Arc<dyn RTPReader + Send + Sync>,
	) -> Arc<dyn RTPReader + Send + Sync> {
		eprintln!("bind remote stream {:?}", _info);
		reader
	}

	async fn unbind_remote_stream(&self, _info: &StreamInfo) {}

	async fn close(&self) -> Result<(), Error> {
		Ok(())
	}
}

pub struct TwccInterceptorRtcpReader {
	parent_reader: Arc<dyn RTCPReader + Send + Sync>,
}

#[async_trait]
impl RTCPReader for TwccInterceptorRtcpReader {
	async fn read(
		&self,
		buf: &mut [u8],
		a: &Attributes,
	) -> Result<(usize, Attributes), Error> {
		let (n, attr) = self.parent_reader.read(buf, a).await?;

		let mut b = &buf[..n];
		let packet = webrtc::rtcp::packet::unmarshal(&mut b)?;

		// eprintln!("rtcp packet {:?}", packet);

		Ok((n, attr))
	}
}

pub struct TwccInterceptorRtpReader {
	parent_reader: Arc<dyn RTPReader + Send + Sync>,
}

#[async_trait]
impl RTPReader for ReceiverStream {
	/// read a rtp packet
	async fn read(
		&self,
		buf: &mut [u8],
		a: &Attributes,
	) -> Result<(usize, Attributes)> {
		let (n, attr) = self.parent_reader.read(buf, a).await?;

		let mut b = &buf[..n];
		let packet = webrtc::rtp::packet::Packet::unmarshal(&mut b)?;

		eprintln!("rtp packet {:?}", packet);

		Ok((n, attr))
	}
}
