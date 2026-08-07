//! # Revfl
//! 
//! A Rust library for parsing and writing Nintendo BFEVFL (Event Flow) and BFEVTM (Event Timeline) files.
//! 
//! 一个用于解析和写入任天堂 BFEVFL (事件流) 和 BFEVTM (事件时间线) 文件的 Rust 库。
//!
//! ## Example / 示例
//! ```no_run
//! use revfl::evfl::EventFlow;
//! use std::fs;
//! 
//! let data = fs::read("example.bfevfl").unwrap();
//! let mut evfl = EventFlow::new();
//! evfl.read(&data);
//! println!("EventFlow name: {}", evfl.name);
//! ```

pub mod enums;
pub mod util;
pub mod common;
pub mod dic;
pub mod container;
pub mod event;
pub mod actor;
pub mod entry_point;
pub mod flowchart;
pub mod timeline;
pub mod evfl;
