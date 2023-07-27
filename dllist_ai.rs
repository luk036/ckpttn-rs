struct Dllink<T> {
    next: Option<*mut Dllink<T>>,
    prev: Option<*mut Dllink<T>>,
    data: T,
}

impl<T> Dllink<T> {
    fn new(data: T) -> Self {
        Self {
            next: None,
            prev: None,
            data,
        }
    }

    fn is_locked(&self) -> bool {
        self.next.is_none()
    }

    fn lock(&mut self) {
        self.next = None;
    }

    fn appendleft(&mut self, node: &mut Dllink<T>) {
        node.next = self.next;
        self.next.unwrap().prev = Some(node);
        self.next = Some(node);
        node.prev = Some(self);
    }

    fn append(&mut self, node: &mut Dllink<T>) {
        node.prev = self.prev;
        self.prev.unwrap().next = Some(node);
        self.prev = Some(node);
        node.next = Some(self);
    }

    fn popleft(&mut self) -> &mut Dllink<T> {
        let res = self.next.unwrap();
        self.next = res.next;
        self.next.unwrap().prev = Some(self);
        unsafe { &mut *res }
    }

    fn pop(&mut self) -> &mut Dllink<T> {
        let res = self.prev.unwrap();
        self.prev = res.prev;
        self.prev.unwrap().next = Some(self);
        unsafe { &mut *res }
    }

    fn detach(&mut self) {
        assert!(self.next.is_some());
        let n = self.next.unwrap();
        let p = self.prev.unwrap();
        p.next = Some(n);
        n.prev = Some(p);
    }
}

struct Dllist<T> {
    head: Dllink<T>,
}

impl<T> Dllist<T> {
    fn new(data: T) -> Self {
        Self {
            head: Dllink::new(data),
        }
    }

    fn is_empty(&self) -> bool {
        self.head.next.is_none()
    }

    fn clear(&mut self) {
        self.head.next = None;
        self.head.prev = None;
    }

    fn appendleft(&mut self, node: &mut Dllink<T>) {
        self.head.appendleft(node);
    }

    fn append(&mut self, node: &mut Dllink<T>) {
        self.head.append(node);
    }

    fn popleft(&mut self) -> &mut Dllink<T> {
        self.head.popleft()
    }

    fn pop(&mut self) -> &mut Dllink<T> {
        self.head.pop()
    }
}

struct DllIterator<'a, T> {
    link: *mut Dllink<T>,
    cur: *mut Dllink<T>,
    marker: std::marker::PhantomData<&'a mut Dllist<T>>,
}

impl<'a, T> DllIterator<'a, T> {
    fn new(head: &'a mut Dllink<T>) -> Self {
        Self {
            link: head as *mut Dllink<T>,
            cur: head.next.unwrap() as *mut Dllink<T>,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for DllIterator<'a, T> {
    type Item = &'a mut Dllink<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.link {
            return None;
        }
        let res = unsafe { &mut *self.cur };
        self.cur = res.next.unwrap() as *mut Dllink<T>;
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dllink() {
        let mut l1 = Dllist::new(());
        let mut l2 = Dllist::new(());
        let mut d = Dllink::new(());
        let mut e = Dllink::new(());
        let mut f = Dllink::new(());

        l1.appendleft(&mut e);
        assert!(!l1.is_empty());

        l1.appendleft(&mut f);
        l1.append(&mut d);
        l2.append(l1.pop());
        l2.append(l1.popleft());
        assert!(!l1.is_empty());
        assert!(!l2.is_empty());
        e.detach();
        assert!(l1.is_empty());

        let mut count = 0;
        for _ in DllIterator::new(&mut l2.head) {
            count += 1;
        }
        assert_eq!(count, 2);
    }
}
