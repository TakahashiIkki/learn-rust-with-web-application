pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


/// aからbを引きます
///
/// # Examples
///
/// ```
/// use sum::sub;
///
/// let r = sub(10, 1);
/// assert_eq!(9, r);
/// ```
pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_test() {
        assert_eq!(2, add(1, 1));
    }
}