use crate::dic::DicWriter;
use crate::flowchart::Flowchart;
use crate::timeline::Timeline;
use crate::util::{read_string, BinaryObject, PlaceholderWriter, ReadStream, SeekContext, WriteStream};
use std::io::{Seek, Write};

/// The root container for an event file, representing either a Flowchart or a Timeline.
/// 
/// 事件文件的根容器，表示一个流程图 (Flowchart) 或时间线 (Timeline)。
#[derive(Debug, Clone, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EventFlow {
    /// The name of the event flow. / 事件流的名称。
    pub name: String,
    /// The flowchart data, if present. / 流程图数据（如果存在）。
    pub flowchart: Option<Flowchart>,
    /// The timeline data, if present. / 时间线数据（如果存在）。
    pub timeline: Option<Timeline>,
}

impl EventFlow {
    /// Creates a new, empty `EventFlow`.
    /// 
    /// 创建一个新的、空的 `EventFlow`。
    pub fn new() -> Self {
        Self {
            name: String::new(),
            flowchart: None,
            timeline: None,
        }
    }

    /// Reads an event flow from binary data.
    /// 
    /// 从二进制数据中读取事件流。
    pub fn read(&mut self, data: &[u8]) {
        let mut stream = ReadStream::new(data);

        let magic = stream.read(8);
        assert_eq!(magic, b"BFEVFL\x00\x00", "Wrong magic");

        let version = stream.read_u16();
        assert_eq!(version, 0x0300, "Wrong version");

        let xa = stream.read_u8();
        let _xb = stream.read_u8();
        assert_eq!(xa, 0, "Wrong xa");

        let bom = stream.read_u16();
        assert_eq!(bom, 0xFEFF, "Wrong byte order mark");

        let _alignment_shifted = stream.read_u8();
        let _xf = stream.read_u8();

        let name_ptr = stream.read_u32();
        self.name = read_string(data, name_ptr as usize);

        let _is_relocated = stream.read_u16();
        let _first_block_offset = stream.read_u16();
        let _relocation_table_offset = stream.read_u32();
        let _file_size = stream.read_u32();

        let num_flowcharts = stream.read_u16();
        let num_timelines = stream.read_u16();
        assert!(num_flowcharts <= 1 && num_timelines <= 1);

        let x24 = stream.read_u32();
        assert_eq!(x24, 0);

        let flowchart_ptr_offset = stream.read_u64();
        let _flowchart_dic_offset = stream.read_u64();
        if num_flowcharts == 1 && flowchart_ptr_offset != 0 {
            let ctx = SeekContext::new(&mut stream, flowchart_ptr_offset as usize);
            let mut fc = Flowchart::new();
            let fc_ptr = ctx.stream.read_u64();
            if fc_ptr != 0 {
                let fc_ctx = SeekContext::new(ctx.stream, fc_ptr as usize);
                fc.read(fc_ctx.stream);
                self.flowchart = Some(fc);
            }
        }

        let timeline_ptr_offset = stream.read_u64();
        let _timeline_dic_offset = stream.read_u64();
        if num_timelines == 1 && timeline_ptr_offset != 0 {
            let ctx = SeekContext::new(&mut stream, timeline_ptr_offset as usize);
            let mut tl = Timeline::new();
            let tl_ptr = ctx.stream.read_u64();
            if tl_ptr != 0 {
                let tl_ctx = SeekContext::new(ctx.stream, tl_ptr as usize);
                tl.read(tl_ctx.stream);
                self.timeline = Some(tl);
            }
        }
    }

    /// Writes the event flow to a binary stream.
    /// Returns `true` if successful.
    /// 
    /// 将事件流写入二进制流。如果成功返回 `true`。
    pub fn write<W: Write + Seek>(&mut self, underlying_stream: &mut W) -> bool {
        let mut stream = WriteStream::new(underlying_stream);

        let has_flowchart = self.flowchart.is_some();
        let has_timeline = self.timeline.is_some();

        if !(has_flowchart || has_timeline) || (has_flowchart && has_timeline) {
            return false;
        }

        stream.write_bytes(b"BFEVFL\x00\x00");
        stream.write_u16(0x0300); // Version
        stream.write_u8(0);
        stream.write_u8(0);
        stream.write_u16(0xFEFF); // BOM
        stream.write_u8(3); // alignment (shifted)
        stream.write_u8(0);
        stream.write_string_ref(&self.name, true);
        stream.write_u16(0); // Is relocated flag
        
        let mut first_block_offset_writer = stream.write_placeholder_u16();
        let mut relocation_table_offset_writer = stream.write_placeholder_u32();
        let mut file_size_writer = stream.write_placeholder_u32();

        stream.write_u16(if has_flowchart { 1 } else { 0 });
        stream.write_u16(if has_timeline { 1 } else { 0 });
        stream.write_u32(0); // Unused?

        let (mut fc_ptr_writer, mut tl_ptr_writer) = self.write_root_structure_metadata(&mut stream);

        if let Some(fc) = &mut self.flowchart {
            let current = stream.tell();
            first_block_offset_writer.write_u16(&mut stream, current as u16);
            if let Some(mut w) = fc_ptr_writer.take() {
                stream.align(8);
                w.write_current_offset(&mut stream);
            }
            fc.write(&mut stream);
        }

        if let Some(tl) = &mut self.timeline {
            tl.write(&mut stream);
            if self.flowchart.is_none() {
                first_block_offset_writer.write_u16(&mut stream, tl.self_offset as u16);
            }
            if let Some(mut w) = tl_ptr_writer.take() {
                w.write_u64(&mut stream, tl.self_offset as u64);
            }
        }

        stream.finalise();

        let file_size = stream.tell();
        file_size_writer.write_u32(&mut stream, file_size as u32);
        
        let rel_table = stream.get_relocation_table_offset();
        relocation_table_offset_writer.write_u32(&mut stream, rel_table as u32);

        true
    }

    fn write_root_structure_metadata<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) -> (Option<PlaceholderWriter>, Option<PlaceholderWriter>) {
        let mut flowchart_array_offset_writer = stream.write_placeholder_ptr_if(self.flowchart.is_some(), true);
        let mut flowchart_dic = DicWriter::new();
        if let Some(fc) = &self.flowchart {
            flowchart_dic.insert(&fc.name);
        }
        flowchart_dic.write_placeholder_offset(stream);

        let mut timeline_array_offset_writer = stream.write_placeholder_ptr_if(self.timeline.is_some(), true);
        let mut timeline_dic = DicWriter::new();
        if let Some(tl) = &self.timeline {
            timeline_dic.insert(&tl.name);
        }
        timeline_dic.write_placeholder_offset(stream);

        let mut fc_ptr_writer = None;
        if self.flowchart.is_some() {
            if let Some(mut w) = flowchart_array_offset_writer.take() {
                w.write_current_offset(stream);
            }
            fc_ptr_writer = Some(stream.write_placeholder_ptr());
        }
        flowchart_dic.write(stream);
        
        let mut tl_ptr_writer = None;
        if self.timeline.is_some() {
            if let Some(mut w) = timeline_array_offset_writer.take() {
                w.write_current_offset(stream);
            }
            tl_ptr_writer = Some(stream.write_placeholder_ptr());
        }
        timeline_dic.write(stream);
        
        (fc_ptr_writer, tl_ptr_writer)
    }
}
