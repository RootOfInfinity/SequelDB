// FINE, I'll make a SkipList, it seems to be way more efficent and easier than a B-Tree.
// I do wanna make a B-Tree later though! You can't stop me!!!
// All that is required now is an iterator through the key value pairs of the skiplist and
// tests.

use std::sync::{Arc, RwLock};

use rand::random_range;

const NULL_KEY: Key = 0;
type Key = u64;
const NULL_DATA_VALUE: DataValue = Vec::new();
type DataValue = Vec<u8>;

const MAX_LEVEL: u8 = 14;

/// This is the node datatype used internally for creating skiplists.
/// The level of the node is determined by the length of the vec.
/// It is always the last item in the skiplist when pointers[0] = None.
pub struct SkipListNode {
    key: Key,
    value: Epitaph,
    pointers: Vec<Option<Arc<RwLock<SkipListNode>>>>,
    deleted: bool,
}

/// Epitaph represents whether a value exists (Alive) or doesn't (Tombstone).
/// The name of the enum comes from my computer science teacher, and exists as a joke
/// in the internal memtable file.
enum Epitaph {
    Alive(DataValue),
    Tombstone,
}

/// KeyLocation represents whether the location of the key was found, in which case
/// it gives the exact location and metadata about predecessors, or it only gives
/// information about what the predecessors would be if it existed.
/// Perhaps the name 'Before' doesn't fit the enum anymore.
enum KeyLocation {
    Exact(
        Arc<RwLock<SkipListNode>>,
        [Arc<RwLock<SkipListNode>>; MAX_LEVEL as usize],
    ),
    Before([Arc<RwLock<SkipListNode>>; MAX_LEVEL as usize]),
}

// Public API for using the Skip List.

/// Honest Function
/// Creates a new concurrent latch-based probabalistic skiplist. Make sure to use Arc::clone for it, and
/// if the list is no longer referenced, it will be deleted from RAM.
/// You should not use key 0 in the list, as the skiplist uses key 0 for the first node.
pub fn create_new_list() -> Arc<RwLock<SkipListNode>> {
    let mut skip_list_start = SkipListNode {
        key: NULL_KEY,
        value: Epitaph::Alive(NULL_DATA_VALUE),
        pointers: Vec::with_capacity(MAX_LEVEL as usize),
        deleted: false,
    };
    for i in 0..(MAX_LEVEL as usize) {
        skip_list_start.pointers[i] = None;
    }
    Arc::new(RwLock::new(skip_list_start))
}

/// Honest Function
/// This function will return the value associated with the key in the skiplist where
/// 'start_node' is the first node in the skiplist. It will not access any write locks, but only
/// gather read locks.
/// It will return None if the key was not found in the skiplist.
pub fn get_value(key: Key, start_node: Arc<RwLock<SkipListNode>>) -> Option<DataValue> {
    let key_location =
        search_for_item_or_predecessors(key, start_node, std::array::from_fn(|_| None));
    match key_location {
        KeyLocation::Exact(node, _) => {
            let read_latch = node.read().unwrap();
            // if some thread deletes the item, we shouldn't return it,
            // and if it is now created, the transaction manager should deal with it.
            match read_latch.value {
                Epitaph::Alive(ref value) => return Some(value.clone()),
                Epitaph::Tombstone => return None,
            }
        }
        KeyLocation::Before(_) => return None,
    }
}

/// Dishonest Function
/// This function will update a key value pair if the key exists, otherwise it will create
/// an entirely new node with a random level and insert it into the right position on the
/// sorted skiplist.
pub fn insert_value(key: Key, value: DataValue, start_node: Arc<RwLock<SkipListNode>>) {
    insert_item(key, Epitaph::Alive(value), start_node);
}

