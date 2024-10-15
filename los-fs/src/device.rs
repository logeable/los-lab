use crate::error::Result;

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, id: usize, data: &mut [u8]) -> Result<()>;
    fn write_block(&self, id: usize, data: &[u8]) -> Result<()>;
}
