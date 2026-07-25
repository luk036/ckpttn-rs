/// ShiftArray container with shifted index access.
///
/// Ported from C++ `ShiftArray` in `array_like.hpp`.
/// Allows accessing elements using shifted indices relative to a base offset.
pub struct ShiftArray<T> {
    data: Vec<T>,
    start: usize,
}

impl<T: Clone + Default> Default for ShiftArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default> ShiftArray<T> {
    /// Create a new empty ShiftArray.
    pub fn new() -> Self {
        ShiftArray {
            data: Vec::new(),
            start: 0,
        }
    }

    /// Create a ShiftArray with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ShiftArray {
            data: Vec::with_capacity(capacity),
            start: 0,
        }
    }

    /// Set the start offset for index shifting.
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    /// Get the current start offset.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Resize the underlying storage.
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.data.resize(new_len, value);
    }

    /// Get the length of the underlying storage.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the storage is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the storage.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Raw index (without shift) access.
    pub fn raw_index(&self, index: usize) -> usize {
        index - self.start
    }
}

use std::ops::{Index, IndexMut};

impl<T: Clone + Default> Index<usize> for ShiftArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.data[index - self.start]
    }
}

impl<T: Clone + Default> IndexMut<usize> for ShiftArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index - self.start]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_array_new() {
        let arr: ShiftArray<u32> = ShiftArray::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.start(), 0);
    }

    #[test]
    fn test_shift_array_set_start() {
        let mut arr: ShiftArray<u32> = ShiftArray::new();
        arr.set_start(5);
        assert_eq!(arr.start(), 5);
    }

    #[test]
    fn test_shift_array_resize_and_access() {
        let mut arr = ShiftArray::new();
        arr.set_start(10);
        arr.resize(5, 0u32);
        assert_eq!(arr.len(), 5);
        arr[10] = 1;
        arr[11] = 2;
        arr[12] = 3;
        assert_eq!(arr[10], 1);
        assert_eq!(arr[11], 2);
        assert_eq!(arr[12], 3);
    }

    #[test]
    fn test_shift_array_index_mut() {
        let mut arr = ShiftArray::new();
        arr.set_start(100);
        arr.resize(3, 0u32);
        arr[100] = 10;
        arr[101] = 20;
        arr[102] = 30;
        assert_eq!(arr[100], 10);
        assert_eq!(arr[101], 20);
        assert_eq!(arr[102], 30);
    }

    #[test]
    fn test_shift_array_clear() {
        let mut arr: ShiftArray<u32> = ShiftArray::new();
        arr.set_start(5);
        arr.resize(3, 0u32);
        assert!(!arr.is_empty());
        arr.clear();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_shift_array_raw_index() {
        let mut arr: ShiftArray<u32> = ShiftArray::new();
        arr.set_start(10);
        arr.resize(5, 0u32);
        assert_eq!(arr.raw_index(10), 0);
        assert_eq!(arr.raw_index(14), 4);
    }
}