/// Dishonest Function
/// This function will delete a value from the skiplist, either by putting a tombstone value
/// in there (updating if it is in the skiplist, or inserting a new node with a tombstone value.),
/// or by deleting the node from the skiplist. You should use a tombstone if the value exists in a
/// disk-resident SSTable, otherwise it is safe to just delete it from the skiplist without a tombstone.
/// It will return None if it could not find the node to delete when 'with_tombstone' == false.
pub fn delete_value(
    key: Key,
    start_node: Arc<RwLock<SkipListNode>>,
    with_tombstone: bool,
) -> Option<()> {
    if with_tombstone {
        insert_item(key, Epitaph::Tombstone, start_node);
        return Some(());
    } else {
        let key_location = search_for_item_or_predecessors(
            key,
            Arc::clone(&start_node),
            std::array::from_fn(|_| None),
        );
        match key_location {
            KeyLocation::Exact(node, predecessors) => {
                let level = node.read().unwrap().pointers.len() - 1;
                delete_node(node, key, level, predecessors, start_node);
                return Some(());
            }
            _ => return None,
        }
    }
}

// Internal API

/// Honest Function
/// This is a recursive function that will traverse the skiplist, using latch crabbing
/// to make sure it doesn't use too many read latches. It will output a KeyLocation,
/// which either gives the exact node where the key is, as well as an array of predecessors,
/// or it will give just an array of predecessors for where the key would have been had it
/// existed.
fn search_for_item_or_predecessors(
    key: Key,
    start_node: Arc<RwLock<SkipListNode>>,
    mut possible_predecessors_by_level: [Option<Arc<RwLock<SkipListNode>>>; MAX_LEVEL as usize],
) -> KeyLocation {
    // TODO: Special case if it's the last node
    // Unwrap if the other threads panic.
    let read_start = start_node.read().unwrap();
    // if read_start.key == key {
    //     return KeyLocation::Exact(Arc::clone(&start_node));
    // }
    // Add self to metadata
    let level = read_start.pointers.len() - 1;
    possible_predecessors_by_level[level] = Some(Arc::clone(&start_node));

    let mut next_node: Option<Arc<RwLock<SkipListNode>>> = None;

    for node_option in read_start.pointers.iter().rev() {
        if let Some(node) = node_option {
            let new_node_read = node.read().unwrap();
            if new_node_read.key == key && !new_node_read.deleted {
                let new_predecessors = refine_predecessors(possible_predecessors_by_level);
                return KeyLocation::Exact(Arc::clone(&node), new_predecessors);
            } else if new_node_read.key < key {
                next_node = Some(Arc::clone(&node));
                break;
            }
        }
    }
    drop(read_start);
    match next_node {
        Some(next_node) => {
            search_for_item_or_predecessors(key, next_node, possible_predecessors_by_level)
        }
        None => {
            for i in (1..possible_predecessors_by_level.len()).into_iter().rev() {
                if let None = possible_predecessors_by_level[i - 1] {
                    possible_predecessors_by_level[i - 1] =
                        possible_predecessors_by_level[i].clone();
                }
            }
            let new_predecessors = refine_predecessors(possible_predecessors_by_level);
            KeyLocation::Before(new_predecessors)
        }
    }
}

/// Dishonest Function
/// This function will do one of two things based on if the node associated with the key exists:
///
/// * If yes, it will update the key value pair with the new value, using a write lock.
/// * If no, it will create a new node with a random level for the key value pair, updating the pointers of its predecessors.
///
/// You can use this function to insert a tombstone value for a key.
/// Research and testing will show if this has the possiblility for deadlocks.
fn insert_item(key: Key, value: Epitaph, start_node: Arc<RwLock<SkipListNode>>) {
    let key_location = search_for_item_or_predecessors(
        key,
        Arc::clone(&start_node),
        std::array::from_fn(|_| None),
    );
    match key_location {
        KeyLocation::Exact(node, _) => {
            let mut write_lock = node.write().unwrap();
            write_lock.value = value;
        }
        KeyLocation::Before(mut possibles) => {
            let level = skip_level() as usize;
            let mut latch_vec: Vec<std::sync::RwLockWriteGuard<'_, SkipListNode>> = (0..=level)
                .map(|index| possibles[index].write().unwrap())
                .collect();
            // make sure predecessors aren't deleted
            while {
                let mut at_least_one_deleted = false;
                for w_latch in &latch_vec[..] {
                    if w_latch.deleted {
                        at_least_one_deleted = true;
                    }
                }
                at_least_one_deleted
            } {
                drop(latch_vec);
                let key_location = search_for_item_or_predecessors(
                    key,
                    Arc::clone(&start_node),
                    std::array::from_fn(|_| None),
                );
                possibles = match key_location {
                    KeyLocation::Exact(_, x) => x,
                    KeyLocation::Before(x) => x,
                };
                latch_vec = (0..=level)
                    .map(|index| possibles[index].write().unwrap())
                    .collect();
            }

            let new_pointers = (0..=latch_vec.len())
                .map(|index| latch_vec[index].pointers[index].clone())
                .collect();
            let new_node = Arc::new(RwLock::new(SkipListNode {
                key,
                value,
                pointers: new_pointers,
                deleted: false,
            }));
            for (i, latch) in latch_vec.iter_mut().enumerate() {
                latch.pointers[i] = Some(Arc::clone(&new_node));
            }
            drop(latch_vec);
        }
    }
}

