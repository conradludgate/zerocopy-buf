use bytes::{Buf, Bytes};
use zerocopy::{network_endian, FromBytes};
use zerocopy_buf::ZeroCopyReadBuf;

#[derive(FromBytes, PartialEq, Debug)]
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

#[derive(FromBytes, PartialEq, Debug)]
#[repr(transparent)]
struct Ipv4Addr([u8; 4]);

#[test]
fn read_bytes() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";
    let mut data = Bytes::from_static(header);
    let header = data.try_read::<Ipv4Header>().unwrap();

    assert_eq!(
        header,
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
    )
}

#[test]
fn read_chunked() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02";
    let (lhs, rhs) = header.split_at(10);
    let mut data = Bytes::from_static(lhs).chain(Bytes::from_static(rhs));
    let header = data.try_read::<Ipv4Header>().unwrap();

    assert_eq!(
        header,
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
    )
}
