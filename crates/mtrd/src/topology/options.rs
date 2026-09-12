use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};

/// Global visual options used by a topology map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyOptions {
    pub background: TopologyBackgroundOptions,
    pub labels: TopologyLabelOptions,
    pub lines: TopologyLineOptions,
    pub stations: TopologyStationOptions,
}

/// Global display options for station-name labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyLabelOptions {
    pub hidden: bool,
}

/// Global styling shared by all metro lines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyLineOptions {
    pub width: TopologyLength,
}

/// Global styling for common and interchange stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyStationOptions {
    pub common: TopologyCommonStationOptions,
    pub interchange: TopologyInterchangeStationOptions,
}

/// Styling shared by common stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyCommonStationOptions {
    pub fill: TopologyCommonStationFill,
    pub stroke: TopologyCommonStationStroke,
}

/// Fill styling for common stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyCommonStationFill {
    pub diameter: TopologyLength,
    #[serde(alias = "colour")]
    pub color: TopologyStationColor,
}

/// Stroke styling for common stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyCommonStationStroke {
    pub width: TopologyLength,
    pub alignment: TopologyStrokeAlignment,
    #[serde(alias = "colour")]
    pub color: TopologyStationColor,
}

/// Styling shared by interchange stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyInterchangeStationOptions {
    pub fill: TopologyInterchangeStationFill,
    pub stroke: TopologyInterchangeStationStroke,
}

/// Fill styling for interchange stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyInterchangeStationFill {
    pub width: TopologyLength,
    #[serde(alias = "colour")]
    pub color: String,
}

/// Stroke styling for interchange stations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyInterchangeStationStroke {
    pub width: TopologyLength,
    pub alignment: TopologyStrokeAlignment,
    #[serde(alias = "colour")]
    pub color: String,
}

/// How a common-station colour is selected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TopologyStationColor {
    Unified { value: String },
    // The empty struct makes `deny_unknown_fields` apply to this fieldless case.
    FollowLine {},
}

/// Placement of a station stroke relative to its fill boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyStrokeAlignment {
    Inside,
    #[serde(alias = "centre")]
    Center,
    Outside,
}

/// A finite, strictly positive topology rendering length.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TopologyLength(f64);

impl TopologyLength {
    /// Construct a finite, strictly positive length.
    pub fn new(value: f64) -> Result<Self, TopologyValueError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(TopologyValueError::InvalidLength(value));
        }

        Ok(Self(value))
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for TopologyLength {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TopologyLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A scalar invariant violation while constructing topology rendering values.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum TopologyValueError {
    #[error("topology length must be finite and strictly positive, got {0}")]
    InvalidLength(f64),
}

/// An opaque colour or a transparent map background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyBackgroundOptions {
    Color { color: String },
    Transparent,
}

impl Serialize for TopologyBackgroundOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::Color { color } => map.serialize_entry("color", color)?,
            Self::Transparent => map.serialize_entry("transparent", &true)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for TopologyBackgroundOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            #[serde(alias = "colour")]
            color: Option<String>,
            transparent: Option<bool>,
        }

        match Fields::deserialize(deserializer)? {
            Fields {
                color: Some(color),
                transparent: None | Some(false),
            } => Ok(Self::Color { color }),
            Fields {
                color: None,
                transparent: Some(true),
            } => Ok(Self::Transparent),
            Fields {
                color: None,
                transparent: Some(false),
            } => Err(serde::de::Error::custom("transparent must be true")),
            Fields {
                color: Some(_),
                transparent: Some(true),
            } => Err(serde::de::Error::custom(
                "background color conflicts with transparent: true",
            )),
            Fields {
                color: None,
                transparent: None,
            } => Err(serde::de::Error::custom(
                "background must contain color or transparent: true",
            )),
        }
    }
}
