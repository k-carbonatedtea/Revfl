# Revfl

[![Crates.io](https://img.shields.io/crates/v/revfl.svg)](https://crates.io/crates/revfl)
[![Documentation](https://docs.rs/revfl/badge.svg)](https://docs.rs/revfl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](README.md)

一个用于解析和写入任天堂 BFEVFL (Binary Format Event Flow，事件流) 和 BFEVTM (Binary Format Event Timeline，事件时间线) 文件的 Rust 库，常用于任天堂游戏（例如《塞尔达传说：旷野之息》、《集合啦！动物森友会》）中的逻辑和过场动画脚本。

这个库允许你通过编程的方式读取、修改和序列化这些事件文件。

## 功能特性

- **完整的 BFEVFL/BFEVTM 支持：** 读取和写入事件流（Event Flow）和事件时间线（Event Timeline）文件。
- **序列化：** 全面支持 `serde`，方便与 JSON 或其他格式相互转换。
- **结构保留：** 对未修改的文件具有完全一致的字节级打包能力。

## 安装说明

将其添加到你的 `Cargo.toml` 文件中：

```toml
[dependencies]
revfl = "0.1.0"
```

## 使用方法

### 读取和写入 BFEVFL 文件

```rust
use revfl::evfl::EventFlow;
use std::fs;
use std::io::Cursor;

fn main() {
    // 1. 读取 BFEVFL 文件
    let data = fs::read("example.bfevfl").expect("读取文件失败");
    
    // 2. 解析事件流
    let mut evfl = EventFlow::new();
    evfl.read(&data);
    
    // 你可以在这里访问 flowcharts（流程图）, timelines（时间线）, actors（角色）, events（事件）和 entry points（入口点）。
    println!("事件流名称: {}", evfl.name);
    
    // 3. 将其写回二进制格式
    let mut output = Cursor::new(Vec::new());
    evfl.write(&mut output);
    
    // 4. 保存为新文件
    fs::write("example_out.bfevfl", output.into_inner()).expect("写入文件失败");
}
```

### 访问流程图数据

```rust
use revfl::evfl::EventFlow;
use std::fs;

fn main() {
    let data = fs::read("example.bfevfl").unwrap();
    let mut evfl = EventFlow::new();
    evfl.read(&data);

    if let Some(flowchart) = &evfl.flowchart {
        println!("流程图名称: {}", flowchart.name);
        println!("角色数量: {}", flowchart.actors.len());
        println!("事件数量: {}", flowchart.events.len());
        
        for entry_point in &flowchart.entry_points {
            println!("入口点: {}", entry_point.name);
        }
    }
}
```

## 核心结构

本库提供的核心结构反映了 BFEVFL 的内部格式：
- `EventFlow`: 根容器，包含 `Flowchart` 或 `Timeline`（或者两者都有）。
- `Flowchart`: 包含逻辑元素，如 `Actor`（角色）、`Event`（事件）和 `EntryPoint`（入口点）。
- `Timeline`: 包含过场动画序列化元素。
- `Actor`: 参与事件流的实体，包含 `Action`（动作）和 `Query`（查询）。
- `Event`: 代表流程中的节点，例如动作 (actions)、分支 (forks)、合并 (joins)、子流程 (subflows) 或切换 (switches)。

## 致谢

本库是 [leoetlino](https://github.com/leoetlino) 编写的原始 Python 版本的 Rust 移植版。
原始的 `evfl` 库地址：[https://github.com/zeldamods/evfl](https://github.com/zeldamods/evfl)。

## 许可证

本项目基于 MIT 许可证开源。
