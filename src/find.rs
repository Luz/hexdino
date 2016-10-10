pub trait FindSubset {
    fn find_subset(&self, subset: &[u8]) -> Option<usize>;
}

impl FindSubset for Vec<u8> {
    fn find_subset(&self, subset: &[u8]) -> Option<usize> {
        if subset.len() > self.len() {
            return None;
        }
        // Search in a for b
        for a in 0..self.len() - subset.len() + 1 {
            for b in 0..subset.len() {
                // if b is not a, skip searching at that position in a
                if subset[b] != self[a + b] {
                    break; // element does not match
                }
                // when all elements of b match in a, return position of a
                if b == subset.len() - 1 {
                    return Some(a);
                }
            }
        }
        None
    }
}

#[test]
// Partial at start
fn find_subset_1() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![1, 2];
    assert_eq!(buf.find_subset(&sub), Some(0));
}
#[test]
// Partial at middle
fn find_subset_2() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![3, 4];
    assert_eq!(buf.find_subset(&sub), Some(2));
}
#[test]
// Partial at end
fn find_subset_3() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![4, 5];
    assert_eq!(buf.find_subset(&sub), Some(3));
}
#[test]
// Partial after end
fn find_subset_4() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![5, 6];
    assert_eq!(buf.find_subset(&sub), None);
}
#[test]
// Partial before start
fn find_subset_5() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![0, 1];
    assert_eq!(buf.find_subset(&sub), None);
}
#[test]
// short
fn find_subset_6() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![2];
    assert_eq!(buf.find_subset(&sub), Some(1));
}
#[test]
// too long
fn find_subset_7() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![1, 2, 3, 4, 5, 6];
    assert_eq!(buf.find_subset(&sub), None);
}
#[test]
// full
fn find_subset_8() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![1, 2, 3, 4, 5];
    assert_eq!(buf.find_subset(&sub), Some(0));
}
#[test]
// Swapped
fn find_subset_9() {
    let buf = vec![1, 2, 3, 4, 5];
    let sub = vec![5, 4, 3, 2, 1];
    assert_eq!(buf.find_subset(&sub), None);
}
