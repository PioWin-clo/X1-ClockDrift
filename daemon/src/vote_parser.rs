use crate::rpc_client::TxView;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub const VOTE_PROGRAM_ID: &str = "Vote111111111111111111111111111111111111111";

const TAG_VOTE: u32 = 0;
const TAG_VOTE_SWITCH: u32 = 6;
const TAG_UPDATE_VOTE_STATE: u32 = 7;
const TAG_UPDATE_VOTE_STATE_SWITCH: u32 = 8;
const TAG_COMPACT_UPDATE_VOTE_STATE: u32 = 9;
const TAG_COMPACT_UPDATE_VOTE_STATE_SWITCH: u32 = 10;
const TAG_TOWER_SYNC: u32 = 14;
const TAG_TOWER_SYNC_SWITCH: u32 = 15;

#[derive(Debug, Clone)]
pub struct ParsedVote {
    pub validator: String,
    pub slot_voted: u64,
    pub ts_chain: Option<i64>,
}

/// Extract every vote-program instruction's (validator, slot_voted, timestamp)
/// from a transaction. Validator = vote account pubkey (instruction's first account).
pub fn extract_votes_from_tx(tx: &TxView) -> Vec<ParsedVote> {
    let mut out = Vec::new();
    for ix in &tx.instructions {
        let program_id = match tx.account_keys.get(ix.program_id_index) {
            Some(s) => s,
            None => continue,
        };
        if program_id != VOTE_PROGRAM_ID {
            continue;
        }
        let validator_idx = match ix.accounts.first() {
            Some(i) => *i,
            None => continue,
        };
        let validator = match tx.account_keys.get(validator_idx) {
            Some(s) => s.clone(),
            None => continue,
        };
        if let Some((slot_voted, ts_chain)) = parse_vote_data(&ix.data_bytes) {
            out.push(ParsedVote {
                validator,
                slot_voted,
                ts_chain,
            });
        }
    }
    out
}

/// Returns (slot_voted, optional timestamp). None if not a recognized vote variant
/// or bytes are malformed.
pub fn parse_vote_data(data: &[u8]) -> Option<(u64, Option<i64>)> {
    if data.len() < 4 {
        return None;
    }
    let mut cur = Cursor::new(data);
    let tag = cur.read_u32::<LittleEndian>().ok()?;

    match tag {
        TAG_VOTE => parse_vote_legacy(&mut cur),
        TAG_VOTE_SWITCH => parse_vote_legacy(&mut cur),
        TAG_UPDATE_VOTE_STATE | TAG_UPDATE_VOTE_STATE_SWITCH => parse_vote_state_update(&mut cur),
        TAG_COMPACT_UPDATE_VOTE_STATE | TAG_COMPACT_UPDATE_VOTE_STATE_SWITCH => {
            parse_compact_vote_state_update(&mut cur)
        }
        TAG_TOWER_SYNC | TAG_TOWER_SYNC_SWITCH => parse_tower_sync(&mut cur),
        other => {
            tracing::debug!(tag = other, "unknown vote instruction variant");
            None
        }
    }
}

/// Vote { slots: Vec<u64>, hash: [u8;32], timestamp: Option<i64> }
fn parse_vote_legacy(cur: &mut Cursor<&[u8]>) -> Option<(u64, Option<i64>)> {
    let n = cur.read_u64::<LittleEndian>().ok()?;
    if n == 0 || n > 1024 {
        return None;
    }
    let mut max_slot = 0u64;
    for _ in 0..n {
        let s = cur.read_u64::<LittleEndian>().ok()?;
        if s > max_slot {
            max_slot = s;
        }
    }
    skip_bytes(cur, 32)?;
    let ts = read_option_i64(cur)?;
    Some((max_slot, ts))
}

/// VoteStateUpdate {
///   lockouts: VecDeque<Lockout { slot: u64, confirmation_count: u32 }>,
///   root: Option<u64>,
///   hash: [u8;32],
///   timestamp: Option<i64>,
/// }
fn parse_vote_state_update(cur: &mut Cursor<&[u8]>) -> Option<(u64, Option<i64>)> {
    let n = cur.read_u64::<LittleEndian>().ok()?;
    if n == 0 || n > 1024 {
        return None;
    }
    let mut max_slot = 0u64;
    for _ in 0..n {
        let s = cur.read_u64::<LittleEndian>().ok()?;
        let _conf = cur.read_u32::<LittleEndian>().ok()?;
        if s > max_slot {
            max_slot = s;
        }
    }
    let _root = read_option_u64(cur)?;
    skip_bytes(cur, 32)?;
    let ts = read_option_i64(cur)?;
    Some((max_slot, ts))
}

