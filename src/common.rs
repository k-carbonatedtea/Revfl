use crate::util::{BinaryObject, ReadStream, WriteStream};
use std::io::{Write, Seek};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ActorIdentifier {
    pub name: String,
    pub sub_name: String,
}

impl ActorIdentifier {
    pub fn new(name: &str, sub_name: &str) -> Self {
        Self {
            name: name.to_string(),
            sub_name: sub_name.to_string(),
        }
    }
}

impl Default for ActorIdentifier {
    fn default() -> Self {
        Self {
            name: String::new(),
            sub_name: String::new(),
        }
    }
}

impl BinaryObject for ActorIdentifier {
    fn read(&mut self, stream: &mut ReadStream) {
        self.name = stream.read_string_ref();
        self.sub_name = stream.read_string_ref();
    }

    fn write<W: Write + Seek>(&mut self, stream: &mut WriteStream<W>) {
        stream.write_string_ref(&self.name, false);
        stream.write_string_ref(&self.sub_name, false);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Argument(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StringHolder(pub String);
