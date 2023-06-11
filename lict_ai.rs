struct Lict<T> {
    rng: std::ops::Range<usize>,
    lst: Vec<T>,
}

impl<T> Lict<T> {
    fn new(lst: Vec<T>) -> Self {
        let len = lst.len();
        Self {
            rng: 0..len,
            lst,
        }
    }

    fn values(&self) -> std::slice::Iter<'_, T> {
        self.lst.iter()
    }

    fn items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.lst.iter().enumerate()
    }
}

impl<T> std::ops::Index<usize> for Lict<T> {
    type Output = T;

    fn index(&self, key: usize) -> &Self::Output {
        &self.lst[key]
    }
}

impl<T> std::ops::IndexMut<usize> for Lict<T> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.lst[key]
    }
}

impl<T> std::iter::IntoIterator for Lict<T> {
    type Item = usize;
    type IntoIter = std::ops::Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.rng
    }
}

impl<T> std::iter::Iterator for Lict<T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.rng.next()
    }
}

impl<T> std::iter::ExactSizeIterator for Lict<T> {
    fn len(&self) -> usize {
        self.rng.len()
    }
}

struct ShiftArray<T> {
    start: usize,
    lst: Vec<T>,
}

impl<T> ShiftArray<T> {
    fn new(lst: Vec<T>) -> Self {
        Self {
            start: 0,
            lst,
        }
    }

    fn items(&self) -> impl Iterator<Item = (usize, &T)> {
        self.lst.iter().enumerate().map(move |(i, v)| (i + self.start, v))
    }
}

impl<T> std::ops::Index<usize> for ShiftArray<T> {
    type Output = T;

    fn index(&self, key: usize) -> &Self::Output {
        &self.lst[key - self.start]
    }
}

impl<T> std::ops::IndexMut<usize> for ShiftArray<T> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.lst[key - self.start]
    }
}

fn main() {
    let mut a = Lict::new(vec![0; 8]);
    for i in &mut a {
        a[i] = i * i;
    }
    for (i, v) in a.items() {
        println!("{}: {}", i, v);
    }
    println!("{}", a.contains(&3));

    let mut b = ShiftArray::new(vec![0; 8]);
    for i in 0..8 {
        b[i] = i * i;
    }
    for (i, v) in b.items() {
        println!("{}: {}", i, v);
    }
    println!("{}", b[3]);
}

