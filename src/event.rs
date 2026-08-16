use crate::container::Container;
use crate::enums::EventType;
use crate::util::{BinaryObject, Index, PlaceholderWriter, ReadStream, RequiredIndex, SeekContext, WriteStream};
use indexmap::IndexMap;
use std::io::{Seek, Write};

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ActionEvent {
    pub nxt: Index<usize>, // Points to event idx
    pub actor: RequiredIndex<usize>, // Points to actor idx
    pub actor_action: RequiredIndex<usize>, // Points to action idx inside actor
    pub params: Option<Container>,
    pub params_offset_writer: Option<PlaceholderWriter>,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SwitchEvent {
    pub actor: RequiredIndex<usize>,
    pub actor_query: RequiredIndex<usize>,
    pub params: Option<Container>,
    pub cases: IndexMap<u32, RequiredIndex<usize>>, // Points to event idx
    pub params_offset_writer: Option<PlaceholderWriter>,
    pub cases_offset_writer: Option<PlaceholderWriter>,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ForkEvent {
    pub join: RequiredIndex<usize>,
    pub forks: Vec<RequiredIndex<usize>>,
    pub forks_offset_writer: Option<PlaceholderWriter>,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct JoinEvent {
    pub nxt: Index<usize>,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SubFlowEvent {
    pub nxt: Index<usize>,
    pub params: Option<Container>,
    pub res_flowchart_name: String,
    pub entry_point_name: String,
    pub params_offset_writer: Option<PlaceholderWriter>,
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum EventData {
    Action(ActionEvent),
    Switch(SwitchEvent),
    Fork(ForkEvent),
    Join(JoinEvent),
    SubFlow(SubFlowEvent),
}

/// Represents a node in the flowchart, such as an action, switch, fork, join, or subflow.
/// 
/// 表示流程图中的一个节点，例如动作 (action)、切换 (switch)、分支 (fork)、合并 (join) 或子流程 (subflow)。
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub name: String,
    pub data: EventData,
}

impl Event {
    /// Creates a new `Event` with the specified name and data.
    /// 
    /// 用指定的名称和数据创建一个新的 `Event`。
    pub fn new(name: &str, data: EventData) -> Self {
        Self {
            name: name.to_string(),
            data,
        }
    }
}

fn should_write_params(params: &Option<Container>) -> bool {
    if let Some(p) = params {
        !p.data.is_empty()
    } else {
        false
    }
}

impl BinaryObject for Event {
    fn read(&mut self, stream: &mut ReadStream) {
        self.name = stream.read_string_ref();
        let etype = stream.read_u8();
        stream.skip(1); // Padding

        match etype {
            0 => { // Action
                let mut data = ActionEvent {
                    nxt: Index::new(stream.read_u16()),
                    actor: RequiredIndex::new(stream.read_u16()),
                    actor_action: RequiredIndex::new(stream.read_u16()),
                    params: None,
                    params_offset_writer: None,
                };
                let params_ptr = stream.read_u64();
                if params_ptr != 0 {
                    let ctx = SeekContext::new(stream, params_ptr as usize);
                    let mut params = Container::new();
                    params.read(ctx.stream);
                    data.params = Some(params);
                }
                assert_eq!(stream.read_u64(), 0);
                assert_eq!(stream.read_u64(), 0);
                self.data = EventData::Action(data);
            }
            1 => { // Switch
                let num_cases = stream.read_u16();
                let mut data = SwitchEvent {
                    actor: RequiredIndex::new(stream.read_u16()),
                    actor_query: RequiredIndex::new(stream.read_u16()),
                    params: None,
                    cases: IndexMap::new(),
                    params_offset_writer: None,
                    cases_offset_writer: None,
                };
                let params_ptr = stream.read_u64();
                if params_ptr != 0 {
                    let ctx = SeekContext::new(stream, params_ptr as usize);
                    let mut params = Container::new();
                    params.read(ctx.stream);
                    data.params = Some(params);
                }
                let cases_offset = stream.read_u64();
                if cases_offset != 0 {
                    let ctx = SeekContext::new(stream, cases_offset as usize);
                    for _ in 0..num_cases {
                        let value = ctx.stream.read_u32();
                        let event_idx = ctx.stream.read_u16();
                        data.cases.insert(value, RequiredIndex::new(event_idx));
                        ctx.stream.align(8);
                    }
                }
                assert_eq!(stream.read_u64(), 0);
                self.data = EventData::Switch(data);
            }
            2 => { // Fork
                let num_forks = stream.read_u16();
                let mut data = ForkEvent {
                    join: RequiredIndex::new(stream.read_u16()),
                    forks: Vec::new(),
                    forks_offset_writer: None,
                };
                assert_eq!(stream.read_u16(), 0);
                let forks_offset = stream.read_u64();
                if num_forks > 0 && forks_offset != 0 {
                    let ctx = SeekContext::new(stream, forks_offset as usize);
                    for _ in 0..num_forks {
                        data.forks.push(RequiredIndex::new(ctx.stream.read_u16()));
                    }
                }
                assert_eq!(stream.read_u64(), 0);
                assert_eq!(stream.read_u64(), 0);
                self.data = EventData::Fork(data);
            }
            3 => { // Join
                let data = JoinEvent {
                    nxt: Index::new(stream.read_u16()),
                };
                assert_eq!(stream.read_u16(), 0);
                assert_eq!(stream.read_u16(), 0);
                assert_eq!(stream.read_u64(), 0);
                assert_eq!(stream.read_u64(), 0);
                assert_eq!(stream.read_u64(), 0);
                self.data = EventData::Join(data);
            }
            4 => { // SubFlow
                let mut data = SubFlowEvent {
                    nxt: Index::new(stream.read_u16()),
                    params: None,
                    res_flowchart_name: String::new(),
                    entry_point_name: String::new(),
                    params_offset_writer: None,
                };
                assert_eq!(stream.read_u16(), 0);
                assert_eq!(stream.read_u16(), 0);
                let params_ptr = stream.read_u64();
                if params_ptr != 0 {
                    let ctx = SeekContext::new(stream, params_ptr as usize);
                    let mut params = Container::new();
                    params.read(ctx.stream);
                    data.params = Some(params);
                }
                data.res_flowchart_name = stream.read_string_ref();
                data.entry_point_name = stream.read_string_ref();
                self.data = EventData::SubFlow(data);
            }
            _ => panic!("Unknown event type"),
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_string_ref(&self.name, false);
        let etype = match &self.data {
            EventData::Action(_) => EventType::Action,
            EventData::Switch(_) => EventType::Switch,
            EventData::Fork(_) => EventType::Fork,
            EventData::Join(_) => EventType::Join,
            EventData::SubFlow(_) => EventType::SubFlow,
        };
        stream.write_u8(etype as u8);
        stream.write_u8(0);

        match &mut self.data {
            EventData::Action(a) => {
                stream.write_u16(a.nxt.idx);
                stream.write_u16(a.actor.idx);
                stream.write_u16(a.actor_action.idx);
                a.params_offset_writer = stream.write_placeholder_ptr_if(should_write_params(&a.params), false);
                stream.write_u64(0);
                stream.write_u64(0);
            }
            EventData::Switch(s) => {
                stream.write_u16(s.cases.len() as u16);
                stream.write_u16(s.actor.idx);
                stream.write_u16(s.actor_query.idx);
                s.params_offset_writer = stream.write_placeholder_ptr_if(should_write_params(&s.params), false);
                s.cases_offset_writer = stream.write_placeholder_ptr_if(!s.cases.is_empty(), true);
                stream.write_u64(0);
            }
            EventData::Fork(f) => {
                stream.write_u16(f.forks.len() as u16);
                stream.write_u16(f.join.idx);
                stream.write_u16(0);
                f.forks_offset_writer = Some(stream.write_placeholder_ptr());
                stream.write_u64(0);
                stream.write_u64(0);
            }
            EventData::Join(j) => {
                stream.write_u16(j.nxt.idx);
                stream.write_u16(0);
                stream.write_u16(0);
                stream.write_u64(0);
                stream.write_u64(0);
                stream.write_u64(0);
            }
            EventData::SubFlow(s) => {
                stream.write_u16(s.nxt.idx);
                stream.write_u16(0);
                stream.write_u16(0);
                s.params_offset_writer = stream.write_placeholder_ptr_if(should_write_params(&s.params), false);
                stream.write_string_ref(&s.res_flowchart_name, false);
                stream.write_string_ref(&s.entry_point_name, false);
            }
        }
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        match &mut self.data {
            EventData::Action(a) => {
                if let Some(mut w) = a.params_offset_writer.take() {
                    w.write_current_offset(stream);
                    if let Some(params) = &mut a.params {
                        params.write(stream);
                    }
                }
            }
            EventData::Switch(s) => {
                if let Some(mut w) = s.cases_offset_writer.take() {
                    stream.align(8);
                    w.write_current_offset(stream);
                    for (value, event) in &s.cases {
                        stream.write_u32(*value);
                        stream.write_u16(event.idx);
                        stream.align(8);
                    }
                }
                if let Some(mut w) = s.params_offset_writer.take() {
                    w.write_current_offset(stream);
                    if let Some(params) = &mut s.params {
                        params.write(stream);
                    }
                }
            }
            EventData::Fork(f) => {
                if let Some(mut w) = f.forks_offset_writer.take() {
                    w.write_current_offset(stream);
                    for fork in &f.forks {
                        stream.write_u16(fork.idx);
                    }
                    stream.align(8);
                }
            }
            EventData::Join(_) => {}
            EventData::SubFlow(s) => {
                if let Some(mut w) = s.params_offset_writer.take() {
                    w.write_current_offset(stream);
                    if let Some(params) = &mut s.params {
                        params.write(stream);
                    }
                }
            }
        }
    }
}
