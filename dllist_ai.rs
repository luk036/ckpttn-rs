struct Dllink<T> {
    next: Option<Box<Dllink<T>>>,
    prev: Option<*mut Dllink<T>>,
    data: T,
}

impl<T> Dllink<T> {
    fn new(data: T) -> Self {
        Dllink {
            next: None,
            prev: None,
            data,
        }
    }

    fn appendleft(&mut self, node: &mut Dllink<T>) {
        node.next = self.next.take();
        node.prev = Some(self as *mut Dllink<T>);
        if let Some(ref mut next) = node.next {
            next.prev = Some(node);
        }
        self.next = Some(Box::new(node));
    }
}

struct DllIterator<'a, T> {
    link: &'a Dllink<T>,
    cur: Option<&'a Dllink<T>>,
}

impl<'a, T> Iterator for DllIterator<'a, T> {
    type Item = &'a Dllink<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur.take() {
            if cur as *const Dllink<T> != self.link as *const Dllink<T> {
                self.cur = cur.next.as_deref();
                return Some(cur);
            }
        }
        None
    }
}

struct Dllist<T> {
    head: Dllink<T>,
}

impl<T> Dllist<T> {
    fn new(data: T) -> Self {
        Dllist {
            head: Dllink::new(data),
        }
    }

    fn appendleft(&mut self, node: &mut Dllink<T>) {
        self.head.appendleft(node);
    }

    fn pop(&mut self) -> Option<Box<Dllink<T>>> {
        self.head.next.take().map(|mut node| {
            self.head.next = node.next.take();
            if let Some(ref mut next) = node.next {
                next.prev = Some(&mut self.head);
            }
            node
        })
    }

    fn popleft(&mut self) -> Option<Box<Dllink<T>>> {
        self.head.next.take().map(|mut node| {
            self.head.next = node.next.take();
            if let Some(ref mut next) = node.next {
                next.prev = Some(&mut self.head);
            }
            node
        })
    }

    fn is_empty(&self) -> bool {
        self.head.next.is_none()
    }

    fn iter(&self) -> DllIterator<T> {
        DllIterator {
            link: &self.head,
            cur: self.head.next.as_deref(),
        }
    }
}

fn test_dllink() {
    let mut L1 = Dllist::new(99);
    let mut L2 = Dllist::new(99);
    let mut d = Box::new(Dllink::new(1));
    let mut e = Box::new(Dllink::new(2));
    let mut f = Box::new(Dllink::new(3));
    L1.appendleft(&mut e);
    assert!(!L1.is_empty());
    L1.appendleft(&mut f);
    L1.append(&mut d);
    L2.append(L1.pop().unwrap());
    L2.append(L1.popleft().unwrap());
    assert!(!L1.is_empty());
    assert!(!L2.is_empty());
    e.detach();
    assert!(L1.is_empty());
    let mut count = 0;
    for _ in L2.iter() {
        count += 1;
    }
    assert_eq!(count, 2);
}

