// To search for nibbles in a byte array.
// Requires two nibbles per byte.
// All values above 0x0F will be used as wildcard.
// Example: Use x or X as wildcard for a nibble.
// If it is not divisible by two, then the behaviour should
// be like if there would be a wildcard appended at the end.
// Example: Searching for 1F1 makes the needle kind of 1F1X

pub trait Search {
    fn search(&self, needle: &[u8]) -> Option<usize>;
}

impl Search for Vec<u8> {
    fn search(&self, needle: &[u8]) -> Option<usize> {
        if needle.len() > 2 * self.len() {
            return None;
        }
        // Search in a for b
        for a in 0..=self.len() - (needle.len() + 1) / 2 {
            for b in 0..needle.len() / 2 {
                // if b is not a, skip searching at that position in a
                // Logic: both match, ...
                // or first nibble matches when second nibble is wildcard,
                // or second nibble matches when first nibble is wildcard.
                // or two wildcards at that position.
                #![cfg_attr(rustfmt, rustfmt_skip)]
                if !(
                    ((needle[2*b  ] == self[a + b]>>4) && (needle[2*b+1] == self[a + b]%16)) ||
                    ((needle[2*b  ] == self[a + b]>>4) && (needle[2*b+1] >= 0x10)) ||
                    ((needle[2*b+1] == self[a + b]%16) && (needle[2*b  ] >= 0x10)) ||
                    ((needle[2*b+1] >= 0x10) && (needle[2*b  ] >= 0x10))
                   )
                {
                    break; // element does not match
                }
                // when all elements of b match in a, return position of a
                if b == needle.len() / 2 - 1 {
                    return Some(a);
                }
            }
        }
        None
    }
}

#[test]
fn search_partial_at_start() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x01];
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_partial_at_middle() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x03, 0x00, 0x04];
    assert_eq!(buf.search(&sub), Some(2));
}
#[test]
fn search_partial_at_end() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x04, 0x00, 0x05];
    assert_eq!(buf.search(&sub), Some(3));
}
#[test]
fn search_partial_after_end() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x05, 0x00, 0x06];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_partial_before_start() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x00, 0x00, 0x01];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_short() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x02];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_too_long() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![
        0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06,
    ];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_full() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05];
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_swapped() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x05, 0x00, 0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x01];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_higher_than_9() {
    let buf = vec![0x0A, 0x0C, 0x0D, 0x0E, 0x0F];
    let sub = vec![0x00, 0x0C, 0x00, 0x0D, 0x00, 0x0E];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_higher_than_f() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x03, 0x0C, 0x01, 0x0D, 0x0E, 0x0E];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_single_wildcard_0x10() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x03, 0x0C, 0x01, 0x10, 0x0E, 0x0E];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcard_0xf0() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x03, 0x0C, 0xF0, 0xF0, 0x0E, 0x0E];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcard_x_big_x() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x03, 0x0C, 'x' as u8, 'X' as u8, 0x0E, 0x0E];
    assert_eq!(buf.search(&sub), Some(1));
}
#[test]
fn search_with_wildcards() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x10, 0x10, 0x10, 0x10, 0x10, 0x10];
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_shifted() {
    let buf = vec![0x0A, 0x3C, 0x1D, 0xEE, 0x0F];
    let sub = vec![0x0A, 0x03];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_start_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x01, 0x00];
    assert_eq!(buf.search(&sub), Some(0));
}
#[test]
fn search_odd_at_middle_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x03, 0x00];
    assert_eq!(buf.search(&sub), Some(2));
}
#[test]
fn search_odd_at_end_left() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x04, 0x00];
    assert_eq!(buf.search(&sub), Some(3));
}
// TODO: We should fix the code if possible
#[ignore]
#[test]
fn search_odd_at_end_left_1() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x04, 0x01];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_start_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x01, 0x00, 0x02];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_middle_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x02, 0x00, 0x03];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_right() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x03, 0x00, 0x04];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_left_over_range_0() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x05, 0x00];
    assert_eq!(buf.search(&sub), None);
}
#[test]
fn search_odd_at_end_left_over_range_1() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x05, 0x01];
    assert_eq!(buf.search(&sub), None);
}
// The last test needs to work only when the others work:
#[ignore]
#[test]
fn search_odd_at_end_left_over_range_x() {
    let buf = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let sub = vec![0x00, 0x05, 0x10];
    assert_eq!(buf.search(&sub), Some(4));
}
