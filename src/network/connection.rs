use super::ack_bitfield::AckBitfield;
use super::{NetworkError, Packet, PacketHeader, ProtocolId, Result, PROTOCOL_ID};
use log::debug;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Duration;

const PACKET_TIMEOUT: Duration = packet_timeout();

pub const fn timeout() -> Duration {
    Duration::from_secs(10)
}

pub(super) const fn header_size() -> usize {
    use std::mem::*;
    size_of::<ProtocolId>() + 2usize * size_of::<SeqNumber>() + size_of::<AckBitfield>()
}

pub(super) const fn packet_timeout() -> Duration {
    Duration::from_secs(1)
}

pub struct Connection {
    timeout_timer: Duration,
    seq: SeqNumber,
    ack: SeqNumber,
    ack_bitfield: AckBitfield,
    timeouts: HashMap<SeqNumber, Duration>,
}

#[derive(Default, Debug, Eq, Hash, PartialEq, Copy, Clone, PartialOrd)]
#[repr(transparent)]
pub struct SeqNumber(u32);

impl std::fmt::Display for SeqNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub type NonDeliveredPackets = Vec<SeqNumber>;

impl Connection {
    pub fn new() -> Self {
        Connection {
            timeout_timer: Duration::from_secs(10),
            seq: Default::default(),
            ack: Default::default(),
            ack_bitfield: Default::default(),
            timeouts: Default::default(),
        }
    }

    pub fn check<B: AsRef<[u8]>>(&mut self, packet: &Packet<B>) -> Result<()> {
        let header = packet.header();
        if header.protocol_id() != PROTOCOL_ID {
            return Err(NetworkError::InvalidProtocolId);
        }

        if let Some(shifts) = header.seq().distance_to(&self.ack) {
            self.ack = header.seq();
            self.ack_bitfield.shift(shifts + 1);
        } else {
            // this unwrap should be safe in this branch
            // because header.seq() < self.seq
            let diff = self.seq.distance_to(&header.seq()).unwrap();

            // if this is None, it means the packet is too old to send the ack
            // this means we will just let it timeout on the other side
            if let Some(bounded_diff) = 32u32.checked_sub(diff) {
                self.ack_bitfield.set(bounded_diff as u8);
            }
        }

        self.timeouts.remove(&header.ack());

        let upper: u32 = header.ack().into();
        let bitset = header.ack_bitfield();

        let lower = upper.checked_sub(32).unwrap_or(0);

        for (index, seq) in (lower..upper).into_iter().enumerate() {
            if bitset.get(index as u8) {
                self.timeouts.remove(&SeqNumber(seq));
            }
        }

        self.timeout_timer = timeout();

        Ok(())
    }

    pub fn fill_header<B: AsRef<[u8]> + AsMut<[u8]>>(&mut self, mut header: PacketHeader<B>) {
        debug!("connection, fill_header");
        header.set(self.seq, self.ack, self.ack_bitfield);

        self.timeouts.insert(self.seq, PACKET_TIMEOUT);
        self.seq = self.seq.next();
    }

    pub fn update(&mut self, dt: std::time::Duration) -> NonDeliveredPackets {
        let mut timed_out = vec![];

        if let Some(timer) = self.timeout_timer.checked_sub(dt) {
            self.timeout_timer = timer;

            for (seq, duration) in self.timeouts.iter_mut() {
                let new_duration = duration.checked_sub(dt);
                if let Some(new_duration) = new_duration {
                    *duration = new_duration;
                } else {
                    timed_out.push(*seq)
                }
            }

            for seq in &timed_out {
                self.timeouts.remove(seq);
            }
        }
        timed_out
    }
}

impl SeqNumber {
    pub(super) fn next(&self) -> Self {
        SeqNumber(self.0.wrapping_add(1))
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Self {
        let bytes: [u8; 4] = bytes.try_into().expect("seq number should be 4 bytes");

        SeqNumber(u32::from_be_bytes(bytes))
    }

    pub(super) fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub(super) fn distance_to(&self, rhs: &Self) -> Option<u32> {
        let abs_dist = match self.0.overflowing_sub(rhs.0) {
            (dist, false) => dist,
            (_dist, true) => rhs.0 - self.0,
        };

        if abs_dist > (u32::MAX / 2) {
            if rhs.0 < self.0 {
                Some(rhs.0.wrapping_sub(self.0))
            } else {
                None
            }
        } else {
            self.0.checked_sub(rhs.0)
        }
    }
}

impl Into<u32> for SeqNumber {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for SeqNumber {
    fn from(n: u32) -> SeqNumber {
        SeqNumber(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection;
    use proptest::prelude::*;

    #[test]
    fn test_connection_one_roundtrip() {
        let mut client_connection = Connection::new();
        let mut server_connection = Connection::new();

        let mut msg1 = Packet::new_boxed(0);
        client_connection.fill_header(msg1.header_mut());
        server_connection.check(&msg1).unwrap();

        let mut msg2 = Packet::new_boxed(0);
        server_connection.fill_header(msg2.header_mut());
        client_connection.check(&msg2).unwrap();

        {
            let (_client_connection, timed_out) = client_connection
                .update(PACKET_TIMEOUT + Duration::from_secs(1))
                .unwrap();
            assert!(timed_out.is_empty());
        }
        {
            let (_conn, timed_out) = server_connection
                .update(PACKET_TIMEOUT + Duration::from_secs(1))
                .unwrap();
            assert_eq!(timed_out.len(), 1);
        }
    }

    #[test]
    fn test_connection_skip_one() {
        let mut client_connection = Connection::new();
        let mut server_connection = Connection::new();

        let mut msg1 = Packet::new_boxed(0);
        client_connection.fill_header(msg1.header_mut());
        server_connection.check(&msg1).unwrap();

        let mut msg2 = Packet::new_boxed(0);
        server_connection.fill_header(msg2.header_mut());

        let mut msg3 = Packet::new_boxed(0);
        server_connection.fill_header(msg3.header_mut());
        client_connection.check(&msg3).unwrap();

        {
            let (_client_connection, timed_out) = client_connection
                .update(PACKET_TIMEOUT + Duration::from_secs(1))
                .unwrap();
            assert!(timed_out.is_empty());
        }
        {
            let (_conn, timed_out) = server_connection
                .update(PACKET_TIMEOUT + Duration::from_secs(1))
                .unwrap();
            assert_eq!(timed_out.len(), 2);
        }
    }

    #[test]
    fn test_connection_timed_out() {
        let connection = Connection::new();

        {
            let new_conn = connection.update(timeout() + Duration::from_secs(11));
            assert!(new_conn.is_none());
        }
    }

    #[test]
    fn test_seq_number_distance() {
        let zero = SeqNumber(0);
        let high = SeqNumber(u32::MAX - 3);

        assert_eq!(zero.distance_to(&high), None);
        assert_eq!(high.distance_to(&zero), Some(4));
    }
}
