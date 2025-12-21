use super::*;

#[test]
fn search_partial_at_start() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "01".as_bytes();
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_partial_at_middle() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0304".as_bytes();
    assert_eq!(buf.search(&sub), Some(2));
}
#[test]
fn search_partial_at_end() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0405".as_bytes();
    assert_eq!(buf.search(&sub), Some(3));
}
#[test]
fn search_partial_after_end() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0506".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_partial_before_start() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0001".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_short() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "02".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_too_long() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "010203040506".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_full() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0102030405".as_bytes();
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_swapped() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "0504030201".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_higher_than_9() {
    let buf = vec![0x0A, 0x0C, 0x0D, 0x0E, 0x0F];
    let sub = "0C0D0E".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_higher_than_f() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "3C1DEE".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_single_wildcard_0x10() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "3C1\x10EE".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcard_0x7f() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "3C\x7f\x7fEE".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcard_x_big_x() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "3CxXEE".as_bytes();
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcards() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "xxxxxx".as_bytes();
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_shifted() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = "A3".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_start_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "010".as_bytes();
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_odd_at_middle_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "030".as_bytes();
    assert_eq!(buf.search(&sub), Some(2));
}
#[test]
fn search_odd_at_end_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "040".as_bytes();
    assert_eq!(buf.search(&sub), Some(3));
}
#[test]
fn search_odd_at_end_left_1() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "041".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_start_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "102".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_middle_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "203".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "304".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_left_over_range_0() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "050".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_left_over_range_1() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "051".as_bytes();
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_left_over_range_x() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = "05X".as_bytes();
    assert_eq!(buf.search(&sub), Some(4));
}
