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
    #[serde(alias = "kind")]
    pub connection_type: String,
    #[serde(alias = "InvertLevel")]
    pub invert_level: Length,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Intermediate {
    #[serde(alias = "ID")]
    pub id: IntermediateId,
    #[serde(alias = "TerrainLevel")]
    pub terrain_level: Length,
    #[serde(alias = "InvertLevel")]
    pub invert_level: Length,
    #[serde(alias = "Depth")]
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
    #[serde(alias = "ID")]
    pub id: PrimaryId,
    #[serde(alias = "TerrainLevel")]
    pub terrain_level: Length,
    #[serde(alias = "InvertLevel")]
    pub invert_level: Length,
    #[serde(alias = "Depth")]
    pub depth: Length,
    #[serde(alias = "UpstreamManholes", default)]
    pub upstream_manholes: Vec<PrimaryId>,
    #[serde(alias = "DownstreamConnection")]
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
    #[serde(alias = "Material")]
    pub material: String,
    #[serde(alias = "Length")]
    pub length: Length,
    #[serde(alias = "Slope")]
    pub slope: Ratio,
    #[serde(alias = "Diameter")]
    pub diameter: Length,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Segment {
    #[serde(alias = "ID")]
    pub id: SegmentId,
    #[serde(alias = "Connection")]
    pub connection: Option<Connection>,
    #[serde(alias = "Pipe")]
    pub pipe: Pipe,
    #[serde(alias = "Intermediates", default)]
    pub intermediates: Vec<IntermediateId>,
    #[serde(alias = "IntermediateDistance")]
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
    #[serde(alias = "Primary", alias = "primaries", default)]
    pub primaries: Vec<Primary>,
    #[serde(alias = "Intermediates", alias = "intermediates", default)]
    pub intermediates: Vec<Intermediate>,
    #[serde(alias = "Segments", alias = "segments", default)]
    pub segments: Vec<Segment>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;
    use uom::si::ratio::ratio;

    #[test]
    fn test_deserialize_contract_json() {
        let json_data = r#"{
            "Intermediates": [
                {
                    "ID": "L-01",
                    "TerrainLevel": 100.00,
                    "InvertLevel": 50.00,
                    "Depth": 50.0
                }
            ],
            "Segments": [
                {
                    "ID": "SEG-ABC",
                    "Connection": {
                        "kind": "DropPipe",
                        "InvertLevel": 10.0
                    },
                    "Pipe": {
                        "Material": "PVC",
                        "Length": 10.0,
                        "Slope": 0.56,
                        "Diameter": 0.05
                    },
                    "Intermediates": ["L-01"],
                    "IntermediateDistance": 20.0
                }
            ],
            "Primary": [
                {
                    "ID": "PV-01",
                    "TerrainLevel": 100.00,
                    "InvertLevel": 50.00,
                    "Depth": 50.0,
                    "UpstreamManholes": ["PV-02"],
                    "DownstreamConnection": ["PV-03", "SEG-ABC"],
                    "psets": {
                        "Topo": {
                            "RN": "11"
                        }
                    }
                }
            ]
        }"#;

        let system: System = serde_json::from_str(json_data).expect("Failed to deserialize System");

        assert_eq!(system.intermediates.len(), 1);
        assert_eq!(system.intermediates[0].id.0, "L-01");
        assert_eq!(system.intermediates[0].terrain_level.get::<meter>(), 100.00);

        assert_eq!(system.segments.len(), 1);
        assert_eq!(system.segments[0].id.0, "SEG-ABC");
        assert_eq!(system.segments[0].pipe.slope.get::<ratio>(), 0.56);

        assert_eq!(system.primaries.len(), 1);
        assert_eq!(system.primaries[0].id.0, "PV-01");
        assert_eq!(
            system.primaries[0].upstream_manholes,
            vec![PrimaryId("PV-02".to_string())]
        );
        assert_eq!(
            system.primaries[0].downstream_manhole,
            Some((
                PrimaryId("PV-03".to_string()),
                SegmentId("SEG-ABC".to_string())
            ))
        );

        // Verify Station trait methods
        assert_eq!(
            system.primaries[0].id(),
            AnyStationId::Primary(PrimaryId("PV-01".to_string()))
        );
        assert_eq!(system.primaries[0].terrain_level().get::<meter>(), 100.00);
        assert_eq!(system.primaries[0].invert_level().get::<meter>(), 50.00);
        assert_eq!(system.primaries[0].depth().get::<meter>(), 50.00);

        // Verify serialization
        let serialized = serde_json::to_string(&system).expect("Failed to serialize System");
        assert!(serialized.contains("primaries"));
        assert!(serialized.contains("PV-01"));
    }
}
