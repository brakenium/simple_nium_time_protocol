use anyhow::bail;
use bytes::{Buf, BytesMut};
use chrono::{NaiveDateTime, TimeZone, Utc};
use std::net::Ipv4Addr;
use std::str;
use std::str::FromStr;
use strum::{AsRefStr, EnumString, FromRepr};
use thiserror::Error;
use tracing::trace;

#[derive(FromRepr, Copy, Clone, Debug)]
#[repr(u8)]
pub enum LeapIndicator {
    NoWarning = 0,
    LastMinuteHas61Seconds = 1,
    LastMinuteHas59Seconds = 2,
    AlarmConditionClockNotSynchronised = 3,
}

#[derive(FromRepr, Copy, Clone, Debug)]
#[repr(u8)]
pub enum VersionNumber {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

#[derive(FromRepr, Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum Mode {
    Reserved = 0,
    SymmetricActive = 1,
    SymmetricPassive = 2,
    Client = 3,
    Server = 4,
    Broadcast = 5,
    ReservedForNtpControlMessage = 6,
    ReservedForPrivateUse = 7,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Stratum {
    KissODeathMessage = 0,
    PrimaryReference = 1,
    SecondaryReference = 2,
    Reserved = 16,
}

impl From<u8> for Stratum {
    fn from(value: u8) -> Self {
        match value {
            0 => Stratum::KissODeathMessage,
            1 => Stratum::PrimaryReference,
            2..=15 => Stratum::SecondaryReference,
            _ => Stratum::Reserved,
            // _ => {
            //     println!("No stratum match found");
            //     Reserved
            // },
        }
    }
}

#[derive(EnumString, AsRefStr, Debug)]
#[repr(u32)]
pub enum ExternalReferenceSource {
    LOCL,
    CESM,
    RBDM,
    PPS,
    IRIG,
    ACTS,
    USNO,
    PTB,
    TDF,
    DCF,
    MSF,
    WWV,
    WWVB,
    WWVH,
    CHU,
    LORC,
    OMEG,
    GPS,
}

#[derive(EnumString, AsRefStr, Debug)]
pub enum KissODeathIdentifier {
    ACST,
    AUTH,
    AUTO,
    BCST,
    CRYP,
    DENY,
    DROP,
    RSTR,
    INIT,
    MCST,
    NKEY,
    RATE,
    RMOT,
    STEP,
}

#[derive(Debug)]
pub enum ReferenceIdentifier {
    KissODeath(KissODeathIdentifier),
    Primary(Option<ExternalReferenceSource>),
    IPv4Secondary(Ipv4Addr),
    IPv6AndOSISecondary(u32),
    UnknownIpVersion(u32),
    ReservedStratum(u32),
}


#[derive(Debug)]
pub struct NtpTimestamp(pub NaiveDateTime);

#[derive(Error, Debug)]
pub enum NtpTimestampError {
    #[error("Supplied timestamp is zero")]
    Zero,
    #[error("Supplied timestamp is invalid")]
    Invalid,
}

impl TryFrom<&mut BytesMut> for NtpTimestamp {
    type Error = NtpTimestampError;

    fn try_from(value: &mut BytesMut) -> Result<Self, Self::Error> {
        trace!("ntp_timestamp: {:?}", value);
        let seconds = value.split_to(4).get_u32() as i64;
        let fraction = value.split_to(4).get_u32();
        let nano_seconds = pad_int(fraction as isize, 9);

        trace!("Seconds: {}, fraction: {}, nanoseconds: {}", seconds, fraction, nano_seconds);

        if seconds == 0 && nano_seconds == 0 {
            return Err(NtpTimestampError::Zero);
        }

        // Might be wrong
        let seconds_unix_format = seconds - 2208988800;

        let timestamp = NaiveDateTime::from_timestamp_opt(seconds_unix_format, nano_seconds as u32);

        trace!(
            "Seconds_unix: {}, seconds_fraction: {} Timestamp: {:?}",
            seconds_unix_format,
            nano_seconds,
            timestamp
        );

        match timestamp {
            Some(ts) => Ok(NtpTimestamp(ts)),
            None => {
                Err(NtpTimestampError::Invalid)
            }
        }
    }
}

impl NtpTimestamp {
    fn to_bytes(&self) -> [u8; 8] {
        let timestamp: u32 = (self.0.timestamp() + 2208988800) as u32;
        let fraction = self.0.timestamp_subsec_nanos();

        trace!("Timestamp: {:?} Fraction: {:?}", timestamp, fraction);

        let mut timestamp_bytes = [0; 4];
        let mut fraction_bytes = [0; 4];

        timestamp_bytes.copy_from_slice(&timestamp.to_be_bytes());
        fraction_bytes.copy_from_slice(&fraction.to_be_bytes());

        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&timestamp_bytes);
        bytes[4..].copy_from_slice(&fraction_bytes[..4]);

