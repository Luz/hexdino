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
    fn search(&self, search: &[u8]) -> Option<usize> {
        let mut needle = Vec::new();
        for i in 0..search.len() {
            let nibble = match search[i] as u8 {
                c @ 48..=57 => c - 48, // Numbers from 0 to 9
                c @ b'a'..=b'f' => c - 87,
                c @ b'A'..=b'F' => c - 55,
                _ => 0x10, // All other elements are wildcards
            };
            needle.push(nibble);
        }

        if needle.len() > 2 * self.len() {
            return None;
        }

        let needle_bytes_odd: bool = needle.len() % 2 == 1;
        let mut bytes_needed_to_match = needle.len() / 2;
        if needle_bytes_odd {
            bytes_needed_to_match += 1; // As we add the wildcard to make it even again
            if needle.last().unwrap_or(&0).clone() >= 0x10 {
                // This fixes the rare case when a user:
                // - uses an odd amount of nibbles to search
                // - ends his 3 nibbles with a wildcard character
                // - wants the 2 nibbles to match on the last character of the haystack
                // See: search_odd_at_end_left_over_range_x()
                bytes_needed_to_match -= 1;
            }
        }

        // Search in h (haystack index) for n (needle index)
        for h in 0..=self.len().saturating_sub(bytes_needed_to_match) {
            for n in 0..bytes_needed_to_match {
                // println!("h={}, n={}", h, n);
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
