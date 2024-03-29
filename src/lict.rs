/// The `Lict` struct is a generic type in Rust that represents a list with a range and a vector.
///
/// Properties:
///
/// * `rng`: The `rng` property is a range that represents the indices of the elements in the `lst`
/// vector. It is of type `std::ops::Range<usize>`, which is a range of `usize` values. This range is
/// used to iterate over the elements in the `lst` vector
/// * `lst`: The `lst` property is a vector that stores elements of type `T`. It is used to store the
/// elements of the `Lict` struct.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lict<T> {
    pub rng: std::ops::Range<usize>,
    pub lst: Vec<T>,
}

impl<T> Lict<T> {
    /// The function `new` creates a new instance of a struct with a range and a vector.
    ///
    /// Arguments:
    ///
    /// * `lst`: A vector of elements of type T.
    ///
    /// Returns:
    ///
    /// The `new` function is returning an instance of the struct that it is defined in.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::Lict;
    ///
    /// assert_eq!(Lict::new(vec![1, 2, 3]), Lict { rng: 0..3, lst: vec![1, 2, 3] });
    /// ```
    pub fn new(lst: Vec<T>) -> Self {
        let len = lst.len();
        Self { rng: 0..len, lst }
    }

    /// The `values` function returns an iterator over the values in a list.
    ///
    /// Returns:
    ///
    /// The `values` function is returning an iterator over the elements of the `lst` field of the struct.
    pub fn values(&self) -> std::slice::Iter<'_, T> {
        self.lst.iter()
    }

    /// The function `items` returns an iterator that yields the index and reference to each element in
    /// the `lst` vector.
    pub fn items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.lst.iter().enumerate()
    }

    /// The function checks if a given key is within the range of the rng vector.
    ///
    /// Arguments:
    ///
    /// * `key`: The `key` parameter is of type `usize`, which represents an unsigned integer. It is
    /// used to specify the index of an element in the `rng` array.
    ///
    /// Returns:
    ///
    /// A boolean value is being returned.
    ///
    /// # Examples:
    ///
    /// ```
    /// use ckpttn_rs::lict::Lict;
    ///
    /// assert_eq!(Lict::new(vec![1, 2, 3]).contains(0), true);
    /// assert_eq!(Lict::new(vec![1, 2, 3]).contains(3), false);
    /// ```
    pub fn contains(&self, key: usize) -> bool {
        key < self.rng.len()
    }
}

impl<T> std::ops::Index<usize> for Lict<T> {
    type Output = T;

    /// The `index` function returns a reference to an element in a list based on the given key.
    ///
    /// Arguments:
    ///
    /// * `key`: The `key` parameter is of type `usize`. It represents the index of the element in the
    /// `lst` field that you want to access.
    ///
    /// Returns:
    ///
    /// The method `index` is returning a reference to an element in the `lst` field of the struct. The
    /// type of the returned value is `&Self::Output`, which is a reference to the output type
    /// associated with the struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::Lict;
    ///
    /// let lict = Lict::new(vec![1, 2, 3]);
    /// assert_eq!(lict[0], 1);
    /// assert_eq!(lict[1], 2);
    /// ```
    fn index(&self, key: usize) -> &Self::Output {
        &self.lst[key]
    }
}

impl<T> std::ops::IndexMut<usize> for Lict<T> {
    /// The function `index_mut` returns a mutable reference to an element in a list based on the given
    /// key.
    ///
    /// Arguments:
    ///
    /// * `key`: The `key` parameter is of type `usize`. It represents the index of the element in the
    /// `lst` field that you want to access and modify.
    ///
    /// Returns:
    ///
    /// A mutable reference to an element in the `lst` vector at the given `key` index.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::Lict;
    ///
    /// let mut lict = Lict::new(vec![1, 2, 3]);
    /// lict[0] = 10;
    /// assert_eq!(lict[0], 10);
    /// ```
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.lst[key]
    }
}

// impl<T> std::iter::IntoIterator for Lict<T> {
//     type Item = usize;
//     type IntoIter = std::ops::Range<usize>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.rng
//     }
// }

impl<T> std::iter::Iterator for Lict<T> {
    type Item = usize;

    /// The function `next` returns the next item from the random number generator.
    ///
    /// Returns:
    ///
    /// The `next` function is returning an `Option<Self::Item>`.
    fn next(&mut self) -> Option<Self::Item> {
        self.rng.next()
    }
}

impl<T> std::iter::ExactSizeIterator for Lict<T> {
    /// The function returns the length of a given object.
    ///
    /// Returns:
    ///
    /// The `len` function is returning the length of the `rng` field of the struct.
    fn len(&self) -> usize {
        self.rng.len()
    }
}