        trace!(
            "Timestamp: {:?} Fraction: {:?} Bytes: {:?}",
            timestamp_bytes,
            fraction_bytes,
            bytes
        );

        bytes
    }
}

#[derive(Debug)]
pub struct NtpMessage {
    pub li: LeapIndicator,
    pub vn: VersionNumber,
    pub mode: Mode,
    pub stratum: Stratum,
    pub poll_interval: u8,
    pub precision: i8,
    pub root_delay: i32,
    pub root_dispersion: u32,
    pub reference_identifier: Option<ReferenceIdentifier>,
    pub reference_timestamp: Option<NtpTimestamp>,
    pub originate_timestamp: Option<NtpTimestamp>,
    pub receive_timestamp: Option<NtpTimestamp>,
    pub transmit_timestamp: NtpTimestamp,
}

#[derive(Debug)]
pub struct NtpServerResponse {
    pub leap_indicator: LeapIndicator,
    pub version_number: VersionNumber,
    pub stratum: Stratum,
    pub poll_interval: u8,
    pub precision: i8,
    pub root_delay: i32,
    pub root_dispersion: u32,
    pub reference_identifier: ReferenceIdentifier,
    pub reference_timestamp: NtpTimestamp,
    pub originate_timestamp: Option<NtpTimestamp>,
    pub receive_timestamp: NtpTimestamp,
    pub transmit_timestamp: Option<NtpTimestamp>,
}

fn pad_int(mut integer: isize, expected_digits: i32) -> isize {
    let digit_length = integer.to_string().len() as i32;
    let shifter = 10_f64.powi(expected_digits - digit_length);

    integer = ((integer as f64) * shifter) as isize;

    integer
}

impl NtpMessage {
    pub fn to_bytes(&self) -> [u8; 48] {
        let mut bytes = [0; 48];
        let flags = (self.li as u8) << 6 | (self.vn as u8) << 3 | (self.mode as u8);
        bytes[0] = flags;

        let stratum = self.stratum as u8;
        bytes[1] = stratum;

        let poll_interval = self.poll_interval;
        bytes[2] = poll_interval;

        let precision = self.precision;
        bytes[3] = precision as u8;

        let root_delay = self.root_delay;
        bytes[4..8].copy_from_slice(&root_delay.to_be_bytes());

        let root_dispersion = self.root_dispersion;
        bytes[8..12].copy_from_slice(&root_dispersion.to_be_bytes());

        match &self.reference_identifier {
            Some(ReferenceIdentifier::KissODeath(kod)) => {
                bytes[12..16].copy_from_slice(kod.as_ref().as_bytes())
            }
            Some(ReferenceIdentifier::Primary(Some(rid))) => {
                let reference_source = rid.as_ref();
                let diff = 4 - reference_source.len();
                bytes[12..16 - diff].copy_from_slice(reference_source.as_ref());
            }
            Some(ReferenceIdentifier::Primary(None)) => bytes[12..16].copy_from_slice(&[0; 4]),
            Some(ReferenceIdentifier::IPv4Secondary(ip)) => {
                bytes[12..16].copy_from_slice(&ip.octets())
            }
            Some(ReferenceIdentifier::IPv6AndOSISecondary(ip_osi)) => {
                bytes[12..16].copy_from_slice(&ip_osi.to_be_bytes())
            }
            Some(ReferenceIdentifier::UnknownIpVersion(ip)) => {
                bytes[12..16].copy_from_slice(&ip.to_be_bytes())
            }
            Some(ReferenceIdentifier::ReservedStratum(stratum)) => {
                bytes[12..16].copy_from_slice(&stratum.to_be_bytes())
            }
            None => bytes[12..16].copy_from_slice(&[0; 4]),
        };

        let reference_timestamp: &[u8; 8] = &match &self.reference_timestamp {
            Some(ts) => ts.to_bytes(),
            None => [0; 8],
        };
        bytes[16..24].copy_from_slice(reference_timestamp);

        let originate_timestamp: &[u8; 8] = &match &self.originate_timestamp {
            Some(ts) => ts.to_bytes(),
            None => [0; 8],
        };
        bytes[24..32].copy_from_slice(originate_timestamp);

        let receive_timestamp: &[u8; 8] = &match &self.receive_timestamp {
            Some(ts) => ts.to_bytes(),
            None => [0; 8],
        };
        bytes[32..40].copy_from_slice(receive_timestamp);

        let transmit_timestamp: &[u8; 8] = &self.transmit_timestamp.to_bytes();
        // let transmit_timestamp = &NtpTimestamp(Utc::now() - Duration::hours(6)).to_bytes();
        bytes[40..48].copy_from_slice(transmit_timestamp);

        trace!("Bytes: {:?}", bytes);

        bytes
    }

