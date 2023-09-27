mod ntp_packet;

use crate::ntp_packet::{
    ExternalReferenceSource, LeapIndicator, NtpMessage, NtpTimestamp, ReferenceIdentifier,
    Stratum, VersionNumber,
};
use bytes::BytesMut;
use chrono::{TimeZone, Utc};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, trace, Level};

fn tracing() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_target(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing();

    // Specify the address to bind to
    let addr = "0.0.0.0:123".parse::<SocketAddr>()?;

    // Create a UDP socket and bind it to the specified address
    let socket = UdpSocket::bind(&addr).await?;
    info!("Listening on: {}", addr);

    // Create a buffer to store incoming data
    let mut buf = BytesMut::with_capacity(512);
    buf.resize(512, 0);

    loop {
        // Receive data into the buffer
        let (size, peer) = socket.recv_from(&mut buf).await?;

        let receive_timestamp = Utc::now();

        // Handle the received data
        let mut data = buf.split_to(size);
        // Resize the buffer to make sure data can continuously be received
        buf.resize(buf.len() + size, 0);

        trace!("Data: {:?}. Size: {}, buf size: {}", data, size, buf.len());

        match NtpMessage::try_from(&mut data) {
            Ok(packet) => {
                // Parse and process the UDP packet data here
                // For example, you can print the received data
                debug!("Received {} bytes from {}: {:?}", size, peer, data);

                // Some example alternate settings to mess around are defined below the value it
                // belongs to. To change it, comment the value out and uncomment the value below it.
                let response = NtpMessage::new_server_response(
                    LeapIndicator::NoWarning,
                    packet.vn,
                    Stratum::PrimaryReference,
                    4,
                    -6,
                    0,
                    0,
                    ReferenceIdentifier::Primary(Some(ExternalReferenceSource::GPS)),
                    NtpTimestamp(Utc::now()),
                    // NtpTimestamp(Utc::now() - Duration::hours(6) - Duration::seconds(3)),
                    Some(packet.transmit_timestamp),
                    NtpTimestamp(receive_timestamp),
                    // NtpTimestamp(receive_timestamp - Duration::hours(6)),
                    Some(NtpTimestamp(Utc::now())),
                    // Some(NtpTimestamp(Utc::now() - Duration::hours(6))),
                );

                // The code below can be used to create a response that doesn't abide by a server's
                // rules
                // let response = NtpMessage {
                //     li: LeapIndicator::NoWarning,
                //     vn: VersionNumber::One,
                //     mode: Mode::Reserved,
                //     stratum: Stratum::KissODeathMessage,
                //     poll_interval: 0,
                //     precision: 0,
                //     root_delay: 0,
                //     root_dispersion: 0,
                //     reference_identifier: Some(ReferenceIdentifier::Primary(Some(ExternalReferenceSource::GPS))),
                //     reference_timestamp: Some(NtpTimestamp(Utc::now())),
                //     originate_timestamp: Some(packet.transmit_timestamp),
                //     receive_timestamp: Some(NtpTimestamp(receive_timestamp)),
                //     transmit_timestamp: NtpTimestamp(Utc::now()),
                // };

                trace!("About to send: {response:?}");
                socket.send_to(&response.to_bytes(), peer).await?;
                trace!("Successfully sent response");
            }
            Err(err) => error!("Unable to parse packet: {err}"),
        };
    }
}