/// The ShiftArray type represents an array that can be shifted to the left or right without copying or
/// moving its elements.
///
/// Properties:
///
/// * `start`: The `start` property represents the index of the first element in the `ShiftArray`. It
/// indicates the starting point from which elements are accessed or shifted.
/// * `lst`: The `lst` property is a vector that holds the elements of the `ShiftArray`. It is of type
/// `Vec<T>`, where `T` is a generic type parameter that can be replaced with any type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShiftArray<T> {
    pub start: usize,
    pub lst: Vec<T>,
}

impl<T> ShiftArray<T> {
    /// The function "new" initializes a new instance of a struct with a starting index of 0 and a given
    /// list.
    ///
    /// Arguments:
    ///
    /// * `lst`: The `lst` parameter is a `Vec<T>`, which is a vector of elements of type `T`.
    ///
    /// Returns:
    ///
    /// The `new` function is returning an instance of the struct that it is defined in.
    pub fn new(lst: Vec<T>) -> Self {
        Self { start: 0, lst }
    }

    /// The function sets the start value of a variable.
    ///
    /// Arguments:
    ///
    /// * `start`: The `start` parameter is of type `usize`, which represents an unsigned integer that
    /// can hold the size of any object in memory. It is used to set the value of the `start` field in
    /// the struct or object that this method belongs to.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::ShiftArray;
    ///
    /// let mut shift_array = ShiftArray::new(vec![1, 2, 3]);
    /// shift_array.set_start(1);
    /// assert_eq!(shift_array.start, 1);
    /// ```
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    /// The `items` function returns an iterator that yields the index and reference to each element in
    /// the `lst` vector, with the index adjusted by the `start` value.
    pub fn items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.lst
            .iter()
            .enumerate()
            .map(move |(i, v)| (i + self.start, v))
    }
}

impl<T> std::ops::Index<usize> for ShiftArray<T> {
    type Output = T;

    /// The `index` function returns a reference to an element in a list based on a given key.
    ///
    /// Arguments:
    ///
    /// * `key`: The `key` parameter is of type `usize` and represents the index of the element to be
    /// accessed in the `lst` field.
    ///
    /// Returns:
    ///
    /// The method `index` returns a reference to an element of `self.lst` at the specified index `key -
    /// self.start`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::ShiftArray;
    /// let mut shift_array = ShiftArray::new(vec![1, 2, 3]);
    /// assert_eq!(shift_array[2], 3);
    /// shift_array.set_start(1);
    /// assert_eq!(shift_array[2], 2);
    /// ```
    fn index(&self, key: usize) -> &Self::Output {
        &self.lst[key - self.start]
    }
}

impl<T> std::ops::IndexMut<usize> for ShiftArray<T> {
    /// The function `index_mut` returns a mutable reference to an element in a list based on a given
    /// key.
    ///
    /// Arguments:
    ///
    /// * `key`: The `key` parameter is of type `usize` and represents the index of the element to be
    /// accessed in the `lst` vector.
    ///
    /// Returns:
    ///
    /// A mutable reference to an element in the `lst` vector, located at the index `key - self.start`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ckpttn_rs::lict::ShiftArray;
    /// let mut shift_array = ShiftArray::new(vec![1, 2, 3]);
    /// assert_eq!(shift_array[2], 3);
    /// shift_array.set_start(1);
    /// assert_eq!(shift_array[2], 2);
    /// shift_array[2] = 4;
    /// assert_eq!(shift_array[2], 4);
    /// ```
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.lst[key - self.start]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lict() {
        let a = Lict::new(vec![0; 8]);
        // for i in &mut a {
        //     a[i] = i * i;
        // }
        for (i, v) in a.items() {
            println!("{}: {}", i, v);
        }
        assert!(a.contains(3));
    }

    #[test]
    fn test_shift_array() {
        let mut a = ShiftArray::new(vec![2, 3, 5, 7, 11]);
        a.set_start(5);
        assert_eq!(a[6], 3);
        for (i, v) in a.items() {
            println!("{}: {}", i, v);
        }
    }
}

// fn main() {
//     let mut a = Lict::new(vec![0; 8]);
//     for i in &mut a {
//         a[i] = i * i;
//     }
//     for (i, v) in a.items() {
//         println!("{}: {}", i, v);
//     }
//     println!("{}", a.contains(&3));

//     let mut b = ShiftArray::new(vec![0; 8]);
//     for i in 0..8 {
//         b[i] = i * i;
//     }
//     for (i, v) in b.items() {
//         println!("{}: {}", i, v);
//     }
//     println!("{}", b[3]);
// }
