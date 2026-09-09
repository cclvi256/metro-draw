use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A finite point in the schematic coordinate system.
///
/// It is serialized and deserialized as `[x, y]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchematicPoint {
    x: f64,
    y: f64,
}

impl SchematicPoint {
    /// Construct a point whose coordinates are both finite.
    pub fn new(x: f64, y: f64) -> Result<Self, SchematicValueError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(SchematicValueError::NonFinitePoint { x, y });
        }

        Ok(Self { x, y })
    }

    pub const fn x(self) -> f64 {
        self.x
    }

    pub const fn y(self) -> f64 {
        self.y
    }
}

impl Serialize for SchematicPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [self.x, self.y].serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SchematicPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y] = <[f64; 2]>::deserialize(deserializer)?;
        Self::new(x, y).map_err(serde::de::Error::custom)
    }
}

/// A finite, strictly positive schematic length.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SchematicLength(f64);

impl SchematicLength {
    /// Construct a finite, strictly positive length.
    pub fn new(value: f64) -> Result<Self, SchematicValueError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(SchematicValueError::InvalidLength(value));
        }

        Ok(Self(value))
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for SchematicLength {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SchematicLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A scalar invariant violation while constructing schematic values.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum SchematicValueError {
    #[error("schematic point coordinates must be finite, got [{x}, {y}]")]
    NonFinitePoint { x: f64, y: f64 },
    #[error("schematic length must be finite and strictly positive, got {0}")]
    InvalidLength(f64),
}
