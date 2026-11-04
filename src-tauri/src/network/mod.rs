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

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;
    use uom::si::ratio::ratio;

    #[test]
    fn test_deserialize_contract_json() {
        let json_data = r#"'
{
    "Intermediates": [
        {
            "ID": "1-1E1",
            "Terrainlevel": 851.439,
            "InvertLevel": 850.335,
            "Depth": 1.104
        },
        {
            "ID": "1-1E2",
            "Terrainlevel": 850.984,
            "InvertLevel": 849.934,
            "Depth": 1.050
        },
        {
            "ID": "1-1E3",
            "Terrainlevel": 850.651,
            "InvertLevel": 849.533,
            "Depth": 1.118
        },
        {
            "ID": "1-1E4",
            "Terrainlevel": 850.246,
            "InvertLevel": 849.132,
            "Depth": 1.114
        },
        {
            "ID": "1-2E1",
            "Terrainlevel": 849.789,
            "InvertLevel": 848.396,
            "Depth": 1.393
        },
        {
            "ID": "1-2E2",
            "Terrainlevel": 848.955,
            "InvertLevel": 847.905,
            "Depth": 1.050
        }
    ],
    "Segments": [
        {
            "ID": "1-1",
            "Connection": {
                "kind": "DG",
                "InvertLevel": 849.112
            },
            "Pipe": {
                "Material": "PVC",
                "Length": 81,
                "Slope": 0.02005,
                "Diameter": 150
            },
            "Intermediates": ["1-1E1", "1-1E2", "1-1E3", "1-1E4"],
            "IntermediateDistance": 20
        },
        {
            "ID": "1-2",
            "Pipe": {
                "Material": "PVC",
                "Length": 60,
                "Slope": 0.02458,
                "Diameter": 150
            },
            "Intermediates": ["1-2E1", "1-2E2"],
            "IntermediateDistance": 20
        }
    ],
    "Primary": [
        {
            "ID": "SB1-001",
            "TerrainLevel": 851.862,
            "InvertLevel": 850.736,
            "Depth": 1.126,
            "UpstreamManholes": [],
            "DownstreamConnection": ["SB1-002", "1-1"],
            "psets": {
                "Topo": {
                    "RN": ""
                }
            }
        },
        {
            "ID": "SB1-002",
            "TerrainLevel": 850.238,
            "InvertLevel": 848.888,
            "Depth": 1.350,
            "UpstreamManholes": ["SB1-001"],
            "DownstreamConnection": ["SB1-003", "1-2"],
            "psets": {
                "Topo": {
                    "RN": ""
                }
            }
        },
        {
            "ID": "SB1-003",
            "TerrainLevel": 848.687,
            "InvertLevel": 847.413,
            "Depth": 1.274,
            "UpstreamManholes": ["SB1-002"],
            "psets": {
                "Topo": {
                    "RN": ""
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
