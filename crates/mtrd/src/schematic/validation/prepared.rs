use super::super::{OctilinearAxis, SchematicPoint, SchematicStation, SchematicStrokeAlignment};

#[derive(Debug, Clone, Copy)]
pub(in crate::schematic) struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<SchematicPoint> for Point {
    fn from(value: SchematicPoint) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
        }
    }
}

#[derive(Debug)]
pub(in crate::schematic) struct PreparedSchematic<'a> {
    pub line_width: f64,
    pub symbols: Vec<PreparedSymbol<'a>>,
    pub strokes: Vec<PreparedStroke<'a>>,
}

#[derive(Debug)]
pub(in crate::schematic) struct PreparedSymbol<'a> {
    pub station: &'a SchematicStation,
    pub center: Point,
    pub shape: PreparedShape,
    pub fill: &'a str,
    pub stroke: &'a str,
    pub stroke_width: f64,
    pub stroke_alignment: SchematicStrokeAlignment,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::schematic) enum PreparedShape {
    Circle {
        diameter: f64,
    },
    Capsule {
        axis: OctilinearAxis,
        diameter: f64,
        length: f64,
    },
}

#[derive(Debug)]
pub(in crate::schematic) struct PreparedStroke<'a> {
    pub id: &'a str,
    pub color: &'a str,
    pub paths: Vec<PreparedPath<'a>>,
}

#[derive(Debug)]
pub(in crate::schematic) struct PreparedPath<'a> {
    pub points: Vec<PreparedPathPoint<'a>>,
    pub closed: bool,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::schematic) struct PreparedPathPoint<'a> {
    pub position: Point,
    pub kind: PreparedPointKind<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::schematic) enum PreparedPointKind<'a> {
    Anchor {
        station: &'a str,
        permitted_axis: Option<OctilinearAxis>,
    },
    Corner {
        id: &'a str,
        radius: f64,
    },
}
