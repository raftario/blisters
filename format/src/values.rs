use derive_more::{Deref, DerefMut, From};

#[derive(Copy, Clone, Debug, Deref, DerefMut, From)]
pub struct Sha1([u8; 20]);

impl Sha1 {
    #[inline]
    pub fn new(hash: [u8; 20]) -> Self {
        Self(hash)
    }
}

impl PartialEq for Sha1 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq::constant_time_eq(&self[..], &other[..])
    }
}
impl Eq for Sha1 {}
