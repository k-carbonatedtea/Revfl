use crate::util::{BinaryObject, ReadStream, WriteStream};
use std::collections::HashMap;
use std::io::{Seek, Write};

fn get_bit(data: &[u8], bit_idx: i32) -> u8 {
    if bit_idx < 0 {
        return 0;
    }
    let bit_idx = bit_idx as usize;
    let byte_idx_from_end = bit_idx / 8;
    if byte_idx_from_end >= data.len() {
        0
    } else {
        let byte_idx = data.len() - 1 - byte_idx_from_end;
        (data[byte_idx] >> (bit_idx % 8)) & 1
    }
}

fn bit_mismatch(data1: &[u8], data2: &[u8]) -> i32 {
    let max_len = std::cmp::max(data1.len(), data2.len());
    for i in 0..(max_len * 8) {
        if get_bit(data1, i as i32) != get_bit(data2, i as i32) {
            return i as i32;
        }
    }
    -1
}

fn first_1bit(data: &[u8]) -> i32 {
    for i in 0..(data.len() * 8) {
        if get_bit(data, i as i32) == 1 {
            return i as i32;
        }
    }
    unreachable!()
}

#[derive(Debug, Clone)]
pub struct IndexTableEntry {
    pub name: String,
    pub compact_bit_idx: u32,
    pub idx0: u16,
    pub idx1: u16,
}

#[derive(Clone)]
struct Node {
    child: [usize; 2],
    data: Vec<u8>,
    bit_idx: i32,
    parent: usize,
}

impl Node {
    fn get_name(&self) -> String {
        if self.data.is_empty() {
            String::new()
        } else {
            String::from_utf8_lossy(&self.data).into_owned()
        }
    }

    fn get_compact_bit_idx(&self) -> u32 {
        if self.bit_idx < 0 {
            return 0xFFFFFFFF;
        }
        let byte_idx = self.bit_idx / 8;
        ((byte_idx << 3) | (self.bit_idx - 8 * (byte_idx as i32))) as u32
    }
}

pub struct Tree {
    nodes: Vec<Node>,
    ordered_entries: Vec<(Vec<u8>, usize)>,
    entry_indices: HashMap<Vec<u8>, usize>,
}

impl Tree {
    pub fn new() -> Self {
        let root = Node {
            child: [0, 0],
            data: vec![],
            bit_idx: -1,
            parent: 0,
        };
        let mut ordered_entries = Vec::new();
        ordered_entries.push((vec![], 0));
        let mut entry_indices = HashMap::new();
        entry_indices.insert(vec![], 0);

        Self {
            nodes: vec![root],
            ordered_entries,
            entry_indices,
        }
    }

    fn search(&self, data: &[u8], prev: bool) -> usize {
        if self.nodes[0].child[0] == 0 {
            return 0;
        }
        let mut node_idx = self.nodes[0].child[0];
        let mut prev_node_idx;
        loop {
            prev_node_idx = node_idx;
            let node = &self.nodes[node_idx];
            let next_idx = node.child[get_bit(data, node.bit_idx) as usize];
            if self.nodes[next_idx].bit_idx <= node.bit_idx {
                node_idx = next_idx;
                break;
            }
            node_idx = next_idx;
        }
        if prev {
            prev_node_idx
        } else {
            node_idx
        }
    }

    fn insert_entry(&mut self, data: Vec<u8>, node_idx: usize) {
        let idx = self.ordered_entries.len();
        self.entry_indices.insert(data.clone(), idx);
        self.ordered_entries.push((data, node_idx));
    }

