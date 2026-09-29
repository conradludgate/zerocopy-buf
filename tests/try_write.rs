use bytes::{BufMut, BytesMut};
use zerocopy::{network_endian, AsBytes};
use zerocopy_buf::ZeroCopyBufMut;

#[derive(AsBytes, PartialEq, Debug)]
#[repr(C)]
struct Ipv4Header {
    version_uhl: u8,
    dscp_ecn: u8,
    total_length: network_endian::U16,
    identification: network_endian::U16,
    flags_fragment: network_endian::U16,
    ttl: u8,
    protocol: u8,
    checksum: network_endian::U16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
}

#[derive(AsBytes, PartialEq, Debug)]
#[repr(transparent)]
struct Ipv4Addr([u8; 4]);

fn header() -> Ipv4Header {
    Ipv4Header {
        version_uhl: 0x45,
        dscp_ecn: 0x00,
        total_length: network_endian::U16::new(20),
        identification: network_endian::U16::new(0),
        flags_fragment: network_endian::U16::new(0),
        ttl: 1,
        protocol: 6,
        checksum: network_endian::U16::new(0),
        src: Ipv4Addr([127, 0, 0, 1]),
        dst: Ipv4Addr([127, 0, 0, 2]),
    }
}

const ENCODED: &[u8] =
    b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";

#[test]
fn write_appends_encoded_bytes() {
    let mut data = BytesMut::from(&b"prefix"[..]);

    data.write(header());

    assert_eq!(&data[..6], b"prefix");
    assert_eq!(&data[6..], ENCODED);
}

#[test]
fn write_to_chunked_buffer() {
    let mut lhs = [0; 10];
    let mut rhs = [0; 10];
    let mut data = (&mut lhs[..]).chain_mut(&mut rhs[..]);

    data.write(header());

    assert_eq!(lhs, ENCODED[..10]);
    assert_eq!(rhs, ENCODED[10..]);
}

#[test]
fn write_unsized_byte_slice() {
    let mut data = BytesMut::new();
    let payload: &[u8] = b"payload";

    data.write_ref(payload);

    assert_eq!(data, payload);
}
