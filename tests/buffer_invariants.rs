use bytes::{Buf, BufMut, Bytes, BytesMut};
use zerocopy::{
    network_endian::{U16, U32},
    FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned,
};
use zerocopy_buf::{ZeroCopyBuf, ZeroCopyBufMut, ZeroCopyReadBuf};

#[derive(FromBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
struct Packet {
    head: U16,
    body: [U16],
}

#[derive(FromBytes, Debug, PartialEq)]
#[repr(C, align(64))]
struct Aligned {
    bytes: [u8; 64],
}

#[test]
fn fragmented_owned_reads_match_contiguous_at_every_split_and_offset() {
    let data: Vec<u8> = (0..96).map(|n| n as u8).collect();
    let offsets = if cfg!(miri) { 2 } else { 8 };
    let lengths: Vec<usize> = if cfg!(miri) {
        vec![0, 1, 63, 64, 65, 72]
    } else {
        (0..=72).collect()
    };
    for offset in 0..offsets {
        for &len in &lengths {
            let input = &data[offset..offset + len];
            let mut splits: Vec<usize> = if cfg!(miri) {
                vec![0, len.min(1), len.min(31), len / 2, len]
            } else {
                (0..=len).collect()
            };
            splits.sort_unstable();
            splits.dedup();
            for split in splits {
                let mut fragmented = input[..split].chain(&input[split..]);
                let before = fragmented.remaining();
                let result = fragmented.try_read::<Aligned>();
                if len < 64 {
                    assert!(result.is_err());
                    assert_eq!(fragmented.remaining(), before);
                    assert_eq!(fragmented.copy_to_bytes(before).as_ref(), input);
                } else {
                    assert_eq!(result.unwrap().bytes, input[..64]);
                    assert_eq!(fragmented.remaining(), len - 64);
                    assert_eq!(fragmented.copy_to_bytes(len - 64).as_ref(), &input[64..]);
                }
            }
        }
    }
}

#[test]
fn zero_sized_owned_reads_do_not_advance_any_buf() {
    let mut empty = (&b""[..]).chain(&b""[..]);
    assert_eq!(empty.try_read::<()>(), Ok(()));
    let mut data = (&b"ab"[..]).chain(&b"cd"[..]);
    assert_eq!(data.try_read::<()>(), Ok(()));
    assert_eq!(data.copy_to_bytes(4).as_ref(), b"abcd");
}

macro_rules! all_backings {
    ($input:expr, $data:ident, $body:block) => {{
        let input: &[u8] = $input;
        {
            let mut $data: &[u8] = input;
            $body
        }
        {
            let mut $data = Bytes::copy_from_slice(input);
            $body
        }
        {
            let mut $data = BytesMut::from(input);
            $body
        }
    }};
}

#[test]
fn explicit_dst_counts_and_overflows_preserve_input_on_failure() {
    let input: Vec<u8> = (0..32).collect();
    for len in 0..=32 {
        for count in [0, 1, 2, 7, 15, 16, 17, usize::MAX / 2, usize::MAX] {
            all_backings!(&input[..len], data, {
                let before = data.chunk().to_vec();
                let result = data.try_get_elems::<Packet>(count);
                if count <= 15 && 2 + count * 2 <= len {
                    let view = result.unwrap();
                    assert_eq!(view.body.len(), count);
                    assert_eq!(view.head.get(), 1);
                    assert_eq!(data.chunk(), &before[2 + count * 2..]);
                } else {
                    assert!(result.is_err());
                    assert_eq!(data.chunk(), before);
                }
            });
        }
    }
}

#[test]
fn inferred_dst_prefixes_and_peeks_agree_and_preserve_remainders() {
    let input: Vec<u8> = (0..32).collect();
    for offset in 0..4 {
        for len in 0..=28 {
            all_backings!(&input[offset..offset + len], data, {
                let before = data.chunk().to_vec();
                let peek_len = match data.try_peek_prefix::<Packet>() {
                    Ok(view) => Some(view.body.len()),
                    Err(_) => None,
                };
                assert_eq!(data.chunk(), before);
                match data.try_get_prefix::<Packet>() {
                    Ok(view) => {
                        let count = (len - 2) / 2;
                        assert_eq!(view.body.len(), count);
                        assert_eq!(Some(count), peek_len);
                        assert_eq!(data.chunk(), &before[2 + count * 2..]);
                    }
                    Err(_) => {
                        assert!(len < 2);
                        assert_eq!(peek_len, None);
                        assert_eq!(data.chunk(), before);
                    }
                }
            });
        }
    }
}

#[test]
fn fixed_reads_and_explicit_peeks_work_at_unaligned_byte_offsets() {
    let input: Vec<u8> = (0..24).collect();
    for offset in 0..8 {
        for len in 0..=12 {
            all_backings!(&input[offset..offset + len], data, {
                let before = data.chunk().to_vec();
                assert_eq!(data.try_peek::<U32>().is_ok(), len >= 4);
                assert_eq!(data.try_peek_elems::<[U16]>(2).is_ok(), len >= 4);
                assert_eq!(data.chunk(), before);
                let result = data.try_get::<U32>();
                if len >= 4 {
                    let expected = u32::from_be_bytes(before[..4].try_into().unwrap());
                    assert_eq!(result.unwrap().get(), expected);
                    assert_eq!(data.chunk(), &before[4..]);
                } else {
                    assert!(result.is_err());
                    assert_eq!(data.chunk(), before);
                }
            });
        }
    }
}

#[test]
fn mutable_prefix_owns_disjoint_storage_from_suffix() {
    let mut data = BytesMut::from(&b"abcdefgh"[..]);
    let mut prefix = data.try_get_elems::<[u8]>(4).unwrap();
    prefix[0] = b'X';
    data[0] = b'Y';
    data.reserve(1024);
    assert_eq!(&*prefix, b"Xbcd");
    assert_eq!(&data[..], b"Yfgh");
    let mut slice = &b"\0\0\0\0"[..];
    assert!(slice.try_get::<U32>().is_ok());
}

#[derive(IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
struct Record {
    bytes: [u8; 8],
}

#[test]
fn writes_cross_valid_fragmented_destination_boundary() {
    for split in 0..=8 {
        let mut left = vec![0u8; split];
        let mut right = vec![0u8; 8 - split];
        let mut dest = left.as_mut_slice().chain_mut(right.as_mut_slice());
        dest.write(&Record {
            bytes: *b"abcdefgh",
        });
        assert_eq!(dest.remaining_mut(), 0);
        let joined: Vec<_> = left.into_iter().chain(right).collect();
        assert_eq!(joined, b"abcdefgh");
    }
}

#[test]
fn bytes_only_owned_copy_chunk_control() {
    let data = Bytes::copy_from_slice(&[1u8]);
    assert_eq!(data.chunk(), &[1]);
}
