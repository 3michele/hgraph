// use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, fmt::{Debug, Display}, hash::{DefaultHasher, Hash, Hasher, RandomState}, ops::{Add, Deref}, rc::Rc};

// /// Best hash for rust
// use ahash::{AHashMap, AHashSet};

// struct EdgeID {
//     id: i64,
//     weight: f64,
//     nodes: Rc<Vec<i32>>,
// }

// impl EdgeID {
//     pub fn new(id: i64, weight: f64, nodes: Rc<Vec<i32>>) -> Self {
//         Self {
//             id,
//             weight,
//             nodes
//         }
//     }
// }

// impl Clone for EdgeID {
//     fn clone(&self) -> Self {
//         Self {
//             id: self.id,
//             weight: self.weight,
//             nodes: Rc::clone(&self.nodes)
//         }
//     }
// }

// impl Debug for EdgeID {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{{id: {}, weight: {}, nodes: {:?}}}", self.id, self.weight, self.nodes.deref())
//     }
// }

// impl Display for EdgeID {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}, {}, {:?}", self.id, self.weight, self.nodes.deref())
//     }
// }

// // Efficient O(1) hashing, since it is
// impl Hash for EdgeID {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.id.hash(state);
//     }
// }

// impl PartialEq for EdgeID {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// impl Eq for EdgeID { }

// #[derive(Debug)]
// pub struct Key {
//     key: Rc<RefCell<i64>>
// }

// impl Key {
//     pub fn new(key: Rc<RefCell<i64>>) -> Self {
//         Self {
//             key
//         }
//     }
// }

// impl PartialEq for Key {
//     fn eq(&self, other: &Self) -> bool {
//         *self.key.borrow() == *other.key.borrow()
//     }
// }
// impl Hash for Key {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         (*self.key.borrow()).hash(state);
//     }
// }
// impl Eq for Key { }

// fn calculate_ahashmap_compatible_hash<T: Hash>(value: &T) -> u64 {
//     // Create a BuildHasher with the same configuration used by AHashMap
//     let build_hasher = RandomState::new(); // This is the default hasher for AHashMap
//     let mut hasher = build_hasher.build_hasher();

//     // Hash the value using this hasher
//     value.hash(&mut hasher);
//     hasher.finish()
// }

// fn main() {
//     // For memory optimizing; Rc<T> is a smart pointer, and, apart from a raw pointer to the value stored, it has two more fields:
//     //  - strong_count: usize - counts strong reference to the data stored;
//     //  - weak_count: usize - counts weak reference to the data stored.
//     // So, we have a memory overhead of 8 + 8 = 16 bytes.
//     //
//     // For time efficiency, the overhead is basically the same of a C pointer. When the last reference to the stored data will be dropped,
//     // then rust automatically deallocates the stored data T: this corresponds to the free()/delete operation in C/C++; So, the overhead
//     // is basically the same, except for  the incrementing/decrementing operation of the fields strong_count / weak_count, which make the
//     // CPU do one, or two, more instructions.
//     let mut map: AHashMap<Rc<Vec<i32>>, EdgeID> = AHashMap::new();
//     let mut set: AHashSet<EdgeID> = AHashSet::new();

//     let edge1 = vec![1, 2, 3];
//     let edge2 = vec![2, 3, 4];
//     let edge3 = vec![4, 5, 6];
//     let edge4 = vec![1,2,3];

//     let p1_0 = Rc::new(edge1);
//     let id1 = EdgeID::new(1, 27.7, Rc::clone(&p1_0));

//     let p2 = Rc::new(edge2);
//     let id2 = EdgeID::new(2, 10.1, Rc::clone(&p2));

//     let p3 = Rc::new(edge3);
//     let id3 = EdgeID::new(3, 12.9, Rc::clone(&p3));

//     let p4 = Rc::new(edge4);

//     // Clone is not expensive, O(1)
//     map.insert(p1_0, id1.clone());
//     map.insert(p2, id2.clone());
//     map.insert(p3, id3.clone());
//     set.insert(id1);
//     set.insert(id2);
//     set.insert(id3);

//     if map.contains_key(&p4) {
//         println!("YES");
//     } else {
//         set.insert(EdgeID::new(4, 100_f64, p4));
//     }

//     println!("map");
//     for (k,v) in map.iter() {
//         println!("k = {:?}, v = {:?}", k, v);
//     }

//     println!("set");
//     for edge_id in set.iter() {
//         println!("{:?}", edge_id);
//     }
//     // It works!!

//     let mut now = Rc::new(100);
//     now.borrow_mut().add(100);

//     println!("{:?}", now);

//     let mut map = HashMap::new();
//     map.insert("Michele", 27);
//     map.insert("Samuel", 200);
//     map.insert("Filippo", 78);

//     let mut hasher = DefaultHasher::new();
//     let hash = "Michele".hash(&mut hasher);
//     let now = hasher.finish();
//     println!("{}", now);

//     println!("========================================================");

//     use ahash::{AHashMap, RandomState};
//     use std::hash::{BuildHasher, Hash, Hasher};

//     let value = "Hello, world!";

//     // Calculate hash compatible with AHashMap
//     let hash_value = calculate_ahashmap_compatible_hash(&value);
//     println!("Hash value of {:?} (compatible with AHashMap) is: {}", value, hash_value);

//     // Insert into AHashMap to verify compatibility
//     let mut map: AHashMap<&str, i32> = AHashMap::new();
//     map.insert("Hello, world!", 42);

//     // Check if we can retrieve it by calculating the same hash
//     let contains_key = map.contains_key("Hello, world!");
//     println!("AHashMap contains key 'Hello, world!': {}", contains_key);
// }

pub fn main() {}
