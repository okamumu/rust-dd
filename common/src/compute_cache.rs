//! Direct-mapped, lossy computed table for DD `apply` memoization (CUDD-style).
//!
//! Shared by the BDD and MDD engines. The operation cache is only a *hint*: a
//! miss costs a recomputation, never a wrong answer. So instead of a growing
//! `HashMap` (probing + periodic rehash on every `put`, which dominates `apply`
//! on workloads of many tiny operations), this is a fixed-power-of-two array
//! indexed by `hash(key) & mask`, with a single slot per bucket that is simply
//! overwritten on collision.
//!
//!   get = one array load + key compare;  put = one array store.
//!
//! Each slot is `[k0, k1, k2, val]` of `u32`. `val == EMPTY` (`u32::MAX`) marks
//! an empty slot; node ids never reach `u32::MAX`, so it is an unambiguous
//! sentinel. Keys are u32-narrowed by the manager (op code + two node ids).

const EMPTY: u32 = u32::MAX;

/// Initial table size (2^14 slots = 256 KiB); grows on demand.
const INIT_BITS: u32 = 14;
/// Hard ceiling on growth (2^24 slots = 256 MiB); beyond this it stays lossy.
const MAX_BITS: u32 = 24;

#[derive(Debug)]
pub struct ComputeCache {
    slots: Vec<[u32; 4]>,
    mask: usize,
    len: usize,
}

impl Default for ComputeCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeCache {
    pub fn new() -> Self {
        Self::with_bits(INIT_BITS)
    }

    fn with_bits(bits: u32) -> Self {
        let n = 1usize << bits;
        Self { slots: vec![[0, 0, 0, EMPTY]; n], mask: n - 1, len: 0 }
    }

    #[inline]
    fn index(&self, k0: u32, k1: u32, k2: u32) -> usize {
        // Cheap multiplicative mix over the three key words.
        let mut h = k0 as u64;
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k1 as u64);
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k2 as u64);
        h ^= h >> 29;
        (h as usize) & self.mask
    }

    #[inline]
    pub fn get(&self, k0: u32, k1: u32, k2: u32) -> Option<u32> {
        let s = &self.slots[self.index(k0, k1, k2)];
        if s[3] != EMPTY && s[0] == k0 && s[1] == k1 && s[2] == k2 {
            Some(s[3])
        } else {
            None
        }
    }

    #[inline]
    pub fn put(&mut self, k0: u32, k1: u32, k2: u32, val: u32) {
        // Keep the load factor under 3/4 so collisions (= lost memo entries)
        // stay rare, by doubling (and rehashing) up to the cap.
        if self.len * 4 >= self.slots.len() * 3 {
            self.grow();
        }
        self.put_nogrow(k0, k1, k2, val);
    }

    #[inline]
    fn put_nogrow(&mut self, k0: u32, k1: u32, k2: u32, val: u32) {
        let i = self.index(k0, k1, k2);
        let s = &mut self.slots[i];
        if s[3] == EMPTY {
            self.len += 1;
        }
        *s = [k0, k1, k2, val];
    }

    fn grow(&mut self) {
        let bits = self.slots.len().trailing_zeros() + 1;
        if bits > MAX_BITS {
            return; // at the ceiling: stay this size, tolerate more collisions
        }
        let old = std::mem::replace(self, Self::with_bits(bits));
        for s in &old.slots {
            if s[3] != EMPTY {
                self.put_nogrow(s[0], s[1], s[2], s[3]);
            }
        }
    }

    /// Drop entries whose operands or result reference a non-live slot.
    /// Mirrors the pre-gc `HashMap::retain` guard: an id may only be reused for
    /// a different node after gc, so a stale entry pointing at a reclaimed slot
    /// must be invalidated. `k1`/`k2` are operand ids (`k2 == 0` for unary ops,
    /// the always-live zero terminal); `val` is the result id.
    pub fn retain_live(&mut self, live: &[bool]) {
        for s in &mut self.slots {
            if s[3] != EMPTY {
                let (f, g, v) = (s[1] as usize, s[2] as usize, s[3] as usize);
                if !(live[f] && live[g] && live[v]) {
                    s[3] = EMPTY;
                    self.len -= 1;
                }
            }
        }
    }

    /// Like [`retain_live`](Self::retain_live) but for caches whose `k0` word is
    /// also a node id (e.g. a ternary `ite(f,g,h)` cache keyed `(f,g,h)`), not an
    /// operation code. All three key words and the result are liveness-checked.
    pub fn retain_live3(&mut self, live: &[bool]) {
        for s in &mut self.slots {
            if s[3] != EMPTY {
                let (f, g, h, v) = (s[0] as usize, s[1] as usize, s[2] as usize, s[3] as usize);
                if !(live[f] && live[g] && live[h] && live[v]) {
                    s[3] = EMPTY;
                    self.len -= 1;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for s in &mut self.slots {
            s[3] = EMPTY;
        }
        self.len = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
