use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use txn_manager::lockmanager::LockManager;
use std::sync::atomic::Ordering::Relaxed;

use crate::buffer_frame::{BufferFrame, FrameGuard};
use crate::prelude::*;
use common::prelude::*;

pub const FRAMES: usize = 500;
pub const MAX_CONTAINERS: usize = 256; 

unsafe impl Sync for BufferFrame {}
unsafe impl Send for BufferFrame {}
unsafe impl Sync for BufferPool {}
unsafe impl Send for BufferPool {}

pub trait BufferPoolTrait: Sync + Send {
    /// Append a new page to a container. Returns the page id and a guard to the frame
    fn new_page(&self, c_id: ContainerId) -> Result<(PageId, FrameGuard), CrustyError>;
    /// Get a frame guard for a page (will ignore any slot_id on the ValueId). Increments the pin count
    fn get_page(&self, v_id: &ValueId, perm: Permissions) -> Result<FrameGuard, CrustyError>;
    /// Register a new container. Returns the container id.
    fn register_container(&self, name: Option<String>, state: StateType) -> Result<ContainerId, CrustyError>;
}

/// Stores the metadata for a container
pub struct ContainerMeta {
    pub container_id: ContainerId,
    pub name: Option<String>,
    pub container_type: StateType,
    pub max_page: PageId,
    pub key_size: usize,
    pub value_size: usize,
}

/// A in-memory buffer pool for managing pages. no eviction policy
pub struct BufferPool {
    // frames: UnsafeCell<[BufferFrame; FRAMES]>,
    /// The buffer frames. Any changes to the frames should be done with the latch held
    frames: UnsafeCell<Vec<BufferFrame>>,
    /// The lock manager
    _lm: Arc<LockManager>,
    /// Mapping of container/page to frame offset. CP_BYTES is the valueID in bytes without slot
    frame_map: UnsafeCell<HashMap<[u8; common::ids::ValueId::CP_BYTES], usize>>,
    /// The container metadata as an array.
    containers: UnsafeCell<[Option<ContainerMeta>; MAX_CONTAINERS]>,
    /// The mutex latch for the buffer pool
    latch: AtomicBool,
    /// The next free frame.
    free_frame: AtomicUsize,
}

impl BufferPool {
    pub fn new(lm: Arc<LockManager>) -> Self {
        info!("Creating a new BP");
        // let frames = [ BufferFrame::new(0); FRAMES];
        // let frames: [BufferFrame; FRAMES] = core::array::from_fn(|i| {
        //     let frame = BufferFrame::new(i);
        //     frame
        // });
        let mut frames = Vec::with_capacity(FRAMES);
        for i in 0 .. FRAMES {
            frames.push(BufferFrame::new(i));
        }
        
        BufferPool {
            frames: UnsafeCell::new(frames),
            //external_frames: Vec::new(),
            _lm: lm,
            frame_map: UnsafeCell::new(HashMap::new()),
            containers: UnsafeCell::new([const { None }; MAX_CONTAINERS]),
            latch: AtomicBool::new(false),
            free_frame: AtomicUsize::new(0),
        }
    }

