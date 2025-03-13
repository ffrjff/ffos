
pub struct AddressSpace {
    pub pid: usize,
    /// 地址空间标识
    pub token: usize,
    /// 根页表
    pub root_pt: PageTable,
    /// 地址空间中的区域
    regions: BTreeMap<VirtPageNum, Box<dyn ASRegion>>,
    /// 该地址空间关联的页表帧
    pt_dirs: Vec<HeapFrameTracker>,
    /// System V 共享内存
    sysv_shm: Arc<Mutex<SysVShm>>,
    /// 堆范围
    heap: Range<VirtPageNum>,
    /// 堆最大位置
    heap_max: VirtPageNum,
  }