    pub fn insert(&mut self, name: &str) {
        let data = name.as_bytes().to_vec();
        let mut current_idx = self.search(&data, true);
        let bit_idx = bit_mismatch(&self.nodes[current_idx].data, &data);

        while bit_idx < self.nodes[self.nodes[current_idx].parent].bit_idx {
            current_idx = self.nodes[current_idx].parent;
        }

        let current_node = self.nodes[current_idx].clone();

        if bit_idx < current_node.bit_idx {
            let new_idx = self.nodes.len();
            let mut new_node = Node {
                child: [0, 0],
                data: data.clone(),
                bit_idx,
                parent: current_node.parent,
            };
            new_node.child[(get_bit(&data, bit_idx) ^ 1) as usize] = current_idx;
            new_node.child[get_bit(&data, bit_idx) as usize] = new_idx;
            self.nodes.push(new_node);

            let parent_idx = current_node.parent;
            let parent_bit = self.nodes[parent_idx].bit_idx;
            self.nodes[parent_idx].child[get_bit(&data, parent_bit) as usize] = new_idx;
            self.nodes[current_idx].parent = new_idx;

            self.insert_entry(data, new_idx);
        } else if bit_idx > current_node.bit_idx {
            let new_idx = self.nodes.len();
            let mut new_node = Node {
                child: [0, 0],
                data: data.clone(),
                bit_idx,
                parent: current_idx,
            };
            if get_bit(&current_node.data, bit_idx) == (get_bit(&data, bit_idx) ^ 1) {
                new_node.child[(get_bit(&data, bit_idx) ^ 1) as usize] = current_idx;
            } else {
                new_node.child[(get_bit(&data, bit_idx) ^ 1) as usize] = 0;
            }
            new_node.child[get_bit(&data, bit_idx) as usize] = new_idx;
            self.nodes.push(new_node);

            let curr_bit = current_node.bit_idx;
            self.nodes[current_idx].child[get_bit(&data, curr_bit) as usize] = new_idx;
            
            self.insert_entry(data, new_idx);
        } else {
            let mut new_bit_idx = first_1bit(&data);
            let child_idx = current_node.child[get_bit(&data, bit_idx) as usize];
            if child_idx != 0 {
                new_bit_idx = bit_mismatch(&self.nodes[child_idx].data, &data);
            }
            let new_idx = self.nodes.len();
            let mut new_node = Node {
                child: [0, 0],
                data: data.clone(),
                bit_idx: new_bit_idx,
                parent: current_idx,
            };
            new_node.child[(get_bit(&data, new_bit_idx) ^ 1) as usize] = child_idx;
            new_node.child[get_bit(&data, new_bit_idx) as usize] = new_idx;
            self.nodes.push(new_node);

            self.nodes[current_idx].child[get_bit(&data, bit_idx) as usize] = new_idx;
            
            self.insert_entry(data, new_idx);
        }
    }

    pub fn get_index_table(&self) -> Vec<IndexTableEntry> {
        let mut table = Vec::new();
        for &(_, node_idx) in &self.ordered_entries {
            let node = &self.nodes[node_idx];
            let data0 = &self.nodes[node.child[0]].data;
            let data1 = &self.nodes[node.child[1]].data;
            let idx0 = *self.entry_indices.get(data0).unwrap() as u16;
            let idx1 = *self.entry_indices.get(data1).unwrap() as u16;
            
            table.push(IndexTableEntry {
                name: node.get_name(),
                compact_bit_idx: node.get_compact_bit_idx(),
                idx0,
                idx1,
            });
        }
        table
    }
}

pub struct DicWriter {
    pub tree: Tree,
    offsets_to_this: Vec<usize>,
}

impl DicWriter {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            offsets_to_this: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: &str) {
        self.tree.insert(key);
    }

    pub fn write_placeholder_offset<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        let pos = stream.tell();
        self.offsets_to_this.push(pos);
        stream.register_pointer(pos);
        stream.write_u64(0xFFFFFFFFFFFFFFFF);
    }

    pub fn write<W: Write + Seek>(&self, stream: &mut WriteStream<W>) {
        let start_pos = stream.tell();
        stream.write_bytes(b"DIC ");
        let index_table = self.tree.get_index_table();
        stream.write_u32((index_table.len() - 1) as u32);
        for entry in index_table {
            stream.write_u32(entry.compact_bit_idx);
            stream.write_u16(entry.idx0);
            stream.write_u16(entry.idx1);
            stream.write_string_ref(&entry.name, false);
        }

        let end_pos = stream.tell();
        for &offset in &self.offsets_to_this {
            stream.seek(offset);
            stream.write_u64(start_pos as u64);
        }
        stream.seek(end_pos);
    }
}

pub struct DicReader {
    pub items: Vec<String>,
}

impl DicReader {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl BinaryObject for DicReader {
    fn read(&mut self, stream: &mut ReadStream) {
        let _magic = stream.read_u32(); // DIC
        let num_entries = stream.read_u32();
        stream.skip(4 + 2 + 2 + 8); // Root entry
        for _ in 0..num_entries {
            stream.skip(4 + 2 + 2);
            let s = stream.read_string_ref();
            assert!(!s.is_empty(), "Invalid entry name");
            self.items.push(s);
        }
    }

    fn write<W: Write + Seek>(&mut self, _stream: &mut WriteStream<W>) {
        unimplemented!()
    }
}
