use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub trait RW {}
pub struct RPtr;
pub struct WPtr;

impl RW for RPtr {}
impl RW for WPtr {}

pub struct RWPtr<T, K: RW> {
    data: *mut T,
    phantom: PhantomData<K>,
}

pub fn new<T>(data: T) -> (RWPtr<T, RPtr>, RWPtr<T, WPtr>) {
    let ptr = Box::into_raw(Box::new(data));
    let read = RWPtr {
        data: ptr,
        phantom: PhantomData::<RPtr>,
    };

    let write = RWPtr {
        data: ptr,
        phantom: PhantomData::<WPtr>,
    };
    (read, write)
}

impl<T, K: RW> Deref for RWPtr<T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T> Clone for RWPtr<T, RPtr> {
    fn clone(&self) -> Self {
        RWPtr {
            data: self.data,
            phantom: PhantomData::<RPtr>,
        }
    }
}
impl<T> DerefMut for RWPtr<T, WPtr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<T> AsMut<T> for RWPtr<T, WPtr> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap() }
    }
}

unsafe impl<T, K: RW> Send for RWPtr<T, K> {}
unsafe impl<T, K: RW> Sync for RWPtr<T, K> {}
