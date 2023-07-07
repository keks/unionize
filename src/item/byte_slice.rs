use super::Item;

impl<const L: usize> Item for [u8; L] {
    fn zero() -> Self {
        [0u8; L]
    }

    fn next(&self) -> Self {
        let mut result: [u8; L] = self.clone();
        for i in 0..L {
            let (sum, did_overflow) = result[i].overflowing_add(1);
            result[i] = sum;
            if !did_overflow {
                break;
            }
        }
        result
    }
}
