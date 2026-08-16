use std::collections::{BTreeMap, BTreeSet};
use std::io::{Seek, SeekFrom, Write};

pub struct IdGenerator {
    id: u32,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn gen_id(&mut self) -> u32 {
        let r = self.id;
        self.id += 1;
        r
    }
}

pub fn align_up(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}

pub fn pascal_string(data: &str) -> Vec<u8> {
    let raw_data = data.as_bytes();
    let mut vec = Vec::with_capacity(2 + raw_data.len() + 1);
    vec.extend_from_slice(&(raw_data.len() as u16).to_le_bytes());
    vec.extend_from_slice(raw_data);
    vec.push(0);
    vec
}

pub fn read_string(data: &[u8], offset: usize) -> String {
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    String::from_utf8_lossy(&data[offset..end]).into_owned()
}

pub fn read_pascal_string(data: &[u8], offset: usize) -> String {
    let len_bytes: [u8; 2] = [data[offset], data[offset + 1]];
    let length = u16::from_le_bytes(len_bytes) as usize;
    let start = offset + 2;
    let end = start + length;
    String::from_utf8_lossy(&data[start..end]).into_owned()
}

pub trait BinaryObject {
    fn read(&mut self, stream: &mut ReadStream);
    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>);
    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        let _ = stream;
    }
}

pub struct ReadStream<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> ReadStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn tell(&self) -> usize {
        self.pos
    }

    pub fn seek(&mut self, offset: usize) {
        self.pos = offset;
    }

    pub fn align(&mut self, align: usize) {
        self.pos = align_up(self.pos, align);
    }

    pub fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    pub fn read(&mut self, n: usize) -> &'a [u8] {
        let res = &self.data[self.pos..self.pos + n];
        self.pos += n;
        res
    }

    pub fn read_u8(&mut self) -> u8 {
        let val = self.data[self.pos];
        self.pos += 1;
        val
    }

    pub fn read_u16(&mut self) -> u16 {
        let bytes: [u8; 2] = self.read(2).try_into().unwrap();
        u16::from_le_bytes(bytes)
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes: [u8; 4] = self.read(4).try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    pub fn read_s32(&mut self) -> i32 {
        let bytes: [u8; 4] = self.read(4).try_into().unwrap();
        i32::from_le_bytes(bytes)
    }

    pub fn read_u64(&mut self) -> u64 {
        let bytes: [u8; 8] = self.read(8).try_into().unwrap();
        u64::from_le_bytes(bytes)
    }

    pub fn read_f32(&mut self) -> f32 {
        let bytes: [u8; 4] = self.read(4).try_into().unwrap();
        f32::from_le_bytes(bytes)
    }

    pub fn read_string_ref(&mut self) -> String {
        let ptr = self.read_u64();
        if ptr == 0 {
            String::new()
        } else {
            read_pascal_string(self.data, ptr as usize)
        }
    }
}

pub struct SeekContext<'a, 'b> {
    pub stream: &'a mut ReadStream<'b>,
    original_offset: usize,
}

impl<'a, 'b> SeekContext<'a, 'b> {
    pub fn new(stream: &'a mut ReadStream<'b>, offset: usize) -> Self {
        let original_offset = stream.tell();
        stream.seek(offset);
        Self { stream, original_offset }
    }
}

impl<'a, 'b> Drop for SeekContext<'a, 'b> {
    fn drop(&mut self) {
        self.stream.seek(self.original_offset);
    }
}

#[derive(Clone)]
pub struct StringRef {
    pub offset: usize,
    pub is_header_name: bool,
}

pub struct WriteStream<'a, W: Write + Seek> {
    stream: &'a mut W,
    pub pos: usize,
    pointers: BTreeSet<usize>,
    strings: BTreeMap<String, Vec<StringRef>>,
    relocation_table_offset: usize,
}

impl<'a, W: Write + Seek> WriteStream<'a, W> {
    pub fn new(stream: &'a mut W) -> Self {
        let mut strings = BTreeMap::new();
        strings.insert(String::new(), Vec::new());
        Self {
            stream,
            pos: 0,
            pointers: BTreeSet::new(),
            strings,
            relocation_table_offset: 0,
        }
    }

    pub fn tell(&self) -> usize {
        self.pos
    }

    pub fn seek(&mut self, pos: usize) {
        self.stream.seek(SeekFrom::Start(pos as u64)).unwrap();
        self.pos = pos;
    }

    pub fn align(&mut self, align: usize) {
        let new_pos = align_up(self.pos, align);
        let diff = new_pos - self.pos;
        if diff > 0 {
            let padding = vec![0u8; diff];
            self.write_bytes(&padding);
        }
    }

    pub fn skip(&mut self, n: usize) {
        let padding = vec![0u8; n];
        self.write_bytes(&padding);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.stream.write_all(data).unwrap();
        self.pos += data.len();
    }

    pub fn write_u8(&mut self, val: u8) {
        self.write_bytes(&[val]);
    }

    pub fn write_u16(&mut self, val: u16) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn write_s16(&mut self, val: i16) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn write_u32(&mut self, val: u32) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn write_s32(&mut self, val: i32) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn write_f32(&mut self, val: f32) {
        self.write_bytes(&val.to_le_bytes());
    }

    pub fn register_pointer(&mut self, offset: usize) {
        self.pointers.insert(offset);
    }

    pub fn write_nullptr(&mut self, register: bool) {
        if register {
            self.register_pointer(self.tell());
        }
        self.write_u64(0);
    }

    pub fn write_string_ref(&mut self, data: &str, is_header_name: bool) {
        let tell = self.tell();
        self.strings.entry(data.to_string()).or_default().push(StringRef {
            offset: tell,
            is_header_name,
        });
        if is_header_name {
            self.write_u32(0xFFFFFFFF);
        } else {
            self.register_pointer(tell);
            self.write_u64(0xFFFFFFFFFFFFFFFF);
        }
    }