/// TowerSync {
///   lockouts: VecDeque<Lockout>,
///   root: Option<u64>,
///   hash: [u8;32],
///   timestamp: Option<i64>,
///   block_id: [u8;32],
/// }
fn parse_tower_sync(cur: &mut Cursor<&[u8]>) -> Option<(u64, Option<i64>)> {
    let n = cur.read_u64::<LittleEndian>().ok()?;
    if n == 0 || n > 1024 {
        return None;
    }
    let mut max_slot = 0u64;
    for _ in 0..n {
        let s = cur.read_u64::<LittleEndian>().ok()?;
        let _conf = cur.read_u32::<LittleEndian>().ok()?;
        if s > max_slot {
            max_slot = s;
        }
    }
    let _root = read_option_u64(cur)?;
    skip_bytes(cur, 32)?;
    let ts = read_option_i64(cur)?;
    Some((max_slot, ts))
}

/// CompactVoteStateUpdate uses short-vec encoding for lockouts and varint offsets.
/// Layout:
///   root: u64 (sentinel u64::MAX if no root)
///   lockouts: short-vec of (offset: varint, confirmation_count: u8)
///     where offset is added to running slot (initialized at root, or 0 if sentinel)
///   hash: [u8;32]
///   timestamp: Option<i64>
fn parse_compact_vote_state_update(cur: &mut Cursor<&[u8]>) -> Option<(u64, Option<i64>)> {
    let root = cur.read_u64::<LittleEndian>().ok()?;
    let mut running = if root == u64::MAX { 0u64 } else { root };

    let n = read_short_u16(cur)? as u64;
    if n > 1024 {
        return None;
    }
    let mut max_slot = running;
    for _ in 0..n {
        let offset = read_varint_u64(cur)?;
        running = running.checked_add(offset)?;
        let _conf = read_u8(cur)?;
        if running > max_slot {
            max_slot = running;
        }
    }
    if max_slot == 0 {
        return None;
    }
    skip_bytes(cur, 32)?;
    let ts = read_option_i64(cur)?;
    Some((max_slot, ts))
}

fn skip_bytes(cur: &mut Cursor<&[u8]>, n: usize) -> Option<()> {
    let mut buf = vec![0u8; n];
    cur.read_exact(&mut buf).ok()?;
    Some(())
}

fn read_option_i64(cur: &mut Cursor<&[u8]>) -> Option<Option<i64>> {
    let tag = read_u8(cur)?;
    match tag {
        0 => Some(None),
        1 => {
            let v = cur.read_i64::<LittleEndian>().ok()?;
            Some(Some(v))
        }
        _ => None,
    }
}

fn read_option_u64(cur: &mut Cursor<&[u8]>) -> Option<Option<u64>> {
    let tag = read_u8(cur)?;
    match tag {
        0 => Some(None),
        1 => {
            let v = cur.read_u64::<LittleEndian>().ok()?;
            Some(Some(v))
        }
        _ => None,
    }
}

fn read_u8(cur: &mut Cursor<&[u8]>) -> Option<u8> {
    let mut b = [0u8; 1];
    cur.read_exact(&mut b).ok()?;
    Some(b[0])
}

/// Solana short-vec u16 length: 1-3 bytes, 7 data bits per byte, top bit = continuation.
fn read_short_u16(cur: &mut Cursor<&[u8]>) -> Option<u16> {
    let mut value: u32 = 0;
    for shift in 0..3 {
        let b = read_u8(cur)?;
        let data = (b & 0x7f) as u32;
        value |= data << (shift * 7);
        if (b & 0x80) == 0 {
            if value > u16::MAX as u32 {
                return None;
            }
            return Some(value as u16);
        }
    }
    None
}

