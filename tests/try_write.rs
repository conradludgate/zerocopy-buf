use bytes::{BufMut, BytesMut};
use zerocopy::{network_endian, Immutable, IntoBytes};
use zerocopy_buf::ZeroCopyBufMut;

#[derive(IntoBytes, Immutable, PartialEq, Debug)]
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

#[derive(IntoBytes, Immutable, PartialEq, Debug)]
#[repr(transparent)]
struct Ipv4Addr([u8; 4]);

#[test]
fn write() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";
    let mut data = BytesMut::new();
    data.write(&Ipv4Header {
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
    });

    assert_eq!(data, header[..]);
}

#[test]
fn write_chunked() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";

    let mut lhs = [0; 10];
    let mut rhs = [0; 10];

    let mut data = (&mut lhs[..]).chain_mut(&mut rhs[..]);

    data.write(&Ipv4Header {
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
    });

    assert_eq!(lhs, header[..10]);
    assert_eq!(rhs, header[10..]);
}
