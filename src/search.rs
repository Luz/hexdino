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

        let needle_bytes_odd: bool = needle.len() % 2 == 1;
        let mut bytes_needed_to_match = needle.len() / 2;
        if needle_bytes_odd {
            bytes_needed_to_match += 1;
            // As we add the wildcard to make it even again
        }

        // Search in h (haystack index) for n (needle index)
        for h in 0..=self.len().saturating_sub(bytes_needed_to_match) {
            for n in 0..bytes_needed_to_match {
                // needle_left_nibble
                let nl = needle[2 * n];
                // needle_right_nibble
                let mut nr = 0x10; // Use the wildcard if it goes out of range
                if 2 * n + 1 < needle.len() {
                    // Ensuring no out of range access
                    nr = needle[2 * n + 1];
                }

                // haystack_left_nibble
                let hl = self[h + n] >> 4;
                // haystack_right_nibble
                let hr = self[h + n] % 16;

                // When left nibble does not match and it is not a wildcard
                if nl != hl && nl < 0x10 {
                    break; // skip as left nibble is not a wildcard and it does not match
                }
                if nr != hr && nr < 0x10 {
                    break; // skip as right nibble is not a wildcard and it does not match
                }
                // when all elements of n match in h
                if n + 1 == bytes_needed_to_match {
                    // return position of h
                    return Some(h);
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
