use std::ops::DerefMut;
use std::{cell::UnsafeCell, ops::Deref, sync::atomic::AtomicU8};
use std::fmt::Debug;
use crate::fixed_page::FixedPage;
use std::sync::atomic::Ordering::Relaxed;

pub struct BufferFrame {
    pub page: UnsafeCell<FixedPage>,
    pub(crate) frame_id: usize,
    pub(crate) pin_count: AtomicU8,
}

pub struct FrameGuard<'a> {
    pub(crate) buffer_frame: &'a BufferFrame,
}

impl BufferFrame {
    pub fn new(frame_id: usize) -> Self {
        BufferFrame {
            page: UnsafeCell::new(FixedPage::empty()),
            frame_id,
            pin_count: AtomicU8::new(0)
        }
    }
    pub fn read(&self) -> FrameGuard {
        self.pin_count.fetch_add(1, Relaxed);
        FrameGuard {
            buffer_frame: self
        }
    }
}

impl Deref for FrameGuard<'_> {
    type Target = FixedPage;

    fn deref(&self) -> &Self::Target {
        // SAFETY: This is safe because the latch is held shared.
        unsafe { &*self.buffer_frame.page.get() }
    }
}

impl DerefMut for FrameGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: This is safe because the latch is held exclusively.
        unsafe { &mut *self.buffer_frame.page.get() }
    }
}

impl Drop for FrameGuard<'_> {
    fn drop(&mut self) {
        self.buffer_frame.pin_count.fetch_sub(1, Relaxed);
    }
}
impl Debug for FrameGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameGuard")
            .field("frame", &self.buffer_frame.frame_id)
            .field("pin", &self.buffer_frame.pin_count.load(Relaxed))
            .finish()
    }
}
