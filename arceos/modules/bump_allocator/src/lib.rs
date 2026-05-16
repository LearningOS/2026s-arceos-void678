#![no_std]

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    byte_pos: usize,
    page_pos: usize,
    byte_allocs: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            byte_pos: 0,
            page_pos: 0,
            byte_allocs: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = align_up(start, SIZE);
        self.end = align_down(start + size, SIZE);
        assert!(self.start <= self.end);
        self.byte_pos = self.start;
        self.page_pos = self.end;
        self.byte_allocs = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        if self.start == 0 && self.end == 0 {
            self.init(start, size);
            return Ok(());
        }

        let new_start = align_up(start, SIZE);
        let new_end = align_down(start + size, SIZE);
        if new_start == self.end {
            self.end = new_end;
            self.page_pos = new_end;
            Ok(())
        } else {
            Err(AllocError::MemoryOverlap)
        }
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<NonNull<u8>> {
        let size = layout.size().max(1);
        let start = align_up(self.byte_pos, layout.align());
        let end = start.checked_add(size).ok_or(AllocError::NoMemory)?;
        if end > self.page_pos {
            return Err(AllocError::NoMemory);
        }
        self.byte_pos = end;
        self.byte_allocs += 1;
        Ok(NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)?)
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: core::alloc::Layout) {
        if self.byte_allocs > 0 {
            self.byte_allocs -= 1;
            if self.byte_allocs == 0 {
                self.byte_pos = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_pos.saturating_sub(self.byte_pos)
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if num_pages == 0 || align_pow2 == 0 || !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let size = num_pages.checked_mul(SIZE).ok_or(AllocError::NoMemory)?;
        let pos = align_down(
            self.page_pos.checked_sub(size).ok_or(AllocError::NoMemory)?,
            align_pow2,
        );
        if pos < self.byte_pos {
            return Err(AllocError::NoMemory);
        }
        self.page_pos = pos;
        Ok(pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Early page allocations are kept until the allocator is reinitialized.
    }

    fn total_pages(&self) -> usize {
        self.total_bytes() / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        self.available_bytes() / SIZE
    }
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

const fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}
