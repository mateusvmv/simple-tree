#![feature(coroutines, coroutine_trait, iter_from_coroutine)]
use std::{iter, mem, ops::RangeBounds};
#[cfg(test)]
mod tests;

const A: usize = 11;
const B: usize = A*2+1;

#[derive(Debug, Clone)]
pub struct SimpleTree<K, V = ()>(Vec<(K, V)>, Option<Vec<SimpleTree<K, V>>>);
impl<K, V> Default for SimpleTree<K, V> {
	fn default() -> Self {
		Self(Default::default(), Default::default())
	}
}
impl<K: Ord, V> SimpleTree<K, V> {
	fn split(ks: &mut Vec<(K, V)>, cs: &mut Vec<Self>, i: usize) {
		let l = SimpleTree(cs[i].0.drain(A+1..).collect(), cs[i].1.as_mut().map(|v| v.drain(A+1..).collect()));
		cs.insert(i+1, l);
		ks.insert(i, cs[i].0.remove(A));
	}
	fn merge(ks: &mut Vec<(K, V)>, cs: &mut Vec<Self>, i: usize) {
		let [l, r] = &mut cs[i..=i+1] else { panic!() };
		let (k, j) = if r.0.len() == A { (A, A + 1) } else { (r.0.len() - A, r.0.len() - A) };
		l.0.push(mem::replace(&mut ks[i], r.0.remove(k-1)));
		l.0.extend(r.0.drain(..k-1));
		if let (Some(l), Some(r)) = (l.1.as_mut(), r.1.as_mut()) {
			l.extend(r.drain(..j));
		}
		if r.0.is_empty() {
			l.0.push(ks.remove(i));
            cs.remove(i+1);
        }
	}
	pub fn insert(&mut self, key: K, val: V) {
		if self.0.len() == B {
			let c = mem::take(self);
			Self::split(&mut self.0, self.1.insert(vec![c]), 0);
		}
		let mut i = self.0.partition_point(|(k, _)| k < &key);
		if let Some(c) = &mut self.1 {
			if c[i].0.len() == B {
				Self::split(&mut self.0, c, i);
				if self.0[i].0 < key { i += 1 };
			}
			c[i].insert(key, val)
		} else {
			self.0.insert(i, (key, val))
		}
	}
	pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
		let Some(cs) = self.1.as_mut() else {
			return self.0.binary_search_by(|(k,_)| k.cmp(key)).ok()
				.map(|i| self.0.remove(i));
		};
		let mut i = self.0.partition_point(|(k,_)| k < key);
		if cs[i].0.len() <= A {
			if i > 0 && cs[i-1].0.len() > A {
				let [l, r] = &mut cs[i-1..=i] else { panic!() };
				r.0.insert(0, l.0.pop().unwrap());
				if let (Some(l), Some(r)) = (l.1.as_mut(), r.1.as_mut()) {
					r.insert(0, l.pop().unwrap());
				}
				mem::swap(&mut self.0[i-1], &mut r.0[0]);
			} else {
				i -= (i > 0 && cs[i-1].0.len() == A) as usize;
				Self::merge(&mut self.0, cs, i);
				if self.0.is_empty() {
					*self = cs.remove(0);
					return self.remove(key);
				}
			}
		}
		if self.0.get(i).map(|(k,_)| k == key).unwrap_or(false) {
			let mut c = &mut cs[i];
			while let Some(gc) = &mut c.1 { c = gc.last_mut().unwrap() }
			mem::swap(&mut self.0[i], c.0.last_mut().unwrap());
		}
		cs[i].remove(key)
	}
	pub fn range(&self, range: impl RangeBounds<K>) -> impl Iterator<Item = &(K, V)> {
		let mut stack = [(self, 0, None); 16];
		let mut len = 1;
		iter::from_coroutine(#[coroutine] move || {
			while len > 0 {
				let (t, mut i, aft) = stack[{ len -= 1; len }];
				while i < t.0.len() && !range.contains(&t.0[i].0) { i += 1 };
				if let Some(cs) = &t.1 {
					if i < t.0.len() && range.contains(&t.0[i].0) {
						stack[len..len+2].copy_from_slice(&[(t, i+1, aft), (&cs[i], 0, Some(&t.0[i]))]);
						len += 2;
					} else {
						stack[{ len += 1; len - 1 }] = (&cs[i], 0, aft);
					}
				} else {
					while i < t.0.len() && range.contains(&t.0[i].0) { yield &t.0[i]; i += 1; }
					if let Some(e) = aft { yield e };
				}
			}
		})
	}
}
