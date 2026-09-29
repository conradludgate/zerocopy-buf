use zerocopy::{network_endian::U16, KnownLayout, Immutable, Unaligned};
use zerocopy_buf::ZeroCopyBuf;

#[test]
fn slice_get_and_peek_methods() {
    let bytes = b"\x01\x02\x03\x04";

    let mut data: &[u8] = bytes;
    let first = data.try_get::<U16>().unwrap();
    assert_eq!(first.get(), 0x0102);
    assert_eq!(data, b"\x03\x04");

    let mut data: &[u8] = bytes;
    let first = data.try_get_elems::<[U16]>(1).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].get(), 0x0102);
    assert_eq!(data, b"\x03\x04");

    let mut data: &[u8] = bytes;
    let first = data.try_peek::<U16>().unwrap();
    assert_eq!(first.get(), 0x0102);
    assert_eq!(data, bytes);

    let mut data: &[u8] = bytes;
    let first = data.try_peek_elems::<[U16]>(1).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].get(), 0x0102);
    assert_eq!(data, bytes);
}

#[test]
fn slice_prefix_error_keeps_input() {
    #[derive(zerocopy::FromBytes, KnownLayout, Immutable, Unaligned)]
    #[repr(C)]
    struct Packet {
        header: U16,
        body: [U16],
    }

    let mut data: &[u8] = b"\x01";
    assert!(data.try_get_prefix::<Packet>().is_err());
    assert!(data.try_peek_prefix::<Packet>().is_err());
    assert_eq!(data, b"\x01");
}
