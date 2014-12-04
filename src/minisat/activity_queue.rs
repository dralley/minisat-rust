use super::index_map::{HasIndex, IndexMap};


pub struct ActivityQueue<V : HasIndex> {
    heap     : Vec<V>,
    indices  : IndexMap<V, uint>,
    activity : IndexMap<V, f64>,
}

impl<V : HasIndex> ActivityQueue<V> {
    pub fn new() -> ActivityQueue<V> {
        ActivityQueue {
            heap     : Vec::new(),
            indices  : IndexMap::new(),
            activity : IndexMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.indices.clear();
    }

    pub fn len(&self) -> uint {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn contains(&self, v : &V) -> bool {
        self.indices.contains_key(v)
    }
}

impl<V : HasIndex + Clone> ActivityQueue<V> {
    pub fn setActivity(&mut self, v : &V, new : f64) {
        match self.activity.insert(v, new) {
            None      => {}
            Some(old) => {
                if self.indices.contains_key(v) {
                    let i = self.indices[*v];
                    if new > old { self.sift_up(i) } else { self.sift_down(i) }
                }
            }
        }
    }

    pub fn bumpActivity(&mut self, v : &V, delta : f64) {
        let new = self.activity[*v] + delta;
        self.setActivity(v, new);
    }

    pub fn insert(&mut self, v : V) {
        if !self.contains(&v) {
            if !self.activity.contains_key(&v) {
                self.activity.insert(&v, 0.0);
            }

            let i = self.heap.len();
            self.indices.insert(&v, i);
            self.heap.push(v);
            self.sift_up(i);
        }
    }

    pub fn pop(&mut self) -> Option<V>
    {
        match self.heap.len() {
            0 => { None }
            1 => {
                let h = self.heap.pop().unwrap();
                self.indices.remove(&h);
                Some(h)
            }
            _ => {
                let h = self.heap[0].clone();
                self.indices.remove(&h);

                let t = self.heap.pop().unwrap();
                self.set(0, t);

                self.sift_down(0);
                Some(h)
            }
        }
    }


    fn sift_up(&mut self, mut i : uint) {
        let x = self.heap[i].clone();
        while i > 0 {
            let pi = (i - 1) >> 1;
            let p = self.heap[pi].clone();
            if self.activity[x] > self.activity[p] {
                self.set(i, p);
                i = pi;
            } else {
                break
            }
        }
        self.set(i, x);
    }

    fn sift_down(&mut self, mut i : uint) {
        let x = self.heap[i].clone();
        let len = self.heap.len();
        loop {
            let li = i + i + 1;
            if li >= len { break; }
            let ri = i + i + 2;
            let ci = if ri < len && self.activity[self.heap[ri]] > self.activity[self.heap[li]] { ri } else { li };
            let c = self.heap[ci].clone();
            if self.activity[c] <= self.activity[x] { break; }
            self.set(i, c);
            i = ci;
        }
        self.set(i, x);
    }

    #[inline]
    fn set(&mut self, i : uint, v : V) {
        self.indices.insert(&v, i);
        self.heap[i] = v;
    }
}

impl<V : HasIndex> Index<uint, V> for ActivityQueue<V> {
    #[inline]
    fn index(&self, i : &uint) -> &V {
        self.heap.index(i)
    }
}
