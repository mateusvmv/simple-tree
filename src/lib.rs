#![feature(coroutines, coroutine_trait, iter_from_coroutine)]
use std::{mem, ops::{Bound, RangeBounds}};
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
		let k = if r.0.len() == A { A + 1 } else { r.0.len() - A };
		l.0.extend(r.0.drain(..k.min(r.0.len())));
		if let (Some(l), Some(r)) = (l.1.as_mut(), r.1.as_mut()) {
			l.extend(r.drain(..k));
		}
		mem::swap(&mut ks[i], l.0.last_mut().unwrap());
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
	pub fn remove(&mut self, key: &K) {
		let Some(cs) = self.1.as_mut() else {
			self.0.retain(|(k,_)| k != key);
			return;
		};
		if self.0.len() == 1 {
			Self::merge(&mut self.0, cs, 0);
			*self = cs.remove(0);
			return self.remove(key);
		}
		let mut i = self.0.binary_search_by(|(k,_)| k.cmp(key))
			.map(|i| {
				let mut c = &mut cs[i];
				while let Some(gc) = &mut c.1 { c = gc.last_mut().unwrap() }
				mem::swap(&mut self.0[i], c.0.last_mut().unwrap());
				i
			})
			.unwrap_or_else(|e| e);
		if cs[i].0.len() > A { return cs[i].remove(key) };
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
		}
		cs[i].remove(key)
	}
	pub fn range(&self, range: impl RangeBounds<K>) -> impl Iterator<Item = &(K, V)> where K: Clone {
		std::iter::from_coroutine(move || {
			let range = Box::new(range);
			let x = (!matches!(range.start_bound(), Bound::Unbounded))
				.then(|| self.0.partition_point(|(k,_)| !(range.start_bound(), Bound::Unbounded).contains(k)))
				.unwrap_or(0);
			let y = (!matches!(range.start_bound(), Bound::Unbounded))
				.then(|| self.0.partition_point(|(k,_)| (Bound::Unbounded, range.end_bound()).contains(k)))
				.unwrap_or(self.0.len());
			for i in x..y {
				let from = (i==x).then(|| range.start_bound().cloned()).unwrap_or(Bound::Unbounded);
				if let Some(c) = &self.1 {
					for k in Box::new(c[i].range((from, Bound::Unbounded))) { yield k };
				}
				yield &self.0[i];
			}
			if let Some(c) = &self.1 {
				for k in Box::new(c[y].range(*range)) { yield k };
			}
		})
	}
}