/// LEB128 unsigned u64.
fn read_varint_u64(cur: &mut Cursor<&[u8]>) -> Option<u64> {
    let mut value: u64 = 0;
    for shift in 0..10 {
        let b = read_u8(cur)?;
        let data = (b & 0x7f) as u64;
        value |= data.checked_shl(shift * 7)?;
        if (b & 0x80) == 0 {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_u32_le(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn write_u64_le(v: &mut Vec<u8>, x: u64) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn write_i64_le(v: &mut Vec<u8>, x: i64) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    #[test]
    fn parses_legacy_vote_with_timestamp() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_VOTE);
        write_u64_le(&mut data, 3);
        write_u64_le(&mut data, 100);
        write_u64_le(&mut data, 101);
        write_u64_le(&mut data, 102);
        data.extend_from_slice(&[0u8; 32]);
        data.push(1);
        write_i64_le(&mut data, 1_700_000_000);

        let (slot, ts) = parse_vote_data(&data).unwrap();
        assert_eq!(slot, 102);
        assert_eq!(ts, Some(1_700_000_000));
    }

    #[test]
    fn parses_legacy_vote_no_timestamp() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_VOTE);
        write_u64_le(&mut data, 1);
        write_u64_le(&mut data, 50);
        data.extend_from_slice(&[0u8; 32]);
        data.push(0);

        let (slot, ts) = parse_vote_data(&data).unwrap();
        assert_eq!(slot, 50);
        assert_eq!(ts, None);
    }

    #[test]
    fn parses_vote_state_update() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_UPDATE_VOTE_STATE);
        write_u64_le(&mut data, 2);
        write_u64_le(&mut data, 200);
        data.extend_from_slice(&31u32.to_le_bytes());
        write_u64_le(&mut data, 201);
        data.extend_from_slice(&30u32.to_le_bytes());
        data.push(1);
        write_u64_le(&mut data, 199);
        data.extend_from_slice(&[0u8; 32]);
        data.push(1);
        write_i64_le(&mut data, 1_700_000_001);

        let (slot, ts) = parse_vote_data(&data).unwrap();
        assert_eq!(slot, 201);
        assert_eq!(ts, Some(1_700_000_001));
    }

    #[test]
    fn parses_tower_sync() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_TOWER_SYNC);
        write_u64_le(&mut data, 1);
        write_u64_le(&mut data, 555);
        data.extend_from_slice(&5u32.to_le_bytes());
        data.push(0);
        data.extend_from_slice(&[0u8; 32]);
        data.push(1);
        write_i64_le(&mut data, 1_700_000_002);

        let (slot, ts) = parse_vote_data(&data).unwrap();
        assert_eq!(slot, 555);
        assert_eq!(ts, Some(1_700_000_002));
    }

    #[test]
    fn parses_compact_vote_state_update() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_COMPACT_UPDATE_VOTE_STATE);
        write_u64_le(&mut data, 1000);
        data.push(2);
        data.push(5);
        data.push(31);
        data.push(3);
        data.push(30);
        data.extend_from_slice(&[0u8; 32]);
        data.push(0);

        let (slot, ts) = parse_vote_data(&data).unwrap();
        assert_eq!(slot, 1008);
        assert_eq!(ts, None);
    }

    #[test]
    fn unknown_discriminant_returns_none() {
        let mut data = Vec::new();
        write_u32_le(&mut data, 99);
        data.extend_from_slice(&[0u8; 64]);
        assert!(parse_vote_data(&data).is_none());
    }

    #[test]
    fn truncated_data_returns_none() {
        let mut data = Vec::new();
        write_u32_le(&mut data, TAG_VOTE);
        write_u64_le(&mut data, 5);
        write_u64_le(&mut data, 1);
        assert!(parse_vote_data(&data).is_none());
    }

    #[test]
    fn varint_roundtrip() {
        for &v in &[0u64, 1, 127, 128, 255, 16383, 16384, 1_234_567, u32::MAX as u64] {
            let mut bytes = Vec::new();
            let mut x = v;
            loop {
                let mut b = (x & 0x7f) as u8;
                x >>= 7;
                if x != 0 {
                    b |= 0x80;
                }
                bytes.push(b);
                if x == 0 {
                    break;
                }
            }
            let mut cur = Cursor::new(bytes.as_slice());
            let got = read_varint_u64(&mut cur).unwrap();
            assert_eq!(got, v);
        }
    }

    #[test]
    fn shortvec_roundtrip() {
        for &v in &[0u16, 1, 127, 128, 255, 16383, 16384, 32768, u16::MAX] {
            let mut bytes = Vec::new();
            let mut x = v as u32;
            loop {
                let mut b = (x & 0x7f) as u8;
                x >>= 7;
                if x != 0 {
                    b |= 0x80;
                }
                bytes.push(b);
                if x == 0 {
                    break;
                }
            }
            let mut cur = Cursor::new(bytes.as_slice());
            let got = read_short_u16(&mut cur).unwrap();
            assert_eq!(got, v);
        }
    }
}
