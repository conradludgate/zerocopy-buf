use bytes::{Bytes, BytesMut};
use zerocopy::{network_endian, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};
use zerocopy_buf::ZeroCopyBuf;

#[derive(FromBytes, KnownLayout, Immutable, Unaligned, IntoBytes, PartialEq, Debug)]
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

#[derive(FromBytes, KnownLayout, Immutable, Unaligned, IntoBytes, PartialEq, Debug)]
#[repr(transparent)]
struct Ipv4Addr([u8; 4]);

#[test]
fn try_get() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\xff\xfe\xfd\xfc";
    let mut data = Bytes::from_static(header);
    let header = data.try_get_elems::<[Ipv4Header]>(2).unwrap();

    assert_eq!(data, b"\xff\xfe\xfd\xfc"[..]);
    drop(data);

    assert_eq!(
        *header,
        [
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
            },
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
        ]
    );
}

#[test]
fn try_get_write() {
    let header =
        b"\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\x45\x00\x00\x14\x00\x00\x00\x00\x01\x06\x00\x00\x7f\x00\x00\x01\x7f\x00\x00\x02\xff\xfe\xfd\xfc";
    let mut data = BytesMut::from(&header[..]);
    let mut header = data.try_get_elems::<[Ipv4Header]>(2).unwrap();

    assert_eq!(data, b"\xff\xfe\xfd\xfc"[..]);
    drop(data);

    assert_eq!(
        *header,
        [
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
            },
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
        ]
    );

    header[0].checksum.set(1);
    header[1].checksum.set(1);
}
