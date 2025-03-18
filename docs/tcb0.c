
struct TaskControlBlock
{
    task_status : Ready, 
    task_context : TaskContext{
        return_address : 2149613752, 
        sp : 18446744073709547520, 
        callee_saved_register : [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ]}, 
    address_space : AddressSpace{
        pid : 0, 
        page_table : PageTable{
            satp : PPN : 0x805b5, 
            frames : [
                FrameTracker:PPN = 0x805b5, 
                FrameTracker:PPN = 0x805b4, 
                FrameTracker:PPN = 0x805b3, 
                FrameTracker:PPN = 0x805b7, 
                FrameTracker:PPN = 0x805b8]}, 
        memory_regions : [ 
             LazyRegion{
                start : VPN : 0x10, 
                end : VPN : 0x12, 
                pages : {
                    VPN : 0x10 : Framed(FrameTracker : PPN = 0x805b6), 
                    VPN : 0x11 : Framed(FrameTracker : PPN = 0x805b9)}, 
                    permission : R | X | U}, 
            LazyRegion{
                start : VPN : 0x12, 
                end : VPN : 0x13, 
                pages : {
                    VPN : 0x12 : Framed(FrameTracker : PPN = 0x805ba)}, 
                    permission : R | U}, 
            LazyRegion{
                start : VPN : 0x13, 
                end : VPN : 0x14, 
                pages : {
                    VPN : 0x13 : Framed(FrameTracker : PPN = 0x805bb)}, 
                    permission : R | W | U}, 
            LazyRegion{
                start : VPN : 0x15, 
                end : VPN : 0x17, 
                pages : {
                    VPN : 0x15 : Framed(FrameTracker : PPN = 0x805bc), 
                    VPN : 0x16 : Framed(FrameTracker : PPN = 0x805bd)}, 
                    permission : R | W | U}, 
            lazyRegion{
                start : VPN : 0x17, 
                end : VPN : 0x17, 
                pages : {}, 
                permission : R | W | U}, 
            LazyRegion{
                start : VPN : 0x7fffffe, 
                end : VPN : 0x7ffffff, 
                pages : {
                    VPN : 0x7fffffe : Framed(FrameTracker : PPN = 0x805be)}, 
                    permission : R | W} ], 
        entry_point : 0}, 
    trap_context_ppn : PPN : 0x805be, 
    base_size : 94208, 
    heap_bottom : 94208, 
    program_break : 94208
}