/// Honest Function
/// This function will delete a node from the skiplist, removing the references to it in
/// other nodes and updating it to show that it is deleted and should not be used.
/// This will not create a tombstone value, this is just for actually deleting the node from the skiplist.
fn delete_node(
    node_to_kill: Arc<RwLock<SkipListNode>>,
    key: Key,
    level: usize,
    mut predecessors_by_level: [Arc<RwLock<SkipListNode>>; MAX_LEVEL as usize],
    start_node: Arc<RwLock<SkipListNode>>,
) {
    let mut predesessor_write_latch_vec: Vec<std::sync::RwLockWriteGuard<'_, SkipListNode>> = (0
        ..=level)
        .map(|index| predecessors_by_level[index].write().unwrap())
        .collect();

    // to make sure we don't have predecessors that are deleted
    while {
        let mut at_least_one_deleted = false;
        for w_latch in &predesessor_write_latch_vec[..] {
            if w_latch.deleted {
                at_least_one_deleted = true;
            }
        }
        at_least_one_deleted
    } {
        drop(predesessor_write_latch_vec);
        let key_location = search_for_item_or_predecessors(
            key,
            Arc::clone(&start_node),
            std::array::from_fn(|_| None),
        );
        predecessors_by_level = match key_location {
            KeyLocation::Exact(_, x) => x,
            KeyLocation::Before(x) => x,
        };
        predesessor_write_latch_vec = (0..=level)
            .map(|index| predecessors_by_level[index].write().unwrap())
            .collect();
    }
    let mut write_latch = node_to_kill.write().unwrap();
    write_latch.deleted = true;
    for (i, w_latch) in predesessor_write_latch_vec.iter_mut().enumerate() {
        w_latch.pointers[i] = write_latch.pointers[i].clone();
    }
}

// Helper Functions

/// Honest Helper Function
/// Panics when given an array where Option::None is the last element.
/// Makes sure there are no instances of None in the lists of predecessors.
/// It takes the one of the next highest level and subs in a None for that, unless it is
/// None, in which it does the next, and so on.
/// This is so that we don't have to handle the errors in later manipulation of the predecessors.
fn refine_predecessors(
    possible_predecessors_by_level: [Option<Arc<RwLock<SkipListNode>>>; MAX_LEVEL as usize],
) -> [Arc<RwLock<SkipListNode>>; MAX_LEVEL as usize] {
    let new_predecessors: [Arc<RwLock<SkipListNode>>; MAX_LEVEL as usize] =
        std::array::from_fn(|i| {
            if let Some(ref val_ref) = possible_predecessors_by_level[i] {
                Arc::clone(val_ref)
            } else {
                panic!("Programming error, should never fail here");
            }
        });
    new_predecessors
}

/// Dishonest Helper Function
/// We want the amount of nodes to be about 30_000.
/// The distrobution is made for that reason.
/// On average, it will be kinda like a binary tree in speed,
/// but it is probablilstic, so it's possible it will just be a 100_000
/// length linked_list.
/// We should flush on a lvl 14 or every 10minutes, but that isn't
/// implemented yet.
fn skip_level() -> u8 {
    let random: u32 = random_range(0..30_000);
    match random {
        0 => 14,
        1..3 => 13,
        3..7 => 12,
        7..15 => 11,
        15..31 => 10,
        31..63 => 9,
        63..127 => 8,
        127..255 => 7,
        255..511 => 6,
        511..1023 => 5,
        1023..2047 => 4,
        2047..4095 => 3,
        4095..8193 => 2,
        8193..16387 => 1,
        16387.. => 0,
    }
}
