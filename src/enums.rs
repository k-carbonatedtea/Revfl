#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataType0 {
    Int = 0,
    Float = 1,
    String = 2,
    WString = 3,
    Stream = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum QueryValueType {
    Bool = 0,
    Int = 1,
    Float = 2,
    String = 3,
    Const = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DataType1 {
    Int = 0,
    Bool = 1,
    Float = 2,
    String = 3,
    WString = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum EventType {
    Action = 0,
    Switch = 1,
    Fork = 2,
    Join = 3,
    SubFlow = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ContainerDataType {
    Argument = 0,
    Container = 1,
    Int = 2,
    Bool = 3,
    Float = 4,
    String = 5,
    WString = 6,
    IntArray = 7,
    BoolArray = 8,
    FloatArray = 9,
    StringArray = 10,
    WStringArray = 11,
    ActorIdentifier = 12,
}

impl ContainerDataType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Argument),
            1 => Some(Self::Container),
            2 => Some(Self::Int),
            3 => Some(Self::Bool),
            4 => Some(Self::Float),
            5 => Some(Self::String),
            6 => Some(Self::WString),
            7 => Some(Self::IntArray),
            8 => Some(Self::BoolArray),
            9 => Some(Self::FloatArray),
            10 => Some(Self::StringArray),
            11 => Some(Self::WStringArray),
            12 => Some(Self::ActorIdentifier),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TriggerType {
    Normal = 0, // kNormal / kFlowchart
    Enter = 1,  // kEnter / kClipEnter
    Leave = 2,  // kLeave / kClipLeave
    Oneshot = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TimelineState {
    NotStarted = 0,
    Playing = 1,
    Stop = 2,
    Pause = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum State {
    Invalid = 0,
    Free = 1,
    NotInvoked = 2,
    Invoked = 3,
    Done = 4,
    Waiting = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum BuildResultType {
    Success = 0,
    InvalidOperation = 1,
    ResFlowchartNotFound = 2,
    EntryPointNotFound = 3,
}
