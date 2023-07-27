use crate::dllist::{Dllink, Dllist};

// Define a static variable SENTINEL of type Dllink
static SENTINEL: Dllink = Dllink { data: [0, 8965], prev: std::ptr::null_mut(), next: std::ptr::null_mut() };

// Define a struct BPQueue
pub struct BPQueue {
    _max: usize,
    _offset: i32,
    _high: usize,
    _bucket: Vec<Dllist>,
}

impl BPQueue {
    // Define a new function for BPQueue
    pub fn new(a: i32, b: i32) -> Self {
        // Check if a is less than or equal to b
        assert!(a <= b);
        // Define offset as a - 1
        let offset = a - 1;
        // Define high as b - offset
        let high = (b - offset) as usize;
        // Define bucket as a vector of Dllist of size high + 1
        let mut bucket = vec![Dllist::new(); high + 1];
        // Append SENTINEL to the first element of bucket
        bucket[0].append(&mut SENTINEL.clone());
        // Return a new instance of BPQueue
        Self {
            _max: 0,
            _offset: offset,
            _high: high,
            _bucket: bucket,
        }
    }

    // Define a function to check if BPQueue is empty
    pub fn is_empty(&self) -> bool {
        self._max == 0
    }

    // Define a function to get the maximum value in BPQueue
    pub fn get_max(&self) -> i32 {
        (self._max as i32) + self._offset
    }

    // Define a function to clear BPQueue
    pub fn clear(&mut self) {
        while self._max > 0 {
            self._bucket[self._max].clear();
            self._max -= 1;
        }
    }

    // Define a function to set the key of an item in BPQueue
    pub fn set_key(&self, it: &mut Dllink, gain: i32) {
        it.data[0] = gain - self._offset;
    }

    // Define a function to append an item directly to BPQueue
    pub fn append_direct(&mut self, it: &mut Dllink) {
        assert!(it.data[0] > self._offset);
        self.append(it, it.data[0]);
    }

    // Define a function to append an item to BPQueue
    pub fn append(&mut self, it: &mut Dllink, k: i32) {
        assert!(k > self._offset);
        it.data[0] = k - self._offset;
        if self._max < it.data[0] as usize {
            self._max = it.data[0] as usize;
        }
        self._bucket[it.data[0] as usize].append(it);
    }

    // Define a function to append a vector of items to BPQueue
    pub fn appendfrom(&mut self, nodes: &mut Vec<Dllink>) {
        for it in nodes.iter_mut() {
            it.data[0] -= self._offset;
            assert!(it.data[0] > 0);
            self._bucket[it.data[0] as usize].append(it);
        }
        self._max = self._high;
        while self._bucket[self._max].is_empty() {
            self._max -= 1;
        }
    }

    // Define a function to remove and return the leftmost item in BPQueue
    pub fn popleft(&mut self) -> Dllink {
        let res = self._bucket[self._max].popleft();
        while self._bucket[self._max].is_empty() {
            self._max -= 1;
        }
        res
    }

    // Define a function to decrease the key of an item in BPQueue
    pub fn decrease_key(&mut self, it: &mut Dllink, delta: i32) {
        it.detach();
        it.data[0] += delta;
        assert!(it.data[0] > 0);
        assert!(it.data[0] <= self._high as i32);
        self._bucket[it.data[0] as usize].append(it);
        if self._max < it.data[0] as usize {
            self._max = it.data[0] as usize;
            return;
        }
        while self._bucket[self._max].is_empty() {
            self._max -= 1;
        }
    }

    // Define a function to increase the key of an item in BPQueue
    pub fn increase_key(&mut self, it: &mut Dllink, delta: i32) {
        it.detach();
        it.data[0] += delta;
        assert!(it.data[0] > 0);
        assert!(it.data[0] <= self._high as i32);
        self._bucket[it.data[0] as usize].appendleft(it);
        if self._max < it.data[0] as usize {
            self._max = it.data[0] as usize;
        }
    }

    // Define a function to modify the key of an item in BPQueue
    pub fn modify_key(&mut self, it: &mut Dllink, delta: i32) {
        if it.next.is_null() {
            return;
        }
        if delta > 0 {
            self.increase_key(it, delta);
        } else if delta < 0 {
            self.decrease_key(it, delta);
        }
    }

    // Define a function to detach an item from BPQueue
    pub fn detach(&mut self, it: &mut Dllink) {
        it.detach();
        while self._bucket[self._max].is_empty() {
            self._max -= 1;
        }
    }

    // Define a function to iterate over BPQueue
    pub fn iter(&self) -> BPQueueIterator {
        BPQueueIterator {
            bpq: self,
            curkey: self._max,
            curitem: self._bucket[self._max].iter(),
        }
    }
}

// Define a struct BPQueueIterator
pub struct BPQueueIterator<'a> {
    bpq: &'a BPQueue,
    curkey: usize,
    curitem: std::slice::Iter<'a, Dllink>,
}

impl<'a> Iterator for BPQueueIterator<'a> {
    type Item = &'a Dllink;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curkey > 0 {
            match self.curitem.next() {
                Some(res) => return Some(res),
                None => {
                    self.curkey -= 1;
                    self.curitem = self.bpq._bucket[self.curkey].iter();
                }
            }
        }
        None
    }
}

