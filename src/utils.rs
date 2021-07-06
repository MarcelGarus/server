pub trait VecOfStrings {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>>;
    fn starts_with(&self, other: Vec<&str>) -> bool;
    fn clone_except_first(&self, n: usize) -> Vec<String>;
}
impl VecOfStrings for Vec<String> {
    fn clone_first_n(&self, n: usize) -> Option<Vec<String>> {
        if self.len() < n {
            None
        } else {
            Some(self.iter().take(n).map(|s| s.clone()).collect())
        }
    }
    fn starts_with(&self, other: Vec<&str>) -> bool {
        self.iter().zip(other).all(|(a, b)| a == b)
    }
    fn clone_except_first(&self, n: usize) -> Vec<String> {
        self.iter().skip(n).map(|s| s.clone()).collect()
    }
}
