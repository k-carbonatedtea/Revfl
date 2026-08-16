use crate::actor::Actor;
use crate::dic::{DicReader, DicWriter};
use crate::entry_point::EntryPoint;
use crate::event::Event;
use crate::util::{BinaryObject, ReadStream, SeekContext, WriteStream};
use std::io::{Seek, Write};

/// Represents a logic flowchart containing actors, events, and entry points.
/// 
/// 表示一个包含角色 (actors)、事件 (events) 和入口点 (entry points) 的逻辑流程图。
#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Flowchart {
    /// The name of the flowchart. / 流程图的名称。
    pub name: String,
    /// The list of actors involved in the flowchart. / 参与流程图的角色列表。
    pub actors: Vec<Actor>,
    /// The list of events defining the flowchart logic. / 定义流程图逻辑的事件列表。
    pub events: Vec<Event>,
    /// The entry points where execution can begin. / 执行开始的入口点列表。
    pub entry_points: Vec<EntryPoint>,
}

impl Flowchart {
    /// Creates a new, empty `Flowchart`.
    /// 
    /// 创建一个新的、空的 `Flowchart`。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            actors: Vec::new(),
            events: Vec::new(),
            entry_points: Vec::new(),
        }
    }

    fn get_action_count(&self) -> u16 {
        self.actors.iter().map(|a| a.actions.len() as u16).sum()
    }

    fn get_query_count(&self) -> u16 {
        self.actors.iter().map(|a| a.queries.len() as u16).sum()
    }
}

impl BinaryObject for Flowchart {
    fn read(&mut self, stream: &mut ReadStream) {
        let _magic = stream.read_u32(); // EVFL
        let _string_pool_offset = stream.read_u32();
        assert_eq!(stream.read_u32(), 0);
        assert_eq!(stream.read_u32(), 0);
        let num_actors = stream.read_u16();
        let _num_actions = stream.read_u16();
        let _num_queries = stream.read_u16();
        let num_events = stream.read_u16();
        let num_entry_points = stream.read_u16();
        assert_eq!(stream.read_u16(), 0);
        assert_eq!(stream.read_u16(), 0);
        assert_eq!(stream.read_u16(), 0);

        self.name = stream.read_string_ref();
        
        let actors_ptr = stream.read_u64();
        if actors_ptr != 0 && num_actors > 0 {
            let ctx = SeekContext::new(stream, actors_ptr as usize);
            for _ in 0..num_actors {
                let mut actor = Actor::new();
                actor.read(ctx.stream);
                self.actors.push(actor);
            }
        }

        let events_ptr = stream.read_u64();
        if events_ptr != 0 && num_events > 0 {
            let ctx = SeekContext::new(stream, events_ptr as usize);
            for _ in 0..num_events {
                let mut event = Event::new("", crate::event::EventData::Join(crate::event::JoinEvent{nxt: crate::util::Index::new(0)})); // dummy init
                event.read(ctx.stream);
                self.events.push(event);
            }
        }

        let dic_ptr = stream.read_u64();
        let mut entry_point_dic = DicReader::new();
        if dic_ptr != 0 {
            let ctx = SeekContext::new(stream, dic_ptr as usize);
            entry_point_dic.read(ctx.stream);
        }
        assert_eq!(entry_point_dic.items.len(), num_entry_points as usize);

        let entry_points_ptr = stream.read_u64();
        if entry_points_ptr != 0 {
            let ctx = SeekContext::new(stream, entry_points_ptr as usize);
            for name in entry_point_dic.items {
                let mut ep = EntryPoint::new(&name);
                ep.read(ctx.stream);
                self.entry_points.push(ep);
            }
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        let self_offset = stream.tell();
        stream.write_bytes(b"EVFL");
        let mut string_pool_rel_offset = stream.write_placeholder_u32();
        stream.write_u32(0);
        stream.write_u32(0);
        stream.write_u16(self.actors.len() as u16);
        stream.write_u16(self.get_action_count());
        stream.write_u16(self.get_query_count());
        stream.write_u16(self.events.len() as u16);
        stream.write_u16(self.entry_points.len() as u16);
        stream.write_u16(0);
        stream.write_u16(0);
        stream.write_u16(0);
        stream.write_string_ref(&self.name, false);
        
        let mut actors_offset_writer = stream.write_placeholder_ptr_if(!self.actors.is_empty(), true);
        let mut events_offset_writer = stream.write_placeholder_ptr_if(!self.events.is_empty(), true);
        
        let mut entry_points_dic = DicWriter::new();
        entry_points_dic.write_placeholder_offset(stream);
        let mut entry_points_offset_writer = stream.write_placeholder_ptr_if(!self.entry_points.is_empty(), true);

        if let Some(mut w) = actors_offset_writer.take() {
            w.write_current_offset(stream);
            for actor in &mut self.actors {
                actor.write(stream);
            }
        }

        if let Some(mut w) = events_offset_writer.take() {
            w.write_current_offset(stream);
            for event in &mut self.events {
                event.write(stream);
            }
        }

        for ep in &self.entry_points {
            entry_points_dic.insert(&ep.name);
        }
        entry_points_dic.write(stream);
        stream.align(8);

        if let Some(mut w) = entry_points_offset_writer.take() {
            w.write_current_offset(stream);
            for ep in &mut self.entry_points {
                ep.write(stream);
            }
        }

        for event in &mut self.events {
            stream.align(8);
            event.write_extra_data(stream);
        }

        for actor in &mut self.actors {
            stream.align(8);
            actor.write_extra_data(stream);
        }

        for ep in &mut self.entry_points {
            stream.align(8);
            ep.write_extra_data(stream);
        }

        stream.align(8);
        string_pool_rel_offset.write_u32(stream, (stream.tell() - self_offset) as u32);
    }
}
