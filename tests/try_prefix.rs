use bytes::{Bytes, BytesMut};
use zerocopy::{network_endian::U16, FromBytes, Immutable, KnownLayout, Unaligned};
use zerocopy_buf::ZeroCopyBuf;

#[derive(FromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Packet {
    header: U16,
    body: [U16],
}

#[test]
fn get_prefix_bytes() {
    let mut data = Bytes::from_static(b"\x01\x02\x03\x04\xff");
    let elems = data.try_get_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\xff"[..]);
}

#[test]
fn get_prefix_bytes_mut() {
    let mut data = BytesMut::from(&b"\x01\x02\x03\x04\xff"[..]);
    let elems = data.try_get_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\xff"[..]);
}

#[test]
fn get_prefix_slice() {
    let bytes = b"\x01\x02\x03\x04\xff";
    let mut data: &[u8] = bytes;
    let elems = data.try_get_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\xff");
}

#[test]
fn peek_prefix_bytes() {
    let mut data = Bytes::from_static(b"\x01\x02\x03\x04\xff");
    let elems = data.try_peek_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\x01\x02\x03\x04\xff"[..]);
}

#[test]
fn peek_prefix_bytes_mut() {
    let mut data = BytesMut::from(&b"\x01\x02\x03\x04\xff"[..]);
    let elems = data.try_peek_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\x01\x02\x03\x04\xff"[..]);
}

#[test]
fn peek_bytes_mut_methods() {
    let mut data = BytesMut::from(&b"\x01\x02\x03\x04"[..]);

    let first = data.try_peek::<U16>().unwrap();
    assert_eq!(first.get(), 0x0102);
    let elems = data.try_peek_elems::<[U16]>(2).unwrap();
    assert_eq!(elems.len(), 2);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, b"\x01\x02\x03\x04"[..]);
}

#[test]
fn peek_prefix_slice() {
    let bytes = b"\x01\x02\x03\x04\xff";
    let mut data: &[u8] = bytes;
    let elems = data.try_peek_prefix::<[U16]>().unwrap();

    assert_eq!(elems.len(), 2);
    assert_eq!(elems[0].get(), 0x0102);
    assert_eq!(elems[1].get(), 0x0304);
    assert_eq!(data, bytes);
}

#[test]
fn get_prefix_bytes_mut_error_restores_input() {
    let mut data = BytesMut::from(&b"\x01"[..]);
    assert!(data.try_get_prefix::<Packet>().is_err());
    assert_eq!(data, b"\x01"[..]);
}

#[test]
fn get_prefix_bytes_error_restores_input() {
    let mut data = Bytes::from_static(b"\x01");
    assert!(data.try_get_prefix::<Packet>().is_err());
    assert_eq!(data, b"\x01"[..]);
}

#[test]
fn get_bytes_mut_errors_restore_input() {
    let mut data = BytesMut::from(&b"\x01"[..]);
    assert!(data.try_get::<U16>().is_err());
    assert_eq!(data, b"\x01"[..]);

    let mut data = BytesMut::from(&b"\x01\x02"[..]);
    assert!(data.try_get_elems::<[U16]>(2).is_err());
    assert_eq!(data, b"\x01\x02"[..]);
}

#[test]
fn get_bytes_elems_error_restores_input() {
    let mut data = Bytes::from_static(b"\x01\x02");
    assert!(data.try_get_elems::<[U16]>(2).is_err());
    assert_eq!(data, b"\x01\x02"[..]);
}
