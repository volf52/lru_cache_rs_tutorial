// From https://dev.to/seanchen1991/implementing-an-lru-cache-in-rust-33pp

use core::fmt;

use arrayvec::ArrayVec;

#[derive(Debug, Clone, Copy)]
pub struct Entry<T> {
    // Value stored in this entry
    val: T,

    // Index of prev
    prev: usize,

    // Index of next
    next: usize,
}

pub struct LRUCache<T, const CAP: usize> {
    entries: ArrayVec<Entry<T>, CAP>,
    head: usize,
    tail: usize,
    length: usize,
}

impl<T, const C: usize> Default for LRUCache<T, C> {
    fn default() -> Self {
        let cache = LRUCache {
            entries: ArrayVec::<Entry<T>, C>::new(),
            head: 0,
            tail: 0,
            length: 0,
        };

        assert!(cache.entries.capacity() < usize::MAX, "Capacity overflow");

        cache
    }
}

impl<T, const C: usize> Clone for LRUCache<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            head: self.head,
            tail: self.tail,
            length: self.length,
        }
    }
}

impl<T, const C: usize> fmt::Debug for LRUCache<T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LRUCache")
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("entries", &self.entries)
            .finish()
    }
}

pub struct IterMut<'a, T, const C: usize> {
    cache: &'a mut LRUCache<T, C>,
    pos: usize,
    done: bool,
}

impl<'a, T, const C: usize> Iterator for IterMut<'a, T, C>
where
    T: 'a,
{
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let entry = unsafe { &mut *(&mut self.cache.entries[self.pos] as *mut Entry<T>) };
        let index = self.pos;

        if self.pos == self.cache.tail {
            self.done = true;
        }

        self.pos = entry.next;

        Some((index, &mut entry.val))
    }
}

impl<T, const C: usize> LRUCache<T, C>
where
    T: Copy,
{
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.head = 0;
        self.tail = 0;
        self.length = 0;
    }

    fn iter_mut(&mut self) -> IterMut<T, C> {
        IterMut {
            pos: self.head,
            done: self.is_empty(),
            cache: self,
        }
    }

    pub fn items(&mut self) -> Vec<T> {
        let x = self.iter_mut();

        x.map(|(_, x)| x.clone()).collect()
    }

    pub fn front(&self) -> Option<&T> {
        self.entries.get(self.head).map(|e| &e.val)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.entries.get_mut(self.head).map(|e| &mut e.val)
    }

    fn push_front(&mut self, index: usize) {
        if self.entries.len() == 1 {
            self.tail = index;
        } else {
            self.entries[index].next = self.head;
            self.entries[self.head].prev = index;
        }

        self.head = index;
    }

    fn pop_back(&mut self) -> usize {
        let old_tail = self.tail;
        let new_tail = self.entries[old_tail].prev;
        self.tail = new_tail;

        old_tail
    }

    fn remove(&mut self, index: usize) {
        assert!(self.length > 0);

        let prev = self.entries[index].prev;
        let next = self.entries[index].next;

        if index == self.head {
            self.head = next;
        } else {
            self.entries[prev].next = next;
        }

        if index == self.tail {
            self.tail = prev;
        } else {
            self.entries[next].prev = prev;
        }

        self.length -= 1;
    }

    #[inline]
    fn touch_index(&mut self, index: usize) {
        if index != self.head {
            self.remove(index);

            self.length += 1;
            self.push_front(index);
        }
    }

    pub fn touch<F>(&mut self, mut pred: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        match self.iter_mut().find(|&(_, ref x)| pred(x)) {
            Some((i, _)) => {
                self.touch_index(i);
                true
            }
            None => false,
        }
    }

    pub fn lookup<F, R>(&mut self, mut pred: F) -> Option<R>
    where
        F: FnMut(&T) -> Option<R>,
    {
        let mut result = None;

        for (i, entry) in self.iter_mut() {
            if let Some(r) = pred(entry) {
                result = Some((i, r));
                break;
            }
        }

        match result {
            None => None,
            Some((i, r)) => {
                self.touch_index(i);
                Some(r)
            }
        }
    }

    pub fn insert(&mut self, val: T) {
        let entry = Entry {
            val,
            prev: 0,
            next: 0,
        };

        let new_head = if self.length == self.entries.capacity() {
            let last_index = self.pop_back();
            self.entries[last_index] = entry;

            last_index
        } else {
            self.entries.push(entry);
            self.length += 1;

            self.entries.len() - 1
        };

        self.push_front(new_head)
    }

    pub fn fetch<F>(&mut self, pred: F) -> Option<&mut T>
    where
        F: FnMut(&T) -> bool,
    {
        if self.touch(pred) {
            self.front_mut()
        } else {
            None
        }
    }
}
