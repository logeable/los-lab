use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    device::BlockDevice,
    error::{Error, Result},
    BLOCK_CACHE_COUNT, BLOCK_SIZE,
};

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}

pub struct BlockCache {
    id: usize,
    data: [u8; BLOCK_SIZE],
    device: Arc<dyn BlockDevice>,
    modified: bool,
}

impl BlockCache {
    pub fn init(id: usize, device: Arc<dyn BlockDevice>) -> Result<Self> {
        let mut data = [0u8; BLOCK_SIZE];
        device.read_block(id, &mut data)?;

        Ok(Self {
            id,
            data,
            device,
            modified: false,
        })
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn sync(&mut self) -> Result<()> {
        if self.modified {
            self.modified = false;
            self.device.write_block(self.id, &self.data)?;
        }

        Ok(())
    }
}

pub struct BlockCacheManager {
    caches: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            caches: VecDeque::new(),
        }
    }

    pub fn get_block_cache(
        &mut self,
        id: usize,
        device: Arc<dyn BlockDevice>,
    ) -> Result<Arc<Mutex<BlockCache>>> {
        if let Some(cache) = self
            .caches
            .iter()
            .find(|(cid, _cache)| *cid == id)
            .map(|(_id, cache)| cache)
        {
            Ok(cache.clone())
        } else {
            if self.caches.len() == BLOCK_CACHE_COUNT {
                if let Some(idx) = self
                    .caches
                    .iter()
                    .position(|(_, cache)| Arc::strong_count(cache) == 1)
                {
                    self.caches.remove(idx);
                } else {
                    return Err(Error::NoFreeCache);
                }
            }

            let cache = BlockCache::init(id, device.clone()).map(|v| Arc::new(Mutex::new(v)))?;

            self.caches.push_back((id, cache.clone()));
            Ok(cache)
        }
    }
}

impl Default for BlockCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_block_cache(id: usize, device: Arc<dyn BlockDevice>) -> Result<Arc<Mutex<BlockCache>>> {
    BLOCK_CACHE_MANAGER.lock().get_block_cache(id, device)
}
