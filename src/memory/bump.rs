//! # Bump frame allocator
//! Some code was borrowed from [Phil Opp's Blog](http://os.phil-opp.com/allocating-frames.html)

use crate::paging::PhysicalAddress;
use super::{Frame, FrameAllocator, MemoryArea, MemoryAreaIter};

use syscall::{PartialAllocStrategy, PhysallocFlags};

pub struct BumpAllocator {//一个分配器
    next_free_frame: Frame,//一个计数器
    current_area: Option<&'static MemoryArea>,//当前的范围
    areas: MemoryAreaIter,
    //用于标记内核使用的位置
    kernel_start: Frame,
    kernel_end: Frame
}

impl BumpAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, memory_areas: MemoryAreaIter) -> Self {
        let mut allocator = Self {
            next_free_frame: Frame::containing_address(PhysicalAddress::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(PhysicalAddress::new(kernel_start)),
            kernel_end: Frame::containing_address(PhysicalAddress::new(kernel_end))
        };
        allocator.choose_next_area();//初始化并分配一个合理的区域
        allocator
    }

    //此功能选择基地址最小的区域，该区域仍具有空闲帧，即next_free_frame小于其最后一帧。
    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.base_addr + area.length - 1;
            Frame::containing_address(PhysicalAddress::new(address as usize)) >= self.next_free_frame
        }).min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl FrameAllocator for BumpAllocator {
    #[allow(unused)]
    fn set_noncore(&mut self, noncore: bool) {}
    //计算得到当前区域的空闲帧数量
    fn free_frames(&self) -> usize {
        let mut count = 0;

        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            let end_frame = Frame::containing_address(PhysicalAddress::new((area.base_addr + area.length - 1) as usize));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                    count += 1;
                } else {
                    // Inside of used range
                }
            }
        }

        count
    }

    fn used_frames(&self) -> usize {
        let mut count = 0;
        //计算得到当前区域的已用帧数量
        for area in self.areas.clone() {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.base_addr as usize));
            let end_frame = Frame::containing_address(PhysicalAddress::new((area.base_addr + area.length - 1) as usize));
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                if frame >= self.kernel_start && frame <= self.kernel_end {
                    // Inside of kernel range
                    count += 1
                } else if frame >= self.next_free_frame {
                    // Frame is in free range
                } else {
                    count += 1;
                }
            }
        }

        count
    }
    //分配一定数量的帧：参数count:数量,flag:标志，min:如果允许多次分配则需要分配的最小数量
    fn allocate_frames3(&mut self, count: usize, flags: PhysallocFlags, strategy: Option<PartialAllocStrategy>, min: usize) -> Option<(Frame, usize)> {
        // TODO: Comply with flags and allocation strategies better.
        if count == 0 {
            return None;
        } else if let Some(area) = self.current_area {
            let space32 = flags.contains(PhysallocFlags::SPACE_32);
            let partial_alloc = flags.contains(PhysallocFlags::PARTIAL_ALLOC);
            let mut actual_size = count;

            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let start_frame = Frame { number: self.next_free_frame.number };
            let mut end_frame = Frame { number: self.next_free_frame.number + (count - 1) };
            let min_end_frame = if partial_alloc { Frame { number: self.next_free_frame.number + (min - 1) } } else { Frame { number: self.next_free_frame.number + (count - 1) } };
            //是否只分配部分帧
            //得到此区域的最后一帧
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(PhysicalAddress::new(address as usize))
            };

            if end_frame > current_area_last_frame && min_end_frame > current_area_last_frame {
                //此区域大小不够
                self.choose_next_area();
                return self.allocate_frames3(count, flags, strategy, min)//递归一下
            } else if partial_alloc {//此区域大小够并且只需要分配部分帧
                end_frame = Frame { number: self.next_free_frame.number + (min - 1) };
                actual_size = min;
            }
            //如果是以32位模式分配内存并且已经超出内存区域了
            if space32 && end_frame.start_address().get() + super::PAGE_SIZE >= 0x1_0000_0000 {
                // assuming that the bump allocator always advances, and that the memory map is sorted,
                // when allocating in 32-bit space we can only return None when the free range was
                // outside 0x0000_0000-0xFFFF_FFFF.
                //
                // we don't want to skip an entire memory region just because one 32-bit allocation failed.
                return None;
            }

            if (start_frame >= self.kernel_start && start_frame <= self.kernel_end)
                    || (end_frame >= self.kernel_start && end_frame <= self.kernel_end) {
                //帧已经用作内核区域
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
                // 更新next_free_frame的值以继续尝试分配。
                return self.allocate_frames3(count, flags, strategy, min)
            }

            //找到了合适的帧了，分配
            self.next_free_frame.number += actual_size;
            return Some((start_frame, actual_size));
        } else {
            None // no free memory areas left, and thus no frames left
        }
    }

    fn deallocate_frames(&mut self, _frame: Frame, _count: usize) {
        //panic!("BumpAllocator::deallocate_frame: not supported: {:?}", frame);
    }
}
