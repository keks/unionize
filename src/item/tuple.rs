use crate::Item;

impl<I1, I2> Item for (I1, I2)
where
    I1: Item,
    I2: Item,
{
    fn zero() -> Self {
        (I1::zero(), I2::zero())
    }

    fn next(&self) -> Self {
        (self.0.clone(), self.1.next())
    }
}
