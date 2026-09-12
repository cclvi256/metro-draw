use serde::{Deserialize, Serialize};

use crate::LocalizedNames;

use super::{SchematicLength, SchematicPoint};

/// A station and its requested schematic symbol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SchematicStation {
    pub id: String,
    pub position: SchematicPoint,
    pub names: LocalizedNames,
    pub symbol: SchematicStationSymbol,
}

/// The station symbol requested by the semantic schematic manifest.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    deny_unknown_fields
)]
pub enum SchematicStationSymbol {
    // Empty struct variants make `deny_unknown_fields` apply to fieldless cases.
    Circle {},
    Capsule {
        axis: OctilinearAxis,
        anchor_count: u8,
        anchor_interval: SchematicLength,
    },
}
/// A route's connection to a common or interchange station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "interchange",
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    deny_unknown_fields
)]
pub enum SchematicStationPort {
    SingleLine,
    Interchange(SchematicInterchangePort),
}

/// A route's geometric relationship to an interchange capsule's major axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    deny_unknown_fields
)]
pub enum SchematicInterchangePort {
    // Empty struct variants make `deny_unknown_fields` apply to fieldless cases.
    MajorAxis {},
    RisingOblique {},
    FallingOblique {},
    SinglePerpendicular {},
    PerpendicularAnchor { index: u8 },
}

/// An undirected octilinear axis in SVG coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OctilinearAxis {
    Horizontal,
    FallingDiagonal,
    Vertical,
    RisingDiagonal,
}
