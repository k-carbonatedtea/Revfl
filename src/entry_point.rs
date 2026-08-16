use crate::util::{BinaryObject, Index, PlaceholderWriter, ReadStream, SeekContext, WriteStream};
use std::io::{Seek, Write};

/// Represents an entry point in a flowchart where execution can start.
/// 
/// 表示流程图中可以开始执行的入口点。
#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EntryPoint {
    pub name: String,
    pub main_event: Index<usize>, // index of Event
    pub sub_flow_event_indices: Vec<u16>,
    pub sub_flow_event_indices_offset_writer: Option<PlaceholderWriter>,
}

impl EntryPoint {
    /// Creates a new `EntryPoint` with the specified name.
    /// 
    /// 用指定的名称创建一个新的 `EntryPoint`。
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            main_event: Index::new(0xFFFF),
            sub_flow_event_indices: Vec::new(),
            sub_flow_event_indices_offset_writer: None,
        }
    }
}

impl BinaryObject for EntryPoint {
    fn read(&mut self, stream: &mut ReadStream) {
        let sub_flow_event_indices_offset = stream.read_u64();
        assert_eq!(stream.read_u64(), 0); // x8
        assert_eq!(stream.read_u64(), 0); // ptr_x10
        let num_sub_flow_event_indices = stream.read_u16();
        assert_eq!(stream.read_u16(), 0); // x1a
        self.main_event.idx = stream.read_u16();
        assert_eq!(stream.read_u16(), 0); // x1e

        if num_sub_flow_event_indices > 0 {
            assert_ne!(sub_flow_event_indices_offset, 0);
            let ctx = SeekContext::new(stream, sub_flow_event_indices_offset as usize);
            for _ in 0..num_sub_flow_event_indices {
                self.sub_flow_event_indices.push(ctx.stream.read_u16());
            }
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        self.sub_flow_event_indices_offset_writer = stream.write_placeholder_ptr_if(!self.sub_flow_event_indices.is_empty(), true);
        stream.write_u64(0); // x8
        stream.write_nullptr(true); // ptr_x10
        stream.write_u16(self.sub_flow_event_indices.len() as u16);
        stream.write_u16(0); // x1a
        stream.write_u16(self.main_event.idx);
        stream.write_u16(0); // x1e
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        if let Some(mut w) = self.sub_flow_event_indices_offset_writer.take() {
            w.write_current_offset(stream);
            for idx in &self.sub_flow_event_indices {
                stream.write_u16(*idx);
            }
            stream.align(8);
        }
        stream.skip(0x18);
    }
}