    pub fn new_server_response(res: NtpServerResponse) -> Self {
        NtpMessage {
            li: res.leap_indicator,
            vn: res.version_number,
            mode: Mode::Server,
            stratum: res.stratum,
            poll_interval: res.poll_interval,
            precision: res.precision,
            root_delay: res.root_delay,
            root_dispersion: res.root_dispersion,
            reference_identifier: Some(res.reference_identifier),
            reference_timestamp: Some(res.reference_timestamp),
            // reference_timestamp: None,
            originate_timestamp: res.originate_timestamp,
            receive_timestamp: Some(res.receive_timestamp),
            transmit_timestamp: match res.transmit_timestamp {
                Some(ts) => ts,
                None => NtpTimestamp(Utc::now().naive_utc()),
            },
        }
    }
}

impl TryFrom<&mut BytesMut> for NtpMessage {
    type Error = anyhow::Error;

    fn try_from(value: &mut BytesMut) -> Result<Self, Self::Error> {
        if value.len() < 48 {
            bail!("Packet is too small");
        }
        let flags = value.split_to(1).get_u8();
        let li = match LeapIndicator::from_repr(&flags >> 6) {
            Some(li) => li,
            None => bail!("Unable to parse LeapIndicator"),
        };
        let vn = match VersionNumber::from_repr((&flags & 0b0011_1000) >> 3) {
            Some(vn) => vn,
            None => bail!("Unable to parse VersionNumber"),
        };
        let mode = match Mode::from_repr(&flags & 0b0000_0111) {
            Some(mode) => mode,
            None => bail!("Unable to parse Mode"),
        };
        trace!("VersionNumber: {vn:?}");
        let stratum = Stratum::from(value.split_to(1).get_u8());
        trace!("Stratum: {stratum:?}");
        let poll_interval = value.split_to(1).get_u8();
        let precision = value.split_to(1).get_i8();
        let root_delay = value.split_to(4).get_i32();
        let root_dispersion = value.split_to(4).get_u32();
        let reference_identifier = {
            let mut slice = value.split_to(4);
            if mode == Mode::Client {
                None
            } else {
                Some(match stratum {
                    Stratum::KissODeathMessage => {
                        let as_string = str::from_utf8(slice.as_ref())?;
                        trace!(
                            "Reference identifier as utf8 string: {as_string:?}. Mode: {mode:?}"
                        );
                        let kod_identifier = match KissODeathIdentifier::from_str(as_string) {
                            Ok(rs) => rs,
                            Err(_) => bail!("Invalid Kiss-O-Death Identifier"),
                        };
                        trace!("Kiss-O-Death Identifier: {kod_identifier:?}");
                        ReferenceIdentifier::KissODeath(kod_identifier)
                    }
                    Stratum::PrimaryReference => {
                        let as_string = str::from_utf8(slice.as_ref())?;
                        trace!(
                            "Reference identifier as utf8 string: {as_string:?}. Mode: {mode:?}"
                        );
                        let reference_source = match ExternalReferenceSource::from_str(as_string) {
                            Ok(rs) => Some(rs),
                            Err(_) => None,
                        };
                        trace!("Reference source: {reference_source:?}");
                        ReferenceIdentifier::Primary(reference_source)
                    }
                    Stratum::SecondaryReference => {
                        ReferenceIdentifier::UnknownIpVersion(slice.get_u32())
                    }
                    Stratum::Reserved => ReferenceIdentifier::ReservedStratum(slice.get_u32()),
                })
            }
        };
        let reference_timestamp: Option<NtpTimestamp> =
            match NtpTimestamp::try_from(&mut value.split_to(8)) {
                Ok(ts) => Some(ts),
                Err(_) => None,
            };
        let originate_timestamp = match NtpTimestamp::try_from(&mut value.split_to(8)) {
            Ok(ts) => Some(ts),
            Err(_) => None,
        };
        let receive_timestamp = match NtpTimestamp::try_from(&mut value.split_to(8)) {
            Ok(ts) => Some(ts),
            Err(_) => None,
        };
        let transmit_timestamp = match NtpTimestamp::try_from(&mut value.split_to(8)) {
            Ok(ts) => ts,
            Err(_) => bail!("Unable to parse transmit timestamp"),
        };

        Ok(NtpMessage {
            li,
            vn,
            mode,
            stratum,
            poll_interval,
            precision,
            root_delay,
            root_dispersion,
            reference_identifier,
            reference_timestamp,
            originate_timestamp,
            receive_timestamp,
            transmit_timestamp,
        })
    }
}
