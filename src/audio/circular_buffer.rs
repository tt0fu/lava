pub struct CircularBuffer<T> {
    size: usize,
    start: usize,
    count: usize,
    data: Vec<T>,
}

impl<T: Clone> CircularBuffer<T> {
    pub fn new(size: usize, fill: T) -> Self {
        Self {
            size,
            start: 0,
            count: 0,
            data: vec![fill; size],
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count <= 0 {
            None
        } else {
            let value = self.data[self.start].clone();
            self.start = (self.start + 1) % self.size;
            self.count -= 1;
            Some(value)
        }
    }
}

impl<T> CircularBuffer<T> {
    pub fn push(&mut self, value: T) {
        while self.count >= self.size {
            self.start = (self.start + 1) % self.size;
            self.count -= 1;
        }
        self.data[(self.start + self.count) % self.size] = value;
        self.count += 1;
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<T> std::ops::Index<usize> for CircularBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(self.start + index) % self.size]
    }
}
