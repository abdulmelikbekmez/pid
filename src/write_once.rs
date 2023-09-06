use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct WriteOnce<T>(*mut T);

unsafe impl<T> Send for WriteOnce<T> {}
unsafe impl<T> Sync for WriteOnce<T> {}

impl<T> WriteOnce<T> {
    pub fn new(data: T) -> Self {
        Self(Box::into_raw(Box::new(data)))
    }

    pub fn tmp(&self) -> &mut T {
        unsafe { self.0.as_mut().unwrap() }
    }

    #[inline(always)]
    pub fn update(&self, data: T) {
        unsafe { *self.0 = data }
    }
}

impl<T> Deref for WriteOnce<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl<T> DerefMut for WriteOnce<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut().unwrap() }
    }
}