    pub fn acquire_latch(&self) -> Result<(), CrustyError> {
        const LATCH_TIMEOUT_MS: u64 = 1000;
        let timeout = std::time::Instant::now() + std::time::Duration::from_millis(LATCH_TIMEOUT_MS);
        loop {
            if std::time::Instant::now() > timeout {
                return Err(CrustyError::CrustyError("Latch timeout".to_string()));
            }
            match self.latch.compare_exchange(false, true, Relaxed, Relaxed) {
                Ok(_b) => return Ok(()),
                Err(_b) => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    pub fn release_latch(&self) {
        self.latch.store(false, Relaxed);
    }
}

impl BufferPoolTrait for BufferPool {
    fn new_page(&self, c_id: ContainerId) -> Result<(PageId, FrameGuard), CrustyError> {
        self.acquire_latch()?;
        // Find the next page id
        let cm = unsafe { &mut *self.containers.get()};
        if cm[c_id as usize].is_none(){
            error!("Trying to create new page for non-registered Container Id {}", c_id);
            self.release_latch();
            return Err(CrustyError::StorageError);
        }
        let meta = cm[c_id as usize].as_mut().unwrap();
        let new_pid = meta.max_page;
        meta.max_page += 1;

        // Find the free frame
        let frame_offset = self.free_frame.fetch_add(1, Relaxed);
        if frame_offset >= FRAMES {
            self.release_latch();
            return Err(CrustyError::CrustyError("Out of free frames".to_string()));
        }
        let frames = unsafe { &mut *self.frames.get()};
        let frame = &frames[frame_offset];

        // Set the frame's meta data to match the container
        let page = unsafe { &mut *frame.page.get()};
        page.update_settings(new_pid, meta.key_size, meta.value_size);

        // Add the cid/vid to frame offset to map
        let cp_bytes = ValueId::new_page(c_id, new_pid).to_cp_bytes();
        let frame_map = unsafe {&mut *self.frame_map.get()};
        frame_map.insert(cp_bytes, frame_offset);

        // Create the frame guard and track the pin
        frame.pin_count.fetch_add(1, Relaxed);
        let guard = FrameGuard {
            buffer_frame: frame
        };
        self.release_latch();
        Ok((new_pid, guard))
    }

    fn get_page(&self, v_id: &ValueId, perm: Permissions) -> Result<FrameGuard, CrustyError> {
        let cp_bytes = v_id.to_cp_bytes();
        self.acquire_latch()?;
        //Find the frame
        let frame_map = unsafe {&mut *self.frame_map.get()};
        let frame_offset = frame_map.get(&cp_bytes);
        if frame_offset.is_none() {
            self.release_latch();
            return Err(CrustyError::CrustyError("Trying to get page that does not exist".to_string()));
        }
        let frame_offset = *frame_offset.unwrap();
        let frames = unsafe { &mut *self.frames.get()};
        let frame = &frames[frame_offset];
        self.release_latch();
        frame.pin_count.fetch_add(1, Relaxed);
        Ok(FrameGuard{
            buffer_frame: frame
        })
    }

    fn register_container(&self, name: Option<String>, state: StateType) -> Result<ContainerId, CrustyError> {
        self.acquire_latch()?;
        let cm = unsafe { &mut *self.containers.get()};
        let cid = cm.iter().position(|x| x.is_none() );
        if cid.is_none() {
            self.release_latch();
            let s = "Ran out of container IDs. Up max".to_string();
            error!("{}",s);
            return Err(CrustyError::CrustyError(s));
        }
        let cid = cid.unwrap();
        // hack for fixed size structures
        let (key_size,value_size) = match state {
            StateType::HashTable => (SEARCH_KEY_SIZE, INDEX_POINTER_SIZE),
            StateType::Tree => (SEARCH_KEY_SIZE, INDEX_POINTER_SIZE),
            StateType::BaseTable => (KEY_SIZE, VALUE_SIZE),
            StateType::MatView => (KEY_SIZE, VALUE_SIZE)
        };
        cm[cid] = Some(ContainerMeta {
            container_id: cid as ContainerId,
            name,
            container_type: state,
            max_page: 0,
            key_size,
            value_size
        });
        self.release_latch();
        Ok(cid as ContainerId)
    }
}

#[cfg(test)]
mod test {
    use common::testutil::init;
    use txn_manager::{lm_trait::LockManagerTrait, lockmanager::LockManager};
    use super::*;

    #[test]
    fn test_bp_simple() {
        init();
        let lm = Arc::new(LockManager::new(500));
        let bp = BufferPool::new(lm);
        let c1 = bp.register_container(None, StateType::BaseTable).expect("Got CID");
        assert_eq!(c1, 0);
        let c2 = bp.register_container(None, StateType::HashTable).expect("Got CID");
        assert_eq!(c2, 1);
        let (p, g) = bp.new_page(c1).expect("Got page");
        assert_eq!(p, 0);
        assert_eq!(g.buffer_frame.frame_id, 0);
        drop(g);
        let (p, g) = bp.new_page(c2).expect("Got page");
        assert_eq!(p, 0);
        assert_eq!(g.buffer_frame.frame_id, 1);
        let (p, mut g) = bp.new_page(c1).expect("Got page");
        assert_eq!(p, 1);
        assert!(g.get_kv(0).is_none());
        let key = [1; KEY_SIZE];
        let val = [2; VALUE_SIZE];
        let slot = g.write(0, false, &key, &val);
        assert!(slot.is_ok());
        drop(g);
        let v_id = ValueId::new_page(c1, p);
        let g = bp.get_page(&v_id, Permissions::ReadOnly).unwrap();
        let (k,v) = g.get_kv(0).unwrap();
        assert_eq!(k, key);
        assert_eq!(v, val);
        assert_eq!(g.buffer_frame.pin_count.load(Relaxed),1);
    }

}