    pub fn write_placeholder_u16(&mut self) -> PlaceholderWriter {
        let offset = self.tell();
        self.write_u16(0xFFFF);
        PlaceholderWriter { offset }
    }

    pub fn write_placeholder_u32(&mut self) -> PlaceholderWriter {
        let offset = self.tell();
        self.write_u32(0xFFFFFFFF);
        PlaceholderWriter { offset }
    }

    pub fn write_placeholder_u64(&mut self) -> PlaceholderWriter {
        let offset = self.tell();
        self.write_u64(0xFFFFFFFFFFFFFFFF);
        PlaceholderWriter { offset }
    }

    pub fn write_placeholder_ptr(&mut self) -> PlaceholderWriter {
        let offset = self.tell();
        self.register_pointer(offset);
        self.write_u64(0xFFFFFFFFFFFFFFFF);
        PlaceholderWriter { offset }
    }

    pub fn write_placeholder_ptr_if(&mut self, condition: bool, register: bool) -> Option<PlaceholderWriter> {
        if condition {
            Some(self.write_placeholder_ptr())
        } else {
            self.write_nullptr(register);
            None
        }
    }

    pub fn get_relocation_table_offset(&self) -> usize {
        self.relocation_table_offset
    }

    pub fn finalise(&mut self) {
        self.align(8);
        self.write_string_pool();
        
        let data_end = self.tell();
        self.align(8);
        self.write_relocation_table(data_end);
    }

    fn write_string_pool(&mut self) {
        self.write_bytes(b"STR ");
        self.write_u32(0);
        self.write_u64(0);
        self.write_u32((self.strings.len() - 1) as u32);

        // Sort strings identically to Python: bitwise backwards
        let mut keys: Vec<String> = self.strings.keys().cloned().collect();
        keys.sort_by(|a, b| {
            fn sort_string(s: &str) -> String {
                let bytes = s.as_bytes();
                if bytes.is_empty() {
                    return "0".to_string();
                }
                let mut bin_str = String::new();
                for &b in bytes {
                    bin_str.push_str(&format!("{:08b}", b));
                }
                let trimmed = bin_str.trim_start_matches('0');
                if trimmed.is_empty() {
                    "0".to_string()
                } else {
                    trimmed.chars().rev().collect::<String>()
                }
            }
            sort_string(a).cmp(&sort_string(b))
        });

        for key in keys {
            let offset = self.tell();
            let refs = self.strings.get(&key).unwrap().clone();
            for r in refs {
                self.seek(r.offset);
                if r.is_header_name {
                    self.write_u32((offset + 2) as u32);
                } else {
                    self.write_u64(offset as u64);
                }
            }
            self.seek(offset);
            self.write_bytes(&pascal_string(&key));
            self.align(2);
        }
    }

    fn write_relocation_table(&mut self, data_end: usize) {
        self.relocation_table_offset = self.tell();
        self.write_bytes(b"RELT");
        self.write_u32(self.relocation_table_offset as u32);
        self.write_u32(1);
        self.write_u32(0);

        self.write_u64(0);
        self.write_u32(0);
        self.write_u32(data_end as u32);
        self.write_u32(0);
        let mut num_entries_writer = self.write_placeholder_u32();

        let mut num_entries = 0;
        let pointers_list: Vec<usize> = self.pointers.iter().cloned().collect();
        let mut pointers_set = self.pointers.clone();

        for p in pointers_list {
            if !pointers_set.contains(&p) {
                continue;
            }
            let mut flag: u32 = 0;
            for i in 0..0x20 {
                let address = p + 8 * i;
                if pointers_set.remove(&address) {
                    flag |= 1 << i;
                }
            }
            self.write_u32(p as u32);
            self.write_u32(flag);
            num_entries += 1;
        }

        num_entries_writer.write_u32(self, num_entries as u32);
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlaceholderWriter {
    pub offset: usize,
}

impl PlaceholderWriter {
    pub fn write_bytes<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>, data: &[u8]) {
        let current_pos = stream.tell();
        stream.seek(self.offset);
        stream.write_bytes(data);
        stream.seek(current_pos);
    }

    pub fn write_u16<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>, val: u16) {
        self.write_bytes(stream, &val.to_le_bytes());
    }

    pub fn write_u32<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>, val: u32) {
        self.write_bytes(stream, &val.to_le_bytes());
    }

    pub fn write_u64<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>, val: u64) {
        self.write_bytes(stream, &val.to_le_bytes());
    }

    pub fn write_current_offset<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        let tell = stream.tell();
        self.write_u64(stream, tell as u64);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Index<T> {
    pub v: Option<T>,
    pub idx: u16,
}

impl<T> Index<T> {
    pub fn new(idx: u16) -> Self {
        Self { v: None, idx }
    }
    
    pub fn from_value(v: Option<T>) -> Self {
        Self { v, idx: 0xFFFF }
    }
}

impl<T> Default for Index<T> {
    fn default() -> Self {
        Self { v: None, idx: 0xFFFF }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RequiredIndex<T> {
    pub v: Option<T>, // Rust requires Option for uninitialized states, or we just keep idx.
    pub idx: u16,
}

impl<T> RequiredIndex<T> {
    pub fn new(idx: u16) -> Self {
        Self { v: None, idx }
    }

    pub fn from_value(v: T) -> Self {
        Self { v: Some(v), idx: 0xFFFF }
    }
}

impl<T> Default for RequiredIndex<T> {
    fn default() -> Self {
        Self { v: None, idx: 0xFFFF }
    }
}
