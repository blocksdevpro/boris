use std::collections::VecDeque;

pub struct SlidingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

pub struct RecordBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

impl RecordBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, samples: &[f32]) {
        for &s in samples {
            if self.buffer.len() == self.capacity {
                self.buffer.pop_front(); // O(1), no shifting
            }
            self.buffer.push_back(s); // O(1) amortized
        }
    }

    // trim should trim the buffer's oldest samples and just keep the most recent ones (up to the capacity)
    pub fn trim(&mut self, size: usize) {
        while self.buffer.len() > size {
            self.buffer.pop_front();
        }
    }

    pub fn read(&self) -> Vec<f32> {
        self.buffer.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl SlidingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    pub fn push(&mut self, samples: &[f32]) {
        for &s in samples {
            if self.buffer.len() == self.capacity {
                self.buffer.pop_front(); // O(1), no shifting
            }
            self.buffer.push_back(s); // O(1) amortized
        }
    }

    pub fn ready(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    pub fn read(&self) -> Vec<f32> {
        let size = self.capacity.min(self.buffer.len());
        let start = self.buffer.len() - size;
        self.buffer.range(start..).copied().collect()
    }

    pub fn read_last(&self, n: usize) -> Vec<f32> {
        let start = self.buffer.len().saturating_sub(n);
        self.buffer.range(start..).copied().collect()
    }
}
