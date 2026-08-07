use crate::common::{ActorIdentifier, Argument};
use crate::dic::{DicReader, DicWriter};
use crate::enums::ContainerDataType;
use crate::util::{pascal_string, BinaryObject, ReadStream, SeekContext, WriteStream};
use indexmap::IndexMap;
use std::io::{Seek, Write};

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ContainerData {
    Int(i32),
    Bool(bool),
    Float(f32),
    String(String),
    Argument(Argument),
    ActorIdentifier(ActorIdentifier),
    IntArray(Vec<i32>),
    BoolArray(Vec<bool>),
    FloatArray(Vec<f32>),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Container {
    pub data: IndexMap<String, ContainerData>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            data: IndexMap::new(),
        }
    }
}

impl BinaryObject for Container {
    fn read(&mut self, stream: &mut ReadStream) {
        let data_type = stream.read_u8();
        assert_eq!(data_type, ContainerDataType::Container as u8, "Invalid data type");
        stream.skip(1);
        let _num_items = stream.read_u16();
        let x4 = stream.read_u32();
        assert_eq!(x4, 0);

        let mut dic = DicReader::new();
        let dic_ptr = stream.read_u64();
        if dic_ptr != 0 {
            let ctx = SeekContext::new(stream, dic_ptr as usize);
            dic.read(ctx.stream);
        }

        for name in dic.items {
            let item_offset = stream.read_u64();
            let ctx = SeekContext::new(stream, item_offset as usize);
            let item = read_item(ctx.stream);
            self.data.insert(name, item);
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_u8(ContainerDataType::Container as u8);
        stream.write_u8(0);
        stream.write_u16(self.data.len() as u16);
        stream.write_u32(0);

        let mut dic = DicWriter::new();
        for key in self.data.keys() {
            dic.insert(key);
        }
        dic.write_placeholder_offset(stream);

        let mut item_ptr_writers = Vec::new();
        for _ in 0..self.data.len() {
            item_ptr_writers.push(stream.write_placeholder_ptr());
        }

        dic.write(stream);

        for (mut ptr_writer, value) in item_ptr_writers.into_iter().zip(self.data.values()) {
            stream.align(8);
            ptr_writer.write_current_offset(stream);
            write_item(stream, value);
        }
    }
}

fn write_item_common_header<W: Write + Seek>(
    stream: &mut WriteStream<W>,
    data_type: ContainerDataType,
    num_items: u16,
) {
    stream.write_u8(data_type as u8);
    stream.write_u8(0);
    stream.write_u16(num_items);
    stream.write_u32(0);
    stream.write_u64(0);
}

fn write_item<W: Write + Seek>(stream: &mut WriteStream<W>, value: &ContainerData) {
    match value {
        ContainerData::Bool(v) => {
            write_item_common_header(stream, ContainerDataType::Bool, 1);
            stream.write_u32(if *v { 0x80000001 } else { 0 });
        }
        ContainerData::Int(v) => {
            write_item_common_header(stream, ContainerDataType::Int, 1);
            stream.write_s32(*v);
        }
        ContainerData::Float(v) => {
            write_item_common_header(stream, ContainerDataType::Float, 1);
            stream.write_f32(*v);
        }
        ContainerData::String(v) => {
            write_item_common_header(stream, ContainerDataType::String, 1);
            let mut ptr_writer = stream.write_placeholder_ptr();
            ptr_writer.write_current_offset(stream);
            stream.write_bytes(&pascal_string(v));
        }
        ContainerData::Argument(v) => {
            write_item_common_header(stream, ContainerDataType::Argument, 1);
            let mut ptr_writer = stream.write_placeholder_ptr();
            ptr_writer.write_current_offset(stream);
            stream.write_bytes(&pascal_string(&v.0));
        }
        ContainerData::ActorIdentifier(v) => {
            write_item_common_header(stream, ContainerDataType::ActorIdentifier, 2);
            let mut ptr_writer1 = stream.write_placeholder_ptr();
            let mut ptr_writer2 = stream.write_placeholder_ptr();
            ptr_writer1.write_current_offset(stream);
            stream.write_bytes(&pascal_string(&v.name));
            stream.align(2);
            ptr_writer2.write_current_offset(stream);
            stream.write_bytes(&pascal_string(&v.sub_name));
        }
        ContainerData::IntArray(v) => {
            write_item_common_header(stream, ContainerDataType::IntArray, v.len() as u16);
            for &val in v {
                stream.write_s32(val);
            }
        }
        ContainerData::BoolArray(v) => {
            write_item_common_header(stream, ContainerDataType::BoolArray, v.len() as u16);
            for &val in v {
                stream.write_u32(if val { 1 } else { 0 });
            }
        }
        ContainerData::FloatArray(v) => {
            write_item_common_header(stream, ContainerDataType::FloatArray, v.len() as u16);
            for &val in v {
                stream.write_f32(val);
            }
        }
        ContainerData::StringArray(v) => {
            write_item_common_header(stream, ContainerDataType::StringArray, v.len() as u16);
            let mut ptr_writers = Vec::new();
            for _ in 0..v.len() {
                ptr_writers.push(stream.write_placeholder_ptr());
            }
            for (mut ptr_writer, val) in ptr_writers.into_iter().zip(v.iter()) {
                stream.align(8);
                ptr_writer.write_current_offset(stream);
                stream.write_bytes(&pascal_string(val));
            }
        }
    }
}

fn read_item(stream: &mut ReadStream) -> ContainerData {
    let dt_val = stream.read_u8();
    let data_type = ContainerDataType::from_u8(dt_val).unwrap();
    stream.skip(1);
    let num_items = stream.read_u16();
    let x4 = stream.read_u32();
    assert_eq!(x4, 0);
    let dic_offset = stream.read_u64();
    assert_eq!(dic_offset, 0);

    match data_type {
        ContainerDataType::Int => ContainerData::Int(stream.read_s32()),
        ContainerDataType::IntArray => {
            let mut arr = Vec::new();
            for _ in 0..num_items {
                arr.push(stream.read_s32());
            }
            ContainerData::IntArray(arr)
        }
        ContainerDataType::Bool => ContainerData::Bool(stream.read_u32() != 0),
        ContainerDataType::BoolArray => {
            let mut arr = Vec::new();
            for _ in 0..num_items {
                arr.push(stream.read_u32() != 0);
            }
            ContainerData::BoolArray(arr)
        }
        ContainerDataType::Float => ContainerData::Float(stream.read_f32()),
        ContainerDataType::FloatArray => {
            let mut arr = Vec::new();
            for _ in 0..num_items {
                arr.push(stream.read_f32());
            }
            ContainerData::FloatArray(arr)
        }
        ContainerDataType::String => ContainerData::String(stream.read_string_ref()),
        ContainerDataType::StringArray => {
            let mut arr = Vec::new();
            for _ in 0..num_items {
                arr.push(stream.read_string_ref());
            }
            ContainerData::StringArray(arr)
        }
        ContainerDataType::Argument => ContainerData::Argument(Argument(stream.read_string_ref())),
        ContainerDataType::ActorIdentifier => {
            let mut actor_identifier = ActorIdentifier::default();
            actor_identifier.read(stream);
            ContainerData::ActorIdentifier(actor_identifier)
        }
        _ => panic!("Unhandled data type: {:?}", data_type),
    }
}
