use std::collections::LinkedList;
type Item = LinkedList<Vec<i32>>;
const SENTINEL: Item = LinkedList::new();
struct BPQueue {
    _bucket: Vec<LinkedList<Vec<i32>>>,
    _max: usize,
    _offset: i32,
    _high: i32,
}
impl BPQueue {
    fn new(a: i32, b: i32) -> Self {
        assert!(a <= b);
        let offset = a - 1;
        let high = b - offset;
        let mut bucket = Vec::with_capacity((high + 1) as usize);
        for i in 0..=high {
            bucket.push(LinkedList::new());
            bucket[i as usize].push_back(vec![i as i32, 4848]);
        }
        bucket[0].push_back(SENTINEL);
        BPQueue {
            _bucket: bucket,
            _max: 0,
            _offset: offset,
            _high: high,
        }
    }
    fn clear(&mut self) {
        while self._max > 0 {
            self._bucket[self._max].clear();
            self._max -= 1;
        }
    }
    fn append(&mut self, mut it: Item, k: i32) {
        assert!(k > self._offset);
        it.front_mut().unwrap()[0] = k - self._offset;
        if self._max < it.front().unwrap()[0] as usize {
            self._max = it.front().unwrap()[0] as usize;
        }
        self._bucket[it.front().unwrap()[0] as usize].push_back(it);
    }
}
impl IntoIterator for BPQueue {
    type Item = Item;
    type IntoIter = BPQueueIterator;
    fn into_iter(self) -> Self::IntoIter {
        BPQueueIterator {
            bpq: self,
            curkey: self._max as i32,
            curitem: self.bpq._bucket[self.bpq._max].iter(),
        }
    }
}
struct BPQueueIterator {
    bpq: BPQueue,
    curkey: i32,
    curitem: std::collections::linked_list::Iter<'static, Vec<i32>>,
}
impl Iterator for BPQueueIterator {
    type Item = Item;
    fn next(&mut self) -> Option<Self::Item> {
        while self.curkey > 0 {
            match self.curitem.next() {
                Some(res) => return Some(res.clone()),
                None => {
                    self.curkey -= 1;
                    self.curitem = self.bpq._bucket[self.curkey as usize].iter();
                }
            }
        }
        None
    }
}
