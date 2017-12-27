use std::mem;
use sat::formula::{Lit, LitVec, Var};
use sat::formula::assignment::Assignment;
use sat::formula::clause::*;


#[derive(Clone, Copy, Debug)]
struct Watcher {
    pub cref: ClauseRef,
    pub blocker: Lit,
}


#[derive(Default, Debug)]
struct WatchesLine {
    watchers: Vec<Watcher>,
    dirty: bool,
}


pub struct Watches {
    watches: LitVec<WatchesLine>,
    pub propagations: u64,
}

impl Watches {
    pub fn new() -> Self {
        Watches {
            watches: LitVec::new(),
            propagations: 0,
        }
    }

    pub fn init_var(&mut self, v: Var) {
        self.watches.init(v.pos_lit());
        self.watches.init(v.neg_lit());
    }

    pub fn try_clear_var(&mut self, _: Var) {}

    pub fn watch_clause(&mut self, c: &Clause, cr: ClauseRef) {
        let (c0, c1) = c.head_pair();
        self.watches[!c0].watchers.push(Watcher {
            cref: cr,
            blocker: c1,
        });
        self.watches[!c1].watchers.push(Watcher {
            cref: cr,
            blocker: c0,
        });
    }

    pub fn unwatch_clause_strict(&mut self, c: &Clause, cr: ClauseRef) {
        let (c0, c1) = c.head_pair();
        self.watches[!c0].watchers.retain(|w| w.cref != cr);
        self.watches[!c1].watchers.retain(|w| w.cref != cr);
    }

    pub fn unwatch_clause_lazy(&mut self, c: &Clause) {
        let (c0, c1) = c.head_pair();
        self.watches[!c0].dirty = true;
        self.watches[!c1].dirty = true;
    }

    // Description:
    //   Propagates all enqueued facts. If a conflict arises, the conflicting clause is returned,
    //   otherwise CRef_Undef.
    //
    //   Post-conditions:
    //     * the propagation queue is empty, even if there was a conflict.
    pub fn propagate(
        &mut self,
        ca: &mut ClauseAllocator,
        assigns: &mut Assignment,
    ) -> Option<ClauseRef> {
        let mut confl = None;
        while let Some(p) = assigns.dequeue() {
            self.propagations += 1;
            let false_lit = !p;

            {
                let ref mut line = self.watches[p];
                if line.dirty {
                    line.watchers.retain(|w| !ca.is_deleted(w.cref));
                    line.dirty = false;
                }
            }

            unsafe {
                let p_watches = &mut self.watches[p].watchers as *mut Vec<Watcher>;

                let begin = (*p_watches).as_mut_ptr();
                let end = begin.offset((*p_watches).len() as isize);

                let mut head = begin;
                let mut tail = begin;
                while head < end {
                    let pwi = *head;
                    head = head.offset(1);

                    if assigns.is_assigned_pos(pwi.blocker) {
                        *tail = pwi;
                        tail = tail.offset(1);
                        continue;
                    }

                    let c = ca.edit(pwi.cref);
                    if c.head() == false_lit {
                        c.swap(0, 1);
                    }
                    //assert!(c[1] == false_lit);

                    // If 0th watch is true, then clause is already satisfied.
                    let cw = Watcher {
                        cref: pwi.cref,
                        blocker: c.head(),
                    };
                    if cw.blocker != pwi.blocker && assigns.is_assigned_pos(cw.blocker) {
                        *tail = cw;
                        tail = tail.offset(1);
                        continue;
                    }

                    // Look for new watch:
                    match c.pull_literal(1, |lit| !assigns.is_assigned_neg(lit)) {
                        Some(lit) => {
                            self.watches[!lit].watchers.push(cw);
                        }

                        // Did not find watch -- clause is unit under assignment:
                        None => {
                            *tail = cw;
                            tail = tail.offset(1);

                            if assigns.is_assigned_neg(cw.blocker) {
                                assigns.dequeue_all();

                                // Copy the remaining watches:
                                while head < end {
                                    *tail = *head;
                                    head = head.offset(1);
                                    tail = tail.offset(1);
                                }

                                confl = Some(cw.cref);
                            } else {
                                assigns.assign_lit(cw.blocker, Some(cw.cref));
                            }
                        }
                    }
                }

                (*p_watches)
                    .truncate(((tail as usize) - (begin as usize)) / mem::size_of::<Watcher>());
            }
        }

        confl
    }

    pub fn reloc_gc(&mut self, from: &mut ClauseAllocator, to: &mut ClauseAllocator) {
        for line in self.watches.iter_mut() {
            line.dirty = false;
            line.watchers.retain(|w| !from.is_deleted(w.cref));
            for w in line.watchers.iter_mut() {
                w.cref = from.reloc_to(to, w.cref).unwrap();
            }
        }
    }
}
