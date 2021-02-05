A crate to enable holding a reference while still being able to push.  
This is possible if you have another lifetime just for storing data
(here called `Owner`).

The data that is inserted needs to not move in memory, because if the container (Vec, HashMap...)
needs to reallocate that would invalidate the reference.
this is garantie is give by the trait `StaticType`.

# Example pushing
```rust
use push_while_ref::{VecOwner, VecChild};

let mut vec = VecOwner::new();
let mut vec = vec.child();
let v1 = vec.push(Box::new(10));
let v2 = vec.push(Box::new(20));
assert_eq!(*v1, 10);
assert_eq!(*v2, 20);
```

# Example inserting
```rust
use push_while_ref::{HashMapOwner, HashMapChild};

let mut map = HashMapOwner::new();
let mut map = map.child();
let v1 = map.insert("10", Box::new(10));
let v2 = map.insert("20", Box::new(20));
assert_eq!(*v1, 10);
assert_eq!(*v2, 20);
```