use std::convert::TryInto;

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct AckBitfield(u32);

impl AckBitfield {
    pub(crate) fn from_bytes(buf: &[u8]) -> Self {
        let arr: [u8; std::mem::size_of::<Self>()] =
            buf.try_into().expect("ackbitfield from expectes 32 bits");
        Self(u32::from_be_bytes(arr))
    }

    pub(crate) fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub(crate) fn set(&mut self, index: u8) {
        assert!(index <= 31);
        self.0 |= 1 << index;
    }

    pub(crate) fn get(&self, index: u8) -> bool {
        assert!(index <= 31);

        let mask = 1u32 << index;
        let and = self.0 & mask;
        and == mask
    }

    pub(crate) fn shift(&mut self, shifts: u32) {
        self.0 >>= 1;
        self.set(31);

        self.0 = self
            .0
            .checked_shr(shifts.checked_sub(1).expect("expected shifts to be >= 1"))
            .unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::connection::*;
    use crate::network::*;
    use proptest::collection;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn header_serde((seq, ack, ack_bitfield) in (any::<u32>(), any::<u32>(), any::<u32>())) {
            let header = {
                let mut buffer = PacketHeader(vec![0u8; 4*4]);
                buffer.set(SeqNumber::from(seq), SeqNumber::from(ack), AckBitfield(ack_bitfield));
                buffer
            };

            assert_eq!(header.seq(), SeqNumber::from(seq));
            assert_eq!(header.ack(), SeqNumber::from(ack));
            assert_eq!(header.ack_bitfield(), AckBitfield(ack_bitfield));
        }
    }

    #[derive(Clone, Debug)]
    enum AckBitfieldAction {
        Set(u8),
        Get(u8),
        Shift(u32),
    }

    fn ack_bitfield_action() -> BoxedStrategy<AckBitfieldAction> {
        prop_oneof![
            any::<u8>().prop_map(move |a| AckBitfieldAction::Set(a % 32)),
            any::<u8>().prop_map(move |a| AckBitfieldAction::Get(a % 32)),
            any::<u8>().prop_map(move |a| AckBitfieldAction::Shift((a % 255 + 1) as u32))
        ]
        .boxed()
    }

    proptest! {
        #[test]
        fn ack_bitfield(actions in collection::vec(ack_bitfield_action(), 10..20)) {
            let mut map = std::collections::HashMap::<u8, bool>::new();
            let mut bitfield = AckBitfield::default();

            for action in actions {
                match action {
                    AckBitfieldAction::Set(idx) => {
                        map.entry(idx).and_modify(|e| *e = true).or_insert(true);
                        bitfield.set(idx);
                    },
                    AckBitfieldAction::Get(idx) => {
                        assert_eq!(bitfield.get(idx), *map.get(&idx).unwrap_or(&false));
                    },
                    AckBitfieldAction::Shift(shifts) => {
                        bitfield.shift(shifts);

                        fn shift(map: &mut std::collections::HashMap<u8, bool>) {
                            let mut keys = map.keys().cloned().filter_map(|k| k.checked_sub(1)).collect::<Vec<_>>();
                            keys.sort();

                            map.entry(0).and_modify(|e| *e = false);

                            for k in keys {
                                let next = map.get(&(k + 1)).cloned().unwrap_or(false);
                                map.entry(k).and_modify(|e| *e = next).or_insert(next);
                                map.entry(k + 1).and_modify(|e| *e = false).or_insert(false);
                            }
                        }

                        shift(&mut map);
                        map.entry(31).and_modify(|e| *e = true).or_insert(true);

                        for _ in 0..shifts - 1 {
                            shift(&mut map);
                        }

                        if shifts > 32 {
                            for i in 0..32 {
                                assert!(!bitfield.get(i));
                            }
                        }
                        else {
                            assert!(bitfield.get(31u8 + 1 - shifts as u8));
                        }
                        // 1 1 1 1 -> (0 1 1 1 -> 1 1 1 1) -> 0 1 1 1 -> 0 0 1 1 -> 0 0 0 1 -> 0 0 0 0
                    },
                }
            }

        }
    }
}
