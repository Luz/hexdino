pub trait FindSubset {
    fn find_subset(&self, subset: &[u8]) -> Option<usize>;
}

impl FindSubset for Vec<u8> {
    fn find_subset(&self, subset: &[u8]) -> Option<usize> {
        if subset.len() > self.len() {
            return None;
        }
        for a in 0 .. self.len() - subset.len() + 1 { //a in self.iter() { //Search in a for b
            for b in 0 .. subset.len() { //subset.iter() {
                if subset[b] != self[a+b] { // if b is not a, skip searching at that position in a
                    break; // element does not match
                }
                if b == subset.len()-1 { // when all elements of b match in a, return position of a
                    return Some(a);
                }
            }
        }
        None
    }
}

#[test]
fn find_subset_1() { // Partial at start
    let buf = vec![1,2,3,4,5];
    let sub = vec![1,2];
    assert_eq!(buf.find_subset(&sub) , Some(0));
}
#[test]
fn find_subset_2() { // Partial at middle
    let buf = vec![1,2,3,4,5];
    let sub = vec![3,4];
    assert_eq!(buf.find_subset(&sub) , Some(2));
}
#[test]
fn find_subset_3() { // Partial at end
    let buf = vec![1,2,3,4,5];
    let sub = vec![4,5];
    assert_eq!(buf.find_subset(&sub) , Some(3));
}
#[test]
fn find_subset_4() { // Partial after end
    let buf = vec![1,2,3,4,5];
    let sub = vec![5,6];
    assert_eq!(buf.find_subset(&sub) , None);
}
#[test]
fn find_subset_5() { // Partial before start
    let buf = vec![1,2,3,4,5];
    let sub = vec![0,1];
    assert_eq!(buf.find_subset(&sub) , None);
}
#[test]
fn find_subset_6() { // short
    let buf = vec![1,2,3,4,5];
    let sub = vec![2];
    assert_eq!(buf.find_subset(&sub) , Some(1));
}
#[test]
fn find_subset_7() { // too long
    let buf = vec![1,2,3,4,5];
    let sub = vec![1,2,3,4,5,6];
    assert_eq!(buf.find_subset(&sub) , None);
}
#[test]
fn find_subset_8() { // full
    let buf = vec![1,2,3,4,5];
    let sub = vec![1,2,3,4,5];
    assert_eq!(buf.find_subset(&sub) , Some(0));
}
#[test]
fn find_subset_9() { // Swapped
    let buf = vec![1,2,3,4,5];
    let sub = vec![5,4,3,2,1];
    assert_eq!(buf.find_subset(&sub) , None);
}

