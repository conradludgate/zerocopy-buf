use bytes::{Buf, Bytes};
use zerocopy::{network_endian, FromBytes, FromZeroes};
use zerocopy_buf::ZeroCopyReadBuf;

#[derive(FromBytes, FromZeroes, PartialEq, Debug)]
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

#[derive(FromBytes, FromZeroes, PartialEq, Debug)]
#[repr(transparent)]
struct Ipv4Addr([u8; 4]);

fn expected_header() -> Ipv4Header {
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

const HEADER: &[u8] =
    b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";

#[test]
fn read_bytes_consumes_only_the_value() {
    let mut data = Bytes::from_static(
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\xff\xfe",
    );

    assert_eq!(data.try_read::<Ipv4Header>().unwrap(), expected_header());
    assert_eq!(data, b"\xff\xfe"[..]);
}

#[test]
fn read_chunked_consumes_only_the_value() {
    let input =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\xff\xfe";
    let (first, rest) = input.split_at(7);
    let (second, third) = rest.split_at(8);
    let mut data = Bytes::from_static(first)
        .chain(Bytes::from_static(second))
        .chain(Bytes::from_static(third));

    assert_eq!(data.try_read::<Ipv4Header>().unwrap(), expected_header());
    assert_eq!(data.chunk(), b"\xff\xfe");
}

#[test]
fn try_read_error_does_not_consume_input() {
    let mut data = Bytes::from_static(&HEADER[..HEADER.len() - 1]);

    assert!(data.try_read::<Ipv4Header>().is_none());
    assert_eq!(data, HEADER[..HEADER.len() - 1]);
}

#[test]
fn read_zero_sized_value_does_not_consume_input() {
    let mut data = Bytes::from_static(b"payload");

    assert_eq!(data.try_read::<()>().unwrap(), ());
    assert_eq!(data, b"payload"[..]);
}
