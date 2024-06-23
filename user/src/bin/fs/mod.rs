use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

const VIRTIO0 : usize = 0x10001000;

struct VirtioHal;

struct VirtIOBlock {
    block : VirtIOBlk<'static, VirtioHal>,
}

impl Hal for VirtioHal {
    // Allocate an identity-mapped page.
    fn dma_alloc(pages: usize) -> usize {
        todo!("dma_alloc {:?}", pages);
    }
    // Deallocate an identity-mapped page.
    fn dma_dealloc(paddr: usize, pages: usize) -> i32 {
        todo!("dma_dealloc {:?} {:?}", paddr, pages);
    }
    // Currently, we choose to use identity mapping.
    fn phys_to_virt(paddr: usize) -> usize {
        return paddr;
    }
    // Currently, we choose to use identity mapping.
    fn virt_to_phys(vaddr: usize) -> usize {
        return vaddr;
    }
}

impl VirtIOBlock {
    pub fn new() -> Self {
        let block = unsafe {
            VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap()
        };
        return Self { block, };
    }

    // Read a block from the disk (in a blocking way).
    pub fn read_blocking(&mut self, block_id: usize, buf: &mut [u8]) {
        self.block.read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }

    // Write a block to the disk (in a blocking way).
    pub fn write_blocking(&mut self, block_id: usize, buf: &[u8]) {
        self.block.write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

#[allow(unused)]
fn get_block() -> &'static mut VirtIOBlock {
    static mut VIRTIO_BLOCK: Option<VirtIOBlock> = None;
    unsafe {
        if VIRTIO_BLOCK.is_none() {
            VIRTIO_BLOCK = Some(VirtIOBlock::new());
        }
        return VIRTIO_BLOCK.as_mut().unwrap();
    }
}

