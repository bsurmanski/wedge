pub type Index = u32;

pub trait IndexType : Copy + Default {
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
    fn is_valid(&self) -> bool;

    fn to_option(&self) -> Option<Self> {
        if self.is_valid() {
            return None;
        }
        return Some(*self);
    }
}

impl IndexType for u32 {
    fn new(x: usize) -> Self {
        return x as Self;
    }

    fn index(&self) -> usize {
        return *self as usize;
    }

    fn is_valid(&self) -> bool {
        return *self != Self::max_value();
    }

    fn max() -> Self {
        return Self::max_value();
    }
}
