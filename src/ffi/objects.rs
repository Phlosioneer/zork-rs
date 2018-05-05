
use libc::{c_char, c_int};
use std::ops::Range;
use core;
use ffi;

// All the metadata about all objects. Stored as parallel arrays.
#[repr(C)]
pub struct Objects {
    limit: c_int,
    pub description_1: [c_int; 220],
    pub description_2: [c_int; 220],
    pub desco: [c_int; 220],
    pub action: [c_int; 220],
    pub flags_1: [c_int; 220],
    pub flags_2: [c_int; 220],
    pub fval: [c_int; 220],
    pub tval: [c_int; 220],

    // The object's size.
    pub size: [c_int; 220],

    // The amount the object can hold, if it can hold anything.
    pub capacity: [c_int; 220],

    // The room the object is in.
    pub room: [c_int; 220],

    // The adventurer that is holding this object, or 0 otherwise.
    pub adventurer: [c_int; 220],

    // The container this object is in, or 0 otherwise.
    pub container: [c_int; 220],
    pub read: [c_int; 220]
}

// All the metadata about one object.
#[derive(Debug, Clone)]
pub struct ObjectEntry {
    index: usize,

    pub description_1: c_int,
    pub description_2: c_int,
    pub desco: c_int,
    pub action: c_int,
    pub flags_1: c_int,
    pub flags_2: c_int,
    pub fval: c_int,
    pub tval: c_int,

    // The object's size.
    pub size: c_int,

    // The amount the object can hold, if it can hold anything.
    pub capacity: c_int,

    // The room the object is in, or 0 if it isn't in a room.
    pub room: c_int,

    // The adventurer that is holding this object, or 0 otherwise.
    pub adventurer: c_int,

    // The container this object is in, or 0 otherwise.
    pub container: c_int,
    pub read: c_int
}

// This struct has info about which objects are "global" (objects 193+)
#[repr(C)]
#[derive(Clone, Debug)]
pub struct GlobalObjects {
    pub mbase: c_int,
    start_index: c_int
}

pub struct Iter<'a> {
    parent: &'a Objects,
    inner: Range<usize>
}

impl GlobalObjects {
    pub fn get_start_id(&self) -> usize {
        (self.start_index as usize) + 1
    }
}


impl Objects {
    /// Gets an object by its id. Object ids start at 1.
    pub fn get(&self, id: usize) -> ObjectEntry {
        if id > self.len() {
            error!("Object id {} is greater than object count ({}).", id, self.len());
            core::exit_program();
        }
        if id == 0 {
            error!("Object id cannot be 0.");
            core::exit_program();
        }

        let index = id - 1;

        ObjectEntry {
            index,

            description_1: self.description_1[index],
            description_2: self.description_2[index],
            desco: self.desco[index],
            action: self.action[index],
            flags_1: self.flags_1[index],
            flags_2: self.flags_2[index],
            fval: self.fval[index],
            tval: self.tval[index],
            size: self.size[index],
            capacity: self.capacity[index],
            room: self.room[index],
            adventurer: self.adventurer[index],
            container: self.container[index],
            read: self.read[index]
        }
    }

    pub fn set(&mut self, values: &ObjectEntry) {
        let index = values.index;

        self.description_1[index] = values.description_1;
        self.description_2[index] = values.description_2;
        self.desco[index] = values.desco;
        self.action[index] = values.action;
        self.flags_1[index] = values.flags_1;
        self.flags_2[index] = values.flags_2;
        self.fval[index] = values.fval;
        self.tval[index] = values.tval;
        self.size[index] = values.size;
        self.capacity[index] = values.capacity;
        self.room[index] = values.room;
        self.adventurer[index] = values.adventurer;
        self.container[index] = values.container;
        self.read[index] = values.read;
    }

    pub fn len(&self) -> usize {
        self.limit as usize
    }

    // Returns an iterator over all objects.
    pub fn iter(&self) -> Iter {
        Iter {
            parent: &self,
            inner: 0 .. self.len()
        }
    }

    // Returns an iterator over global objects.
    pub fn global_ids(&self) -> Iter {
        let start = unsafe { ffi::global_items.get_start_id() };
        Iter {
            parent: &self,
            inner: (start - 1) .. self.len()
        }
    }
}

impl ObjectEntry {
    // Get the ID for this object.
    pub fn get_id(&self) -> usize {
        self.index + 1
    }

    // This bit is set if the object is visible.
    pub fn is_visible(&self) -> bool {
        ffi::c_int_to_bool(self.flags_1 & (1 << 15))
    }

    // This bit is set if the object is transparent.
    pub fn is_transparent(&self) -> bool {
        ffi::c_int_to_bool(self.flags_1 & (1 << 11))
    }
    
    // This bit is set if the object can be reached (by the player for grabbing).
    pub fn get_find_bit(&self) -> bool {
        ffi::c_int_to_bool(self.flags_2 & (1 << 15))
    }

    // This bit is set if the object is a container and it's open.
    pub fn is_open(&self) -> bool {
        ffi::c_int_to_bool(self.flags_2 & (1 << 3))
    }

    // This bit is set if the object is searchable.
    pub fn is_searchable(&self) -> bool {
        ffi::c_int_to_bool(self.flags_2 & (1 << 0))
    }

    // True if the given noun/adjective pair describes this object.
    pub fn matches(&self, noun: usize, adjective: usize) -> bool {
        trace!("calling thisit_({}, {}, {}, {})", noun, adjective, self.get_id(), 0);
        let ret = unsafe { ffi::thisit_(noun as c_int, adjective as c_int,
                                   self.get_id() as c_int, 0).into() };
        trace!("thisit_ returned {}", ret);
        ret
    }

    // True if this item is in the given room.
    pub fn is_in_room(&self, room: usize) -> bool {
        trace!("calling qhere_({}, {})", self.get_id(), room);
        let ret = unsafe { ffi::qhere_(self.get_id() as c_int, room as c_int).into() };
        trace!("qhere_ returned {}", ret);
        ret
    }

}

impl<'a> Iterator for Iter<'a> {
    type Item = ObjectEntry;

    fn next(&mut self) -> Option<ObjectEntry> {
        self.inner.next().map(|index| self.parent.get(index + 1))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = <Range<usize> as ExactSizeIterator>::len(&self.inner);
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}
