#[derive(Clone)]
pub struct CircularBuffer<T> {
    start: usize,
    size: usize,
    data: Vec<T>,
}

impl<T: Copy> CircularBuffer<T> {
    pub fn new(max_size: usize, fill: T) -> Self {
        Self {
            start: 0,
            size: 0,
            data: vec![fill; max_size],
        }
    }

    pub fn push(&mut self, value: &T) {
        let len = self.data.len();

        while self.size >= len {
            self.start = (self.start + 1) % len;
            self.size -= 1;
        }
        self.data[(self.start + self.size) % len] = value.clone();
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size <= 0 {
            None
        } else {
            let value = self.data[self.start].clone();
            self.start = (self.start + 1) % self.data.len();
            self.size -= 1;
            Some(value)
        }
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start(&self) -> usize {
        self.start
    }
}

impl<T> std::ops::Index<usize> for CircularBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[(self.start + index) % self.data.len()]
    }
}
