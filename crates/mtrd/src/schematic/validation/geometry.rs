use super::{
    OctilinearAxis, Point, PreparedPathPoint, PreparedPointKind, PreparedStroke,
    SchematicRenderError,
};

const EPSILON: f64 = 1.0e-9;

pub(in crate::schematic) fn validate_path(
    line: &str,
    path: usize,
    points: &[PreparedPathPoint<'_>],
    closed: bool,
) -> Result<(), SchematicRenderError> {
    let segment_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    let mut axes = Vec::with_capacity(segment_count);
    for index in 0..segment_count {
        let next = (index + 1) % points.len();
        let dx = points[next].position.x - points[index].position.x;
        let dy = points[next].position.y - points[index].position.y;
        if nearly_zero(dx) && nearly_zero(dy) {
            return Err(SchematicRenderError::CoincidentPoints {
                line: line.to_owned(),
                path,
            });
        }
        axes.push(axis_for_delta(dx, dy).ok_or_else(|| {
            SchematicRenderError::NonOctilinearLeg {
                line: line.to_owned(),
                path,
            }
        })?);
    }

    for (index, point) in points.iter().enumerate() {
        let incoming = if index > 0 {
            Some(axes[index - 1])
        } else if closed {
            axes.last().copied()
        } else {
            None
        };
        let outgoing = if index < axes.len() {
            Some(axes[index])
        } else {
            None
        };
        match point.kind {
            PreparedPointKind::Anchor {
                station,
                permitted_axis,
            } => {
                if let Some(required) = permitted_axis
                    && incoming
                        .into_iter()
                        .chain(outgoing)
                        .any(|axis| axis != required)
                {
                    return Err(SchematicRenderError::InvalidPortAxis {
                        station: station.to_owned(),
                    });
                }
                if let (Some(before), Some(after)) = (incoming, outgoing)
                    && (before != after || anchor_reverses(points, index, closed))
                {
                    return Err(SchematicRenderError::BendAtStation {
                        line: line.to_owned(),
                        path,
                        station: station.to_owned(),
                    });
                }
            }
            PreparedPointKind::Corner { id, radius } => {
                let (Some(_), Some(_)) = (incoming, outgoing) else {
                    return Err(SchematicRenderError::CollinearCorner {
                        corner: id.to_owned(),
                    });
                };
                validate_corner(points, index, closed, id, radius)?;
            }
        }
    }
    for index in 0..segment_count {
        let next = (index + 1) % points.len();
        let length = distance(points[index].position, points[next].position);
        let first_cut = corner_cut(points, index, closed);
        let second_cut = corner_cut(points, next, closed);
        if first_cut + second_cut >= length - EPSILON {
            return Err(SchematicRenderError::CornerRadiiOverlap {
                line: line.to_owned(),
                path,
            });
        }
    }
    Ok(())
}

fn anchor_reverses(points: &[PreparedPathPoint<'_>], index: usize, closed: bool) -> bool {
    if !closed && (index == 0 || index + 1 == points.len()) {
        return false;
    }
    let previous = points[(index + points.len() - 1) % points.len()].position;
    let current = points[index].position;
    let next = points[(index + 1) % points.len()].position;
    let incoming_x = current.x - previous.x;
    let incoming_y = current.y - previous.y;
    let outgoing_x = next.x - current.x;
    let outgoing_y = next.y - current.y;
    incoming_x * outgoing_x + incoming_y * outgoing_y < 0.0
}

fn corner_cut(points: &[PreparedPathPoint<'_>], index: usize, closed: bool) -> f64 {
    let PreparedPointKind::Corner { radius, .. } = points[index].kind else {
        return 0.0;
    };
    if !closed && (index == 0 || index + 1 == points.len()) {
        return 0.0;
    }
    let previous = points[(index + points.len() - 1) % points.len()].position;
    let corner = points[index].position;
    let next = points[(index + 1) % points.len()].position;
    let incoming = unit(corner.x - previous.x, corner.y - previous.y);
    let outgoing = unit(next.x - corner.x, next.y - corner.y);
    let internal_angle = (-incoming.0 * outgoing.0 - incoming.1 * outgoing.1)
        .clamp(-1.0, 1.0)
        .acos();
    radius / (internal_angle / 2.0).tan()
}

pub(in crate::schematic) fn validate_non_overlapping_legs(
    strokes: &[PreparedStroke<'_>],
) -> Result<(), SchematicRenderError> {
    let mut legs = Vec::new();
    for stroke in strokes {
        for path in &stroke.paths {
            let count = if path.closed {
                path.points.len()
            } else {
                path.points.len().saturating_sub(1)
            };
            for index in 0..count {
                legs.push((
                    path.points[index].position,
                    path.points[(index + 1) % path.points.len()].position,
                ));
            }
        }
    }
    for (index, &(first_start, first_end)) in legs.iter().enumerate() {
        for &(second_start, second_end) in &legs[index + 1..] {
            if legs_overlap(first_start, first_end, second_start, second_end) {
                return Err(SchematicRenderError::OverlappingLegs);
            }
        }
    }
    Ok(())
}

fn legs_overlap(
    first_start: Point,
    first_end: Point,
    second_start: Point,
    second_end: Point,
) -> bool {
    let dx = first_end.x - first_start.x;
    let dy = first_end.y - first_start.y;
    let offset_x = second_start.x - first_start.x;
    let offset_y = second_start.y - first_start.y;
    let scale = dx.abs().max(dy.abs()).max(1.0);
    if (dx * offset_y - dy * offset_x).abs() > EPSILON * scale * scale {
        return false;
    }
    let second_dx = second_end.x - second_start.x;
    let second_dy = second_end.y - second_start.y;
    if (dx * second_dy - dy * second_dx).abs() > EPSILON * scale * scale {
        return false;
    }
    let (first_min, first_max, second_min, second_max) = if dx.abs() >= dy.abs() {
        (
            first_start.x.min(first_end.x),
            first_start.x.max(first_end.x),
            second_start.x.min(second_end.x),
            second_start.x.max(second_end.x),
        )
    } else {
        (
            first_start.y.min(first_end.y),
            first_start.y.max(first_end.y),
            second_start.y.min(second_end.y),
            second_start.y.max(second_end.y),
        )
    };
    first_max.min(second_max) - first_min.max(second_min) > EPSILON * scale
}

fn validate_corner(
    points: &[PreparedPathPoint<'_>],
    index: usize,
    closed: bool,
    id: &str,
    radius: f64,
) -> Result<(), SchematicRenderError> {
    if !closed && (index == 0 || index + 1 == points.len()) {
        return Err(SchematicRenderError::CollinearCorner {
            corner: id.to_owned(),
        });
    }
    let previous = points[(index + points.len() - 1) % points.len()].position;
    let current = points[index].position;
    let next = points[(index + 1) % points.len()].position;
    let incoming = unit(current.x - previous.x, current.y - previous.y);
    let outgoing = unit(next.x - current.x, next.y - current.y);
    let cross = incoming.0 * outgoing.1 - incoming.1 * outgoing.0;
    let dot = incoming.0 * outgoing.0 + incoming.1 * outgoing.1;
    if nearly_zero(cross) {
        return Err(if dot > 0.0 {
            SchematicRenderError::CollinearCorner {
                corner: id.to_owned(),
            }
        } else {
            SchematicRenderError::ReversingCorner {
                corner: id.to_owned(),
            }
        });
    }
    let internal_angle = (-incoming.0 * outgoing.0 - incoming.1 * outgoing.1)
        .clamp(-1.0, 1.0)
        .acos();
    let tangent = radius / (internal_angle / 2.0).tan();
    let previous_length = distance(previous, current);
    let next_length = distance(current, next);
    if !tangent.is_finite()
        || tangent >= previous_length - EPSILON
        || tangent >= next_length - EPSILON
    {
        return Err(SchematicRenderError::CornerRadiusTooLarge {
            corner: id.to_owned(),
        });
    }
    Ok(())
}

pub(in crate::schematic) fn corner_tangents(
    previous: Point,
    corner: Point,
    next: Point,
    radius: f64,
) -> (Point, Point, bool) {
    let incoming = unit(corner.x - previous.x, corner.y - previous.y);
    let outgoing = unit(next.x - corner.x, next.y - corner.y);
    let internal_angle = (-incoming.0 * outgoing.0 - incoming.1 * outgoing.1)
        .clamp(-1.0, 1.0)
        .acos();
    let tangent = radius / (internal_angle / 2.0).tan();
    (
        Point {
            x: corner.x - incoming.0 * tangent,
            y: corner.y - incoming.1 * tangent,
        },
        Point {
            x: corner.x + outgoing.0 * tangent,
            y: corner.y + outgoing.1 * tangent,
        },
        incoming.0 * outgoing.1 - incoming.1 * outgoing.0 > 0.0,
    )
}

fn axis_for_delta(dx: f64, dy: f64) -> Option<OctilinearAxis> {
    let scale = dx.abs().max(dy.abs()).max(1.0);
    if dx.abs() <= EPSILON * scale {
        Some(OctilinearAxis::Vertical)
    } else if dy.abs() <= EPSILON * scale {
        Some(OctilinearAxis::Horizontal)
    } else if (dx.abs() - dy.abs()).abs() <= EPSILON * scale {
        if dx * dy > 0.0 {
            Some(OctilinearAxis::FallingDiagonal)
        } else {
            Some(OctilinearAxis::RisingDiagonal)
        }
    } else {
        None
    }
}

pub(in crate::schematic) fn rotate_counter_clockwise(axis: OctilinearAxis) -> OctilinearAxis {
    match axis {
        OctilinearAxis::Horizontal => OctilinearAxis::RisingDiagonal,
        OctilinearAxis::FallingDiagonal => OctilinearAxis::Horizontal,
        OctilinearAxis::Vertical => OctilinearAxis::FallingDiagonal,
        OctilinearAxis::RisingDiagonal => OctilinearAxis::Vertical,
    }
}

pub(in crate::schematic) fn rotate_clockwise(axis: OctilinearAxis) -> OctilinearAxis {
    match axis {
        OctilinearAxis::Horizontal => OctilinearAxis::FallingDiagonal,
        OctilinearAxis::FallingDiagonal => OctilinearAxis::Vertical,
        OctilinearAxis::Vertical => OctilinearAxis::RisingDiagonal,
        OctilinearAxis::RisingDiagonal => OctilinearAxis::Horizontal,
    }
}

pub(in crate::schematic) fn perpendicular(axis: OctilinearAxis) -> OctilinearAxis {
    match axis {
        OctilinearAxis::Horizontal => OctilinearAxis::Vertical,
        OctilinearAxis::FallingDiagonal => OctilinearAxis::RisingDiagonal,
        OctilinearAxis::Vertical => OctilinearAxis::Horizontal,
        OctilinearAxis::RisingDiagonal => OctilinearAxis::FallingDiagonal,
    }
}

pub(in crate::schematic) fn axis_vector(axis: OctilinearAxis) -> (f64, f64) {
    const DIAGONAL: f64 = std::f64::consts::FRAC_1_SQRT_2;
    match axis {
        OctilinearAxis::Horizontal => (1.0, 0.0),
        OctilinearAxis::FallingDiagonal => (DIAGONAL, DIAGONAL),
        OctilinearAxis::Vertical => (0.0, 1.0),
        OctilinearAxis::RisingDiagonal => (DIAGONAL, -DIAGONAL),
    }
}

fn unit(dx: f64, dy: f64) -> (f64, f64) {
    let length = dx.hypot(dy);
    (dx / length, dy / length)
}

fn distance(first: Point, second: Point) -> f64 {
    (second.x - first.x).hypot(second.y - first.y)
}

fn nearly_zero(value: f64) -> bool {
    value.abs() <= EPSILON
}
