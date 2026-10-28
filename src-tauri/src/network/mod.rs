use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uom::si::f64::*;

pub type PropertySetBag = HashMap<String, HashMap<String, Value>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SegmentId(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PrimaryId(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct IntermediateId(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AnyStationId {
    Primary(PrimaryId),
    Intermediate(IntermediateId),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Connection {
    pub connection_type: String,
    pub invert_level: Length,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Intermediate {
    pub id: PrimaryId,
    pub terrain_level: Length,
    pub invert_level: Length,
    pub depth: Length,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Primary {
    pub id: PrimaryId,
    pub terrain_level: Length,
    pub invert_level: Length,
    pub depth: Length,
    pub upstream_manholes: Vec<PrimaryId>,
    pub downstream_manhole: Option<(PrimaryId, SegmentId)>,
    pub psets: PropertySetBag,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Segment {
    pub id: SegmentId,
    pub connection: Option<Connection>,
    pub pipe: Pipe,
    pub intermediates: Vec<IntermediateId>,
    pub intermediate_distance: Length,
}

pub trait Station {
    fn id(&self) -> AnyStationId;
    fn terrain_level(&self) -> Length;
    fn invert_level(&self) -> Length;
    fn depth(&self) -> Length;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pipe {
    pub material: String,
    pub length: Length,
    pub slope: Ratio,
    pub diameter: Length,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct System {
    pub primaries: Vec<Primary>,
    pub intermediates: Vec<Intermediate>,
    pub segments: Vec<Segment>,
}
