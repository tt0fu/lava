pub struct CircularBuffer<T, const SIZE: usize> {
    start: usize,
    size: usize,
    data: [T; SIZE],
}

impl<T: Copy, const SIZE: usize> CircularBuffer<T, SIZE> {
    pub fn new(fill: T) -> Self {
        Self {
            start: 0,
            size: 0,
            data: [fill; SIZE],
        }
    }

    pub fn push(&mut self, value: &T) {
        while self.size >= SIZE {
            self.start = (self.start + 1) % SIZE;
            self.size -= 1;
        }
        self.data[(self.start + self.size) % SIZE] = value.clone();
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size <= 0 {
            None
        } else {
            let value = self.data[self.start].clone();
            self.start = (self.start + 1) % SIZE;
            self.size -= 1;
            Some(value)
        }
    }

    pub fn data(&self) -> [T; SIZE] {
        self.data
    }
}

impl<T, const SIZE: usize> CircularBuffer<T, SIZE> {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start(&self) -> usize {
        self.start
    }
}

impl<T, const SIZE: usize> std::ops::Index<usize> for CircularBuffer<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(self.start + index) % SIZE]
    }
}
