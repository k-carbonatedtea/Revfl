use crate::actor::Actor;
use crate::container::Container;
use crate::enums::TriggerType;
use crate::util::{BinaryObject, PlaceholderWriter, ReadStream, RequiredIndex, SeekContext, WriteStream};
use std::io::{Seek, Write};

/// Represents a cutscene clip within a timeline.
/// 
/// 表示时间线中的一个过场动画剪辑 (Clip)。
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Clip {
    pub start_time: f32,
    pub duration: f32,
    pub actor: RequiredIndex<usize>,
    pub actor_action: RequiredIndex<usize>,
    pub actor_concurrent_clip: u8,
    pub params: Option<Container>,
    pub params_offset_writer: Option<PlaceholderWriter>,
}

impl Clip {
    /// Creates a new, empty `Clip`.
    /// 
    /// 创建一个新的、空的 `Clip`。
    pub fn new() -> Self {
        Self {
            start_time: -1.0,
            duration: -1.0,
            actor: RequiredIndex::new(0xFFFF),
            actor_action: RequiredIndex::new(0xFFFF),
            actor_concurrent_clip: 0xFF,
            params: None,
            params_offset_writer: None,
        }
    }
}

impl Default for Clip {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Clip {
    fn read(&mut self, stream: &mut ReadStream) {
        self.start_time = stream.read_f32();
        self.duration = stream.read_f32();
        self.actor.idx = stream.read_u16();
        self.actor_action.idx = stream.read_u16();
        self.actor_concurrent_clip = stream.read_u8();
        stream.skip(3);
        let params_ptr = stream.read_u64();
        if params_ptr != 0 {
            let ctx = SeekContext::new(stream, params_ptr as usize);
            let mut params = Container::new();
            params.read(ctx.stream);
            self.params = Some(params);
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_f32(self.start_time);
        stream.write_f32(self.duration);
        stream.write_u16(self.actor.idx);
        stream.write_u16(self.actor_action.idx);
        stream.write_u8(self.actor_concurrent_clip);
        stream.write_bytes(&[0, 0, 0]);
        self.params_offset_writer = stream.write_placeholder_ptr_if(self.params.is_some(), false);
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        if let Some(mut w) = self.params_offset_writer.take() {
            w.write_current_offset(stream);
            if let Some(params) = &mut self.params {
                params.write(stream);
            }
        }
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Oneshot {
    pub time: f32,
    pub actor: RequiredIndex<usize>,
    pub actor_action: RequiredIndex<usize>,
    pub params: Option<Container>,
    pub params_offset_writer: Option<PlaceholderWriter>,
}

impl Oneshot {
    pub fn new() -> Self {
        Self {
            time: -1.0,
            actor: RequiredIndex::new(0xFFFF),
            actor_action: RequiredIndex::new(0xFFFF),
            params: None,
            params_offset_writer: None,
        }
    }
}

impl Default for Oneshot {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Oneshot {
    fn read(&mut self, stream: &mut ReadStream) {
        self.time = stream.read_f32();
        self.actor.idx = stream.read_u16();
        self.actor_action.idx = stream.read_u16();
        stream.skip(8);
        let params_ptr = stream.read_u64();
        if params_ptr != 0 {
            let ctx = SeekContext::new(stream, params_ptr as usize);
            let mut params = Container::new();
            params.read(ctx.stream);
            self.params = Some(params);
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_f32(self.time);
        stream.write_u16(self.actor.idx);
        stream.write_u16(self.actor_action.idx);
        stream.write_u64(0);
        self.params_offset_writer = stream.write_placeholder_ptr_if(self.params.is_some(), false);
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        if let Some(mut w) = self.params_offset_writer.take() {
            w.write_current_offset(stream);
            if let Some(params) = &mut self.params {
                params.write(stream);
            }
        }
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cut {
    pub start_time: f32,
    pub x4: u32,
    pub name: String,
    pub params: Option<Container>,
    pub params_offset_writer: Option<PlaceholderWriter>,
}

impl Cut {
    pub fn new() -> Self {
        Self {
            start_time: -1.0,
            x4: 0xFFFFFFFF,
            name: String::new(),
            params: None,
            params_offset_writer: None,
        }
    }
}

impl Default for Cut {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Cut {
    fn read(&mut self, stream: &mut ReadStream) {
        self.start_time = stream.read_f32();
        self.x4 = stream.read_u32();
        self.name = stream.read_string_ref();
        let params_ptr = stream.read_u64();
        if params_ptr != 0 {
            let ctx = SeekContext::new(stream, params_ptr as usize);
            let mut params = Container::new();
            params.read(ctx.stream);
            self.params = Some(params);
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_f32(self.start_time);
        stream.write_u32(self.x4);
        stream.write_string_ref(&self.name, false);
        self.params_offset_writer = stream.write_placeholder_ptr_if(self.params.is_some(), false);
    }

    fn write_extra_data<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        if let Some(mut w) = self.params_offset_writer.take() {
            w.write_current_offset(stream);
            if let Some(params) = &mut self.params {
                params.write(stream);
            }
        }
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Trigger {
    pub clip: RequiredIndex<usize>,
    pub trigger_type: TriggerType,
    pub trigger_time: f32, // Used for sorting, not serialized directly
}

impl Trigger {
    pub fn new() -> Self {
        Self {
            clip: RequiredIndex::new(0xFFFF),
            trigger_type: TriggerType::Normal,
            trigger_time: 0.0,
        }
    }
}

impl Default for Trigger {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Trigger {
    fn read(&mut self, stream: &mut ReadStream) {
        self.clip.idx = stream.read_u16();
        let tval = stream.read_u8();
        self.trigger_type = match tval {
            0 => TriggerType::Normal,
            1 => TriggerType::Enter,
            2 => TriggerType::Leave,
            3 => TriggerType::Oneshot,
            _ => TriggerType::Normal,
        };
        stream.skip(1);
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_u16(self.clip.idx);
        stream.write_u8(self.trigger_type as u8);
        stream.write_u8(0);
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Subtimeline {
    pub name: String,
}

impl Subtimeline {
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }
}

impl Default for Subtimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Subtimeline {
    fn read(&mut self, stream: &mut ReadStream) {
        self.name = stream.read_string_ref();
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_string_ref(&self.name, false);
    }
}

/// Represents a timeline container, used for sequencing events over time (e.g., in cutscenes).
/// 
/// 表示一个时间线容器，用于按时间顺序排列事件（例如：过场动画中）。
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Timeline {
    pub name: String,
    pub duration: f32,
    pub actors: Vec<Actor>,
    pub clips: Vec<Clip>,
    pub oneshots: Vec<Oneshot>,
    pub triggers: Vec<Trigger>,
    pub subtimelines: Vec<Subtimeline>,
    pub cuts: Vec<Cut>,
    pub params: Option<Container>,
    pub self_offset: usize,
}

impl Timeline {
    /// Creates a new, empty `Timeline`.
    /// 
    /// 创建一个新的、空的 `Timeline`。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            duration: -1.0,
            actors: Vec::new(),
            clips: Vec::new(),
            oneshots: Vec::new(),
            triggers: Vec::new(),
            subtimelines: Vec::new(),
            cuts: Vec::new(),
            params: None,
            self_offset: 0,
        }
    }

    fn get_action_count(&self) -> u16 {
        self.actors.iter().map(|a| a.actions.len() as u16).sum()
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryObject for Timeline {
    fn read(&mut self, stream: &mut ReadStream) {
        let _magic = stream.read_u32(); // TLIN
        let _string_pool_offset = stream.read_u32();
        assert_eq!(stream.read_u32(), 0);
        assert_eq!(stream.read_u32(), 0);
        self.duration = stream.read_f32();
        let num_actors = stream.read_u16();
        let _num_actions = stream.read_u16();
        let num_clips = stream.read_u16();
        let num_oneshots = stream.read_u16();
        let num_subtimelines = stream.read_u16();
        let num_cuts = stream.read_u16();
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

        let clips_ptr = stream.read_u64();
        if clips_ptr != 0 && num_clips > 0 {
            let ctx = SeekContext::new(stream, clips_ptr as usize);
            for _ in 0..num_clips {
                let mut clip = Clip::new();
                clip.read(ctx.stream);
                self.clips.push(clip);
            }
        }

        let oneshots_ptr = stream.read_u64();
        if oneshots_ptr != 0 && num_oneshots > 0 {
            let ctx = SeekContext::new(stream, oneshots_ptr as usize);
            for _ in 0..num_oneshots {
                let mut oneshot = Oneshot::new();
                oneshot.read(ctx.stream);
                self.oneshots.push(oneshot);
            }
        }

        let triggers_ptr = stream.read_u64();
        if triggers_ptr != 0 && num_clips > 0 {
            let ctx = SeekContext::new(stream, triggers_ptr as usize);
            for _ in 0..(num_clips * 2) {
                let mut trigger = Trigger::new();
                trigger.read(ctx.stream);
                self.triggers.push(trigger);
            }
        }

        stream.align(8);
        
        let subtimelines_ptr = stream.read_u64();
        if subtimelines_ptr != 0 && num_subtimelines > 0 {
            let ctx = SeekContext::new(stream, subtimelines_ptr as usize);
            for _ in 0..num_subtimelines {
                let mut sub = Subtimeline::new();
                sub.read(ctx.stream);
                self.subtimelines.push(sub);
            }
        }

        let cuts_ptr = stream.read_u64();
        if cuts_ptr != 0 && num_cuts > 0 {
            let ctx = SeekContext::new(stream, cuts_ptr as usize);
            for _ in 0..num_cuts {
                let mut cut = Cut::new();
                cut.read(ctx.stream);
                self.cuts.push(cut);
            }
        }

        let params_ptr = stream.read_u64();
        if params_ptr != 0 {
            let ctx = SeekContext::new(stream, params_ptr as usize);
            let mut params = Container::new();
            params.read(ctx.stream);
            self.params = Some(params);
        }
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        for trigger in &mut self.triggers {
            if trigger.trigger_type == TriggerType::Enter {
                trigger.trigger_time = self.clips[trigger.clip.idx as usize].start_time;
            } else if trigger.trigger_type == TriggerType::Leave {
                let clip = &self.clips[trigger.clip.idx as usize];
                trigger.trigger_time = clip.start_time + clip.duration;
            } else {
                trigger.trigger_time = 0.0;
            }
        }
        self.triggers.sort_by(|a, b| a.trigger_time.partial_cmp(&b.trigger_time).unwrap_or(std::cmp::Ordering::Equal));

        for actor in &mut self.actors {
            actor.write_extra_data(stream);
            stream.align(8);
        }

        let mut param_offset = None;
        if let Some(params) = &mut self.params {
            param_offset = Some(stream.tell());
            params.write(stream);
        }

        stream.align(8);
        self.self_offset = stream.tell();
        stream.write_bytes(b"TLIN");
        let mut string_pool_rel_offset = stream.write_placeholder_u32();
        stream.write_u32(0);
        stream.write_u32(0);
        stream.write_f32(self.duration);
        stream.write_u16(self.actors.len() as u16);
        stream.write_u16(self.get_action_count());
        stream.write_u16(self.clips.len() as u16);
        stream.write_u16(self.oneshots.len() as u16);
        stream.write_u16(self.subtimelines.len() as u16);
        stream.write_u16(self.cuts.len() as u16);
        stream.write_string_ref(&self.name, false);

        let mut actors_offset_writer = stream.write_placeholder_ptr_if(!self.actors.is_empty(), true);
        let mut clips_offset_writer = stream.write_placeholder_ptr_if(!self.clips.is_empty(), true);
        let mut oneshots_offset_writer = stream.write_placeholder_ptr_if(!self.oneshots.is_empty(), true);
        let mut triggers_offset_writer = stream.write_placeholder_ptr_if(!self.triggers.is_empty(), true);
        let mut subtimelines_offset_writer = stream.write_placeholder_ptr_if(!self.subtimelines.is_empty(), true);
        let mut cuts_offset_writer = stream.write_placeholder_ptr_if(!self.cuts.is_empty(), true);

        if let Some(offset) = param_offset {
            stream.register_pointer(stream.tell());
            stream.write_u64(offset as u64);
        } else {
            stream.write_u64(0);
        }

        if let Some(mut w) = actors_offset_writer.take() {
            w.write_current_offset(stream);
            for actor in &mut self.actors {
                actor.write(stream);
            }
            stream.align(8);
        }

        if let Some(mut w) = clips_offset_writer.take() {
            w.write_current_offset(stream);
            for clip in &mut self.clips {
                clip.write(stream);
            }
            stream.align(8);
        }

        if let Some(mut w) = oneshots_offset_writer.take() {
            w.write_current_offset(stream);
            for oneshot in &mut self.oneshots {
                oneshot.write(stream);
            }
            stream.align(8);
        }

        if let Some(mut w) = subtimelines_offset_writer.take() {
            w.write_current_offset(stream);
            for sub in &mut self.subtimelines {
                sub.write(stream);
            }
            stream.align(8);
        }

        if let Some(mut w) = triggers_offset_writer.take() {
            w.write_current_offset(stream);
            for trigger in &mut self.triggers {
                trigger.write(stream);
            }
            stream.align(8);
        }

        if let Some(mut w) = cuts_offset_writer.take() {
            w.write_current_offset(stream);
            for cut in &mut self.cuts {
                cut.write(stream);
            }
            stream.align(8);
        }

        for clip in &mut self.clips {
            clip.write_extra_data(stream);
            stream.align(8);
        }
        for oneshot in &mut self.oneshots {
            oneshot.write_extra_data(stream);
            stream.align(8);
        }
        for cut in &mut self.cuts {
            cut.write_extra_data(stream);
            stream.align(8);
        }

        stream.align(8);
        string_pool_rel_offset.write_u32(stream, (stream.tell() - self.self_offset) as u32);
    }
}
