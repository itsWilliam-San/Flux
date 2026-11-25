use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uom::si::f64::*;

pub type PropertySetBag = HashMap<String, HashMap<String, Value>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SegmentId(pub String);

impl From<String> for SegmentId {
    fn from(s: String) -> Self {
        SegmentId(s)
    }
}

impl From<&str> for SegmentId {
    fn from(s: &str) -> Self {
        SegmentId(s.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrimaryId(pub String);

impl From<String> for PrimaryId {
    fn from(s: String) -> Self {
        PrimaryId(s)
    }
}

impl From<&str> for PrimaryId {
    fn from(s: &str) -> Self {
        PrimaryId(s.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntermediateId(pub String);

impl From<String> for IntermediateId {
    fn from(s: String) -> Self {
        IntermediateId(s)
    }
}

impl From<&str> for IntermediateId {
    fn from(s: &str) -> Self {
        IntermediateId(s.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AnyStationId {
    Primary(PrimaryId),
    Intermediate(IntermediateId),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Connection {
    #[serde(rename = "kind")]
    pub connection_type: String,
    #[serde(rename = "InvertLevel")]
    pub invert_level: Length,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Intermediate {
    #[serde(rename = "ID")]
    pub id: IntermediateId,
    #[serde(rename = "TerrainLevel")]
    pub terrain_level: Length,
    #[serde(rename = "InvertLevel")]
    pub invert_level: Length,
    #[serde(rename = "Depth")]
    pub depth: Length,
}

impl Station for Intermediate {
    fn id(&self) -> AnyStationId {
        AnyStationId::Intermediate(self.id.clone())
    }

    fn terrain_level(&self) -> Length {
        self.terrain_level
    }

    fn invert_level(&self) -> Length {
        self.invert_level
    }

    fn depth(&self) -> Length {
        self.depth
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Primary {
    #[serde(rename = "ID")]
    pub id: PrimaryId,
    #[serde(rename = "TerrainLevel")]
    pub terrain_level: Length,
    #[serde(rename = "InvertLevel")]
    pub invert_level: Length,
    #[serde(rename = "Depth")]
    pub depth: Length,
    #[serde(rename = "UpstreamManholes", default)]
    pub upstream_manholes: Vec<PrimaryId>,
    #[serde(rename = "DownstreamConnection")]
    pub downstream_manhole: Option<(PrimaryId, SegmentId)>,
    #[serde(default)]
    pub psets: PropertySetBag,
}

impl Station for Primary {
    fn id(&self) -> AnyStationId {
        AnyStationId::Primary(self.id.clone())
    }

    fn terrain_level(&self) -> Length {
        self.terrain_level
    }

    fn invert_level(&self) -> Length {
        self.invert_level
    }

    fn depth(&self) -> Length {
        self.depth
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Pipe {
    #[serde(rename = "Material")]
    pub material: String,
    #[serde(rename = "Length")]
    pub length: Length,
    #[serde(rename = "Slope")]
    pub slope: Ratio,
    #[serde(rename = "Diameter")]
    pub diameter: Length,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Segment {
    #[serde(rename = "ID")]
    pub id: SegmentId,
    #[serde(rename = "Connection")]
    pub connection: Option<Connection>,
    #[serde(rename = "Pipe")]
    pub pipe: Pipe,
    #[serde(rename = "Intermediates", default)]
    pub intermediates: Vec<IntermediateId>,
    #[serde(rename = "IntermediateDistance")]
    pub intermediate_distance: Length,
}

pub trait Station {
    fn id(&self) -> AnyStationId;
    fn terrain_level(&self) -> Length;
    fn invert_level(&self) -> Length;
    fn depth(&self) -> Length;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct System {
    #[serde(rename = "Primary")]
    pub primaries: Vec<Primary>,
    #[serde(rename = "Intermediates", default)]
    pub intermediates: Vec<Intermediate>,
    #[serde(rename = "Segments")]
    pub segments: Vec<Segment>,
}
