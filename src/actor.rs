use crate::common::{ActorIdentifier, StringHolder};
use crate::container::Container;
use crate::util::{BinaryObject, Index, PlaceholderWriter, ReadStream, SeekContext, WriteStream};
use std::io::{Seek, Write};

/// Represents an entity (Actor) involved in the event flow, capable of performing actions or answering queries.
/// 
/// 表示参与事件流的实体 (Actor)，能够执行动作 (actions) 或回答查询 (queries)。
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Actor {
    pub identifier: ActorIdentifier,
    pub argument_name: String,
    pub argument_entry_point: Index<usize>, // points to entry point
    pub actions: Vec<StringHolder>,
    pub queries: Vec<StringHolder>,
    pub params: Option<Container>,
    pub concurrent_clips: u16,

    pub actions_offset_writer: Option<PlaceholderWriter>,
    pub actions_offset: Option<usize>,
    pub queries_offset_writer: Option<PlaceholderWriter>,
    pub queries_offset: Option<usize>,
    pub params_offset_writer: Option<PlaceholderWriter>,
    pub params_offset: Option<usize>,
}

impl Actor {
    /// Creates a new, empty `Actor`.
    /// 
    /// 创建一个新的、空的 `Actor`。
    pub fn new() -> Self {
        Self {
            identifier: ActorIdentifier::default(),
            argument_name: String::new(),
            argument_entry_point: Index::new(0xFFFF),
            actions: Vec::new(),
            queries: Vec::new(),
            params: None,
            concurrent_clips: 0xFFFF,
            actions_offset_writer: None,
            actions_offset: None,
            queries_offset_writer: None,
            queries_offset: None,
            params_offset_writer: None,
            params_offset: None,
        }
    }
}

impl BinaryObject for Actor {
    fn read(&mut self, stream: &mut ReadStream) {
        self.identifier.read(stream);
        self.argument_name = stream.read_string_ref();
        let actions_offset = stream.read_u64();
        let queries_offset = stream.read_u64();
        let params_ptr = stream.read_u64();
        if params_ptr != 0 {
            let ctx = SeekContext::new(stream, params_ptr as usize);
            let mut params = Container::new();
            params.read(ctx.stream);
            self.params = Some(params);
        }
        let num_actions = stream.read_u16();
        let num_queries = stream.read_u16();
        self.argument_entry_point.idx = stream.read_u16();
        self.concurrent_clips = stream.read_u16();

        if actions_offset != 0 {
            let ctx = SeekContext::new(stream, actions_offset as usize);
            for _ in 0..num_actions {
                self.actions.push(StringHolder(ctx.stream.read_string_ref()));
            }
        }
        if queries_offset != 0 {
            let ctx = SeekContext::new(stream, queries_offset as usize);
            for _ in 0..num_queries {
                self.queries.push(StringHolder(ctx.stream.read_string_ref()));
            }
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        self.identifier.write(stream);
        stream.write_string_ref(&self.argument_name, false);
        self.actions_offset_writer = stream.write_placeholder_ptr_if(!self.actions.is_empty(), true);
        self.queries_offset_writer = stream.write_placeholder_ptr_if(!self.queries.is_empty(), true);
        self.params_offset_writer = stream.write_placeholder_ptr_if(self.params.is_some(), false);
        stream.write_u16(self.actions.len() as u16);
        stream.write_u16(self.queries.len() as u16);
        stream.write_u16(self.argument_entry_point.idx);
        stream.write_u16(self.concurrent_clips);

        if let Some(offset) = self.actions_offset {
            if let Some(mut w) = self.actions_offset_writer.take() {
                w.write_u64(stream, offset as u64);
                self.actions_offset_writer = Some(w);
            }
        }
        if let Some(offset) = self.queries_offset {
            if let Some(mut w) = self.queries_offset_writer.take() {
                w.write_u64(stream, offset as u64);
                self.queries_offset_writer = Some(w);
            }
        }
        if let Some(offset) = self.params_offset {
            if let Some(mut w) = self.params_offset_writer.take() {
                w.write_u64(stream, offset as u64);
                self.params_offset_writer = Some(w);
            }
        }
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        if let Some(params) = &mut self.params {
            stream.align(8);
            if let Some(mut w) = self.params_offset_writer.take() {
                w.write_current_offset(stream);
                self.params_offset_writer = Some(w);
            } else {
                self.params_offset = Some(stream.tell());
            }
            params.write(stream);
        }

        if !self.actions.is_empty() {
            stream.align(8);
            if let Some(mut w) = self.actions_offset_writer.take() {
                w.write_current_offset(stream);
                self.actions_offset_writer = Some(w);
            } else {
                self.actions_offset = Some(stream.tell());
            }
            for s in &self.actions {
                stream.write_string_ref(&s.0, false);
            }
        }

        if !self.queries.is_empty() {
            stream.align(8);
            if let Some(mut w) = self.queries_offset_writer.take() {
                w.write_current_offset(stream);
                self.queries_offset_writer = Some(w);
            } else {
                self.queries_offset = Some(stream.tell());
            }
            for s in &self.queries {
                stream.write_string_ref(&s.0, false);
            }
        }
    }
}
