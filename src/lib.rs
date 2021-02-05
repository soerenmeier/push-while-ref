//! A crate to enable holding a reference while still being able to push.  
//! This is possible if you have another lifetime just for storing data
//! (here called `Owner`).
//!
//! The data that is inserted needs to not move in memory, because if the container (Vec, HashMap...)
//! needs to reallocate that would invalidate the reference.
//! this is garantie is give by the trait `StaticType`.
//!
//! # Example pushing
//! ```
//! use push_and_read::{VecOwner, VecChild};
//! let mut vec = VecOwner::new();
//! let mut vec = vec.child();
//! let v1 = vec.push(Box::new(10));
//! let v2 = vec.push(Box::new(20));
//! assert_eq!(*v1, 10);
//! assert_eq!(*v2, 20);
//! ```
//!
//! # Example inserting
//! ```
//! # use push_and_read::{HashMapOwner, HashMapChild};
//! let mut map = HashMapOwner::new();
//! let mut map = map.child();
//! let v1 = map.insert("10", Box::new(10));
//! let v2 = map.insert("20", Box::new(20));
//! assert_eq!(*v1, 10);
//! assert_eq!(*v2, 20);
//! ```


use std::hash::Hash;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VecOwner<T>(Vec<T>);

impl<T> VecOwner<T>
where T: StaticType {

	/// Create a new empty vector.
	pub fn new() -> Self {
		Self(vec![])
	}

	/// you need to make sure that the pointer is not used after
	/// VecOwner goes out of scope and that the ptr is only used to cast to a reference
	#[inline]
	pub(crate) fn push(&mut self, v: T) -> *const T::Ref {
		let ptr = v.ref_ptr();
		self.0.push(v);

		debug_assert_eq!(
			self.0.last().unwrap().ref_ptr(),
			ptr,
			"Trait promises we're not uphold"
		);

		ptr
	}

	pub fn child(&mut self) -> VecChild<'_, T> {
		VecChild(self)
	}

}

#[derive(Debug)]
pub struct VecChild<'a, T>(&'a mut VecOwner<T>);

impl<'a, T> VecChild<'a, T>
where T: StaticType {

	/// pushes the value to the owner
	/// and returns a reference to the inserted value
	/// without using the current lifetime
	///
	/// ```
	/// # use push_and_read::{VecOwner, VecChild};
	/// let mut vec = VecOwner::new();
	/// let mut vec = vec.child();
	/// let v1 = vec.push(Box::new(10));
	/// let v2 = vec.push(Box::new(20));
	/// assert_eq!(*v1, 10);
	/// assert_eq!(*v2, 20);
	/// ```
	pub fn push(&mut self, v: T) -> &'a T::Ref {
		let ptr = self.0.push(v);
		// safe because ptr does not live longer than VecOwner
		unsafe { &*ptr }
	}
}


#[derive(Debug)]
pub struct HashMapOwner<K, T>(HashMap<K, T>);

impl<K, T> HashMapOwner<K, T>
where
	K: Hash + Eq + Clone,
	T: StaticType {

	pub fn new() -> Self {
		Self(HashMap::new())
	}

	/// you need to make sure that the pointer is not used after
	/// VecOwner goes out of scope and that the ptr is only used to cast to a reference
	#[inline]
	pub(crate) fn try_insert(&mut self, key: K, v: T) -> Option<*const T::Ref> {
		// check if key already contained in HashMap
		// this needs to be done because else we would invalid the promise we give
		if self.0.contains_key(&key) {
			return None
		}

		let ptr = v.ref_ptr();

		// check that traits are upholding their promise
		if cfg!(debug_assertions) {
			self.0.insert(key.clone(), v);
			let v = self.0.get(&key).unwrap();
			assert_eq!(ptr, v.ref_ptr(), "Trait promises we're not uphold");

		// we expect the trait to upholde their promise
		} else {
			self.0.insert(key, v);
		}

		Some(ptr)
	}

	pub fn child(&mut self) -> HashMapChild<'_, K, T> {
		HashMapChild(self)
	}

}


