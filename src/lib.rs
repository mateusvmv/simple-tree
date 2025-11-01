#![feature(new_range_api, array_windows, let_chains)]
use std::{iter, mem::{self}, ops::RangeBounds, range::Bound};
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
	fn uplift(ks: &mut Vec<(K, V)>, cs: &mut Vec<Self>, ci: usize, gc: usize) {
		let l = SimpleTree(
			cs[ci].0.drain(gc+1..).collect(),
			cs[ci].1.as_mut().map(|v| v.drain(gc+1..).collect()));
		cs.insert(ci + 1, l);
		ks.insert(ci, cs[ci].0.remove(gc));
	}
	fn downlift(ks: &mut Vec<(K, V)>, cs: &mut Vec<Self>, ci: usize) {
		let r = cs.remove(ci + 1);
		cs[ci].0.push(ks.remove(ci));
		cs[ci].0.extend(r.0);
		let Some(cic) = &mut cs[ci].1 else { return };
		cic.extend(r.1.expect("Not same layer?"));
	}
	pub fn insert(&mut self, key: K, val: V) {
		if self.0.len() == B {
			let c = mem::take(self);
			Self::uplift(&mut self.0, self.1.insert(vec![c]), 0, A);
		}
		let mut i = self.0.partition_point(|(k, _)| k < &key);
		if let Some(e) = self.0.get_mut(i) && e.0 == key {
			e.1 = val;
			return;
		}
		if let Some(c) = &mut self.1 {
			if c[i].0.len() == B {
				Self::uplift(&mut self.0, c, i, A);
				if self.0[i].0 < key { i += 1 };
			}
			c[i].insert(key, val)
		} else {
			self.0.insert(i, (key, val))
		}
	}
	fn borrow(ks: &mut Vec<(K, V)>, cs: &mut Vec<Self>, ci: &mut usize) {
		if cs[*ci].0.len() > A { return };
		if let Some(c) = cs.get(*ci + 1) {
			if c.0.len() > A {
				Self::uplift(ks, cs, *ci + 1, c.0.len() - A - 1);
			}
		} else if cs[*ci-1].0.len() <= A {
			*ci -= 1;
		} else {
			Self::uplift(ks, cs, *ci - 1, A);
		}
		Self::downlift(ks, cs, *ci);
	}
	pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
		let Some(cs) = self.1.as_mut() else {
			return self.0.binary_search_by(|(k,_)| k.cmp(key)).ok()
				.map(|i| self.0.remove(i));
		};
		let mut i = self.0.partition_point(|(k,_)| k < key);
		Self::borrow(&mut self.0, cs, &mut i);
		if self.0.get(i).map(|(k,_)| k == key).unwrap_or(false) {
			let mut c = &mut cs[i];
			while let Some(gc) = &mut c.1 {
				let mut ci = gc.len() - 1;
				Self::borrow(&mut c.0, gc, &mut ci);
				c = gc.last_mut().unwrap()
			}
			return Some(mem::replace(&mut self.0[i], c.0.pop().unwrap()));
		}
		if self.0.is_empty() {
			*self = cs.remove(0);
			return self.remove(key);
		}
		cs[i].remove(key)
	}
	pub fn get(&self, key: &K) -> Option<&(K, V)> {
		let Some(cs) = self.1.as_ref() else {
			return self.0.binary_search_by(|(k,_)| k.cmp(key)).ok()
				.map(|i| &self.0[i]);
		};
		let i = self.0.partition_point(|(k,_)| k < key);
		if self.0.get(i).map(|(k,_)| k == key).unwrap_or(false) {
			return Some(&self.0[i]);
		}
		cs[i].get(key)
	}
	pub fn range(&self, range: impl RangeBounds<K>) -> impl Iterator<Item = &(K, V)> {
		let mut stack = [(self, 0, None); 8];
		let mut len = 1;
		iter::from_fn(move || {
			let sb = range.start_bound();
			let eb = range.end_bound();
			while len > 0 {
				let (t, mut i, aft) = &stack[len - 1];
				while i < t.0.len() && !(sb, Bound::Unbounded).contains(&t.0[i].0) {
					i += 1;
				}
				if let Some(cs) = &t.1 {
					if i < t.0.len() && (Bound::Unbounded, eb).contains(&t.0[i].0) {
						stack[len] = (&cs[i], 0, Some(&t.0[i]));
						stack[len-1].1 = i + 1;
						len += 1;
					} else {
						stack[len-1].0 = &cs[i];
						stack[len-1].1 = 0;
					}
					continue;
				}
				if i < t.0.len() && (Bound::Unbounded, eb).contains(&t.0[i].0) {
					let r = Some(&t.0[i]);
					stack[len-1].1 = i + 1;
					return r;
				}
				len -= 1;
				if let Some(e) = aft {
					return Some(e);
				}
			}
			None
		})
	}
}
