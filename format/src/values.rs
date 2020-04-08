use derive_more::{Deref, DerefMut, From};

#[derive(Debug, Copy, Clone, Deref, DerefMut, From)]
pub struct Sha1(pub [u8; 20]);

impl PartialEq for Sha1 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq::constant_time_eq(&self[..], &other[..])
    }
}
impl Eq for Sha1 {}