#[derive(Debug)]
pub struct HashMapChild<'a, K, T>(&'a mut HashMapOwner<K, T>);

impl<'a, K, T> HashMapChild<'a, K, T>
where
	K: Hash + Eq + Clone,
	T: StaticType {

	/// Tries to insert key and value
	/// if key already exists returns None
	/// 
	/// Else returns a reference to the inserted value
	/// without using the current lifetime
	///
	/// ```
	/// # use push_and_read::{HashMapOwner, HashMapChild};
	/// let mut map = HashMapOwner::new();
	/// let mut map = map.child();
	/// let v1 = map.try_insert("10", Box::new(10)).unwrap();
	/// let v2 = map.try_insert("20", Box::new(20)).unwrap();
	/// assert_eq!(*v1, 10);
	/// assert_eq!(*v2, 20);
	/// ```
	pub fn try_insert(&mut self, key: K, v: T) -> Option<&'a T::Ref> {
		self.0.try_insert(key, v)
			.map(|ptr| {
				// safe because ref does not live longer than HashMapOwner
				unsafe { &*ptr }
			})
	}

	/// Insert key and value to the owner
	/// 
	/// Else returns a reference to the inserted value
	/// without using the current lifetime
	///
	/// # Panics
	/// Panics if key already exists
	///
	/// ```
	/// # use push_and_read::{HashMapOwner, HashMapChild};
	/// let mut map = HashMapOwner::new();
	/// let mut map = map.child();
	/// let v1 = map.insert("10", Box::new(10));
	/// let v2 = map.insert("20", Box::new(20));
	/// assert_eq!(*v1, 10);
	/// assert_eq!(*v2, 20);
	/// ```
	pub fn insert(&mut self, key: K, v: T) -> &'a T::Ref {
		let ptr = self.0.try_insert(key, v).expect("Key already exists");
		// safe because ref does not live longer than HashMapOwner
		unsafe { &*ptr }
	}

}


/// A value that is fixed in memory.
///
/// If you implement this
/// you need to guarantee that Self and Ref does not move in memory
///
/// Most probably it should be allocated on the heap
pub unsafe trait StaticType {
	type Ref: ?Sized;

	/// the caller of ref_ptr promises to
	/// not use the ptr after Self goes out of scope
	/// and to use the ptr only to cast to a references
	fn ref_ptr(&self) -> *const Self::Ref;
}

// unsafe impl<T> StaticType for Vec<T> {
// 	type Ref = [T];

// 	fn ref_ptr(&self) -> *const Self::Ref {
// 		&*self
// 	}

// 	fn ref(&self) -> &Self::Ref {
// 		&self
// 	}
// }

unsafe impl<T: ?Sized> StaticType for Box<T> {
	type Ref = T;

	#[inline]
	fn ref_ptr(&self) -> *const Self::Ref {
		&**self
	}
}


#[cfg(test)]
mod tests {

	use super::*;

	fn is_send<T: Send>() {}
	fn is_sync<T: Sync>() {}

	#[test]
	fn insert_to_vec() {
		let mut v = VecOwner::new();
		let mut v = v.child();
		let s1 = v.push(Box::new(String::from("hey")));
		let s2 = v.push(Box::new(String::from("hey 2")));
		assert_eq!("hey", s1);
		assert_eq!("hey 2", s2);
	}

	#[test]
	#[should_panic]
	fn insert_twice() {

		let mut files = HashMapOwner::new();
		let mut files = files.child();

		files.insert("abc", vec![1].into_boxed_slice());
		files.insert("abc", vec![1].into_boxed_slice());

	}

	#[test]
	fn test_auto_traits() {
		type Basic = Box<usize>;
		is_send::<VecOwner<Basic>>();
		is_sync::<VecOwner<Basic>>();
		is_send::<VecChild<Basic>>();
		is_sync::<VecChild<Basic>>();
		is_send::<HashMapOwner<Basic, Basic>>();
		is_sync::<HashMapOwner<Basic, Basic>>();
		is_send::<HashMapChild<Basic, Basic>>();
		is_sync::<HashMapChild<Basic, Basic>>();
	}

}