use super::super::{
    SchematicRenderError, SchematicStrokeAlignment,
    validation::{Point, PreparedSchematic, PreparedShape, axis_vector},
};

const PADDING: f64 = 24.0;

#[derive(Debug, Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    pub(super) fn from_schematic(
        schematic: &PreparedSchematic<'_>,
    ) -> Result<Self, SchematicRenderError> {
        let mut bounds: Option<Self> = None;
        for stroke in &schematic.strokes {
            for path in &stroke.paths {
                for point in &path.points {
                    extend_point(&mut bounds, point.position, schematic.line_width / 2.0);
                }
            }
        }
        for symbol in &schematic.symbols {
            let stroke_outset = match symbol.stroke_alignment {
                SchematicStrokeAlignment::Inside => 0.0,
                SchematicStrokeAlignment::Center => symbol.stroke_width / 2.0,
                SchematicStrokeAlignment::Outside => symbol.stroke_width,
            };
            let (half_x, half_y) = match symbol.shape {
                PreparedShape::Circle { diameter } => {
                    let radius = diameter / 2.0 + stroke_outset;
                    (radius, radius)
                }
                PreparedShape::Capsule {
                    axis,
                    diameter,
                    length,
                } => {
                    let (unit_x, unit_y) = axis_vector(axis);
                    let half_length = length / 2.0 + stroke_outset;
                    let half_diameter = diameter / 2.0 + stroke_outset;
                    (
                        unit_x.abs() * half_length + unit_y.abs() * half_diameter,
                        unit_y.abs() * half_length + unit_x.abs() * half_diameter,
                    )
                }
            };
            extend_rect(
                &mut bounds,
                symbol.center.x - half_x,
                symbol.center.y - half_y,
                symbol.center.x + half_x,
                symbol.center.y + half_y,
            );
        }
        let Some(bounds) = bounds else {
            return Ok(Self {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 256.0,
                max_y: 96.0,
            });
        };
        let bounds = Self {
            min_x: bounds.min_x - PADDING,
            min_y: bounds.min_y - PADDING,
            max_x: bounds.max_x + PADDING,
            max_y: bounds.max_y + PADDING,
        };
        if [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter()
            .all(f64::is_finite)
            && bounds.width().is_finite()
            && bounds.height().is_finite()
        {
            Ok(bounds)
        } else {
            Err(SchematicRenderError::CoordinateRange)
        }
    }

    pub(super) fn width(self) -> f64 {
        self.max_x - self.min_x
    }

    pub(super) fn height(self) -> f64 {
        self.max_y - self.min_y
    }
}

fn extend_point(bounds: &mut Option<Bounds>, point: Point, outset: f64) {
    extend_rect(
        bounds,
        point.x - outset,
        point.y - outset,
        point.x + outset,
        point.y + outset,
    );
}

fn extend_rect(bounds: &mut Option<Bounds>, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
    match bounds {
        Some(bounds) => {
            bounds.min_x = bounds.min_x.min(min_x);
            bounds.min_y = bounds.min_y.min(min_y);
            bounds.max_x = bounds.max_x.max(max_x);
            bounds.max_y = bounds.max_y.max(max_y);
        }
        None => {
            *bounds = Some(Bounds {
                min_x,
                min_y,
                max_x,
                max_y,
            });
        }
    }
}
