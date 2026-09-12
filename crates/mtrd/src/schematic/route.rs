use serde::{Deserialize, Serialize};

use crate::LocalizedNames;

use super::{SchematicLength, SchematicPoint, SchematicStationPort};

/// A layout corner shared by its semantic reference and geometric result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SchematicCorner {
    pub id: String,
    pub position: SchematicPoint,
    pub radius: SchematicLength,
}

/// A semantic metro line and its schematic paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SchematicLine {
    pub id: String,
    pub names: LocalizedNames,
    #[serde(alias = "colour")]
    pub color: String,
    pub paths: Vec<SchematicPath>,
}

/// An ordered semantic traversal of stations and layout corners.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SchematicPath {
    pub visits: Vec<SchematicRouteVisit>,
    pub closed: bool,
}

/// A station or corner visited by a semantic schematic path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    deny_unknown_fields
)]
pub enum SchematicRouteVisit {
    Station {
        station_id: String,
        port: SchematicStationPort,
    },
    Corner {
        corner_id: String,
    },
}
