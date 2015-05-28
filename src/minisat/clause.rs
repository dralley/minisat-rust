use std::fmt;
use std::ops::{Index, IndexMut};
use super::literal::{Lit};
use super::index_map::{HasIndex};


pub type ClauseRef = usize;


struct ClauseHeader {
    mark      : u32,
    learnt    : bool,
    has_extra : bool,
    reloced   : bool,
    size      : usize
}

pub struct Clause {
    header   : ClauseHeader,
    data     : Vec<Lit>,
    data_act : f32,
    data_abs : u32,
    data_rel : Option<ClauseRef>,
}

impl Clause {
    #[inline]
    pub fn len(&self) -> usize {
        self.header.size
    }

    #[inline]
    pub fn mark(&self) -> u32 {
        self.header.mark
    }

    #[inline]
    pub fn learnt(&self) -> bool {
        self.header.learnt
    }

    #[inline]
    pub fn reloced(&self) -> bool {
        self.header.reloced
    }

    #[inline]
    pub fn activity(&self) -> f64 {
        assert!(self.header.has_extra);
        self.data_act as f64
    }

    #[inline]
    pub fn setActivity(&mut self, act : f64) {
        assert!(self.header.has_extra);
        self.data_act = act as f32;
    }

    #[inline]
    pub fn setMark(&mut self, m : u32) {
        self.header.mark = m
    }

    #[inline]
    pub fn retainSuffix<F : Fn(&Lit) -> bool>(&mut self, base : usize, f : F) {
        let mut i = base;
        while i < self.header.size {
            if f(&self.data[i]) {
                i += 1
            } else {
                self.header.size -= 1;
                self.data[i] = self.data[self.header.size];
            }
        }
    }

    fn calcAbstraction(&mut self) {
        assert!(self.header.has_extra);
        let mut abstraction : u32 = 0;
        for i in 0 .. self.header.size {
            abstraction |= 1 << (self.data[i].var().toIndex() & 31);
        }
        self.data_abs = abstraction; //data[header.size].abs = abstraction;
    }
}

impl Index<usize> for Clause {
    type Output = Lit;

    #[inline]
    fn index<'a>(&'a self, index : usize) -> &'a Lit {
        self.data.index(index)
    }
}

impl IndexMut<usize> for Clause {
    #[inline]
    fn index_mut<'a>(&'a mut self, index : usize) -> &'a mut Lit {
        self.data.index_mut(index)
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "("));
        for i in 0 .. self.len() {
            try!(write!(f, "{}{}", if i == 0 { "" } else { " | " }, self[i]));
        }
        write!(f, ")")
    }
}


fn clauseSize(size : usize, has_extra : bool) -> usize {
    4 * (1 + size + (has_extra as usize))
}


pub struct ClauseAllocator {
    clauses : Vec<Box<Clause>>,
    size    : usize,
    wasted  : usize,
}

impl ClauseAllocator {
    pub fn new() -> ClauseAllocator {
        ClauseAllocator {
            clauses : Vec::new(),
            size    : 0,
            wasted  : 0,
        }
    }

    pub fn alloc(&mut self, ps : &Vec<Lit>, learnt : bool) -> ClauseRef {
        let use_extra = learnt;
        let mut c = Box::new(Clause {
            header   : ClauseHeader { mark : 0, learnt : learnt, has_extra : use_extra, reloced : false, size : ps.len() },
            data     : ps.clone(),
            data_act : 0.0,
            data_abs : 0,
            data_rel : None,
        });

        if c.header.has_extra {
            if c.header.learnt {
                c.data_act = 0.0;
            } else {
                c.calcAbstraction();
            };
        }

        let len = self.clauses.len();
        self.clauses.push(c);
        self.size += clauseSize(ps.len(), use_extra);

        len
    }

    fn allocCopy(&mut self, that : &Clause) -> ClauseRef {
        let mut tmp = Vec::new();
        for i in 0 .. that.len() {
            tmp.push(that[i]);
        }
        self.alloc(&tmp, that.header.learnt)
    }

    pub fn free(&mut self, cr : ClauseRef) {
        let size = {
            let c = &self[cr];
            clauseSize(c.header.size, c.header.has_extra)
        };
        self.wasted += size;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn wasted(&self) -> usize {
        self.wasted
    }

    pub fn numberOfClauses(&self) -> usize {
        self.clauses.len()
    }

    pub fn reloc(&mut self, to : &mut ClauseAllocator, cr : &mut ClauseRef) {
        let c = &mut self[*cr];
        if c.header.reloced {
            *cr = c.data_rel.unwrap();
        } else {
            *cr = to.allocCopy(c);
            c.header.reloced = true;
            c.data_rel = Some(*cr);
        }
    }
}

impl Index<ClauseRef> for ClauseAllocator {
    type Output = Clause;

    #[inline]
    fn index<'a>(&'a self, index : ClauseRef) -> &'a Clause {
        assert!(index < self.clauses.len());
        &(*self.clauses[index])
    }
}

impl IndexMut<ClauseRef> for ClauseAllocator {
    #[inline]
    fn index_mut<'a>(&'a mut self, index : ClauseRef) -> &'a mut Clause {
        assert!(index < self.clauses.len());
        &mut(*self.clauses[index])
    }
}
