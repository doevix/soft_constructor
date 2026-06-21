use serde::{self, Deserialize, Serialize, Deserializer};
use serde_xml_rs;

pub struct Loader;

fn deserialize_mass_id<'de, D>(deserializer: D) -> Result<usize, D::Error>
where D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.strip_prefix('m')
    .ok_or_else(|| serde::de::Error::custom(format!("expected id starting with 'm' got '{s}'")))?
    .parse::<usize>()
    .map_err(serde::de::Error::custom)
}

impl Loader {
    pub fn load(path: &str) -> Result<ModelData, serde_xml_rs::Error> {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| serde_xml_rs::Error::Custom(e.to_string()))?;
        serde_xml_rs::from_str(&xml)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelData {
    #[serde(default)]
    pub comment: String,
    pub container: ContainerData,
    pub environment: EnvironmentData,
    pub collisions: CollisionData,
    pub wave: WaveData,
    pub settings: SettingData,
    pub nodes: NodesData,
    pub links: LinksData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerData {
    #[serde(rename = "@width")]
    pub width: f64,
    #[serde(rename = "@height")]
    pub height: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentData {
    #[serde(rename = "@gravity")]
    pub gravity: f64,
    #[serde(rename = "@friction")]
    pub friction: f64,
    #[serde(rename = "@springyness")]
    pub springyness: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollisionData {
    #[serde(rename = "@surface_friction")]
    pub surface_friction: f64,
    #[serde(rename = "@surface_reflection")]
    pub surface_reflection: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WaveData {
    #[serde(rename = "@amplitude")]
    pub amplitude: f64,
    #[serde(rename = "@phase")]
    pub phase: f64,
    #[serde(rename = "@speed")]
    pub speed: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SettingData {
    #[serde(rename = "@gravitydirection")]
    pub gravitydirection: String,
    #[serde(rename = "@wavedirection")]
    pub wavedirection: String,
    #[serde(rename = "@autoreverse")]
    pub autoreverse: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodesData {
    #[serde(rename = "mass")]
    pub masses: Vec<MassData>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinksData {
    #[serde(rename = "spring", default)]
    pub springs: Vec<SpringData>,
    #[serde(rename = "muscle", default)]
    pub muscles: Vec<MuscleData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MassData {
    #[serde(rename = "@id", deserialize_with = "deserialize_mass_id")]
    pub id: usize,
    #[serde(rename = "@x")]
    pub x: f64,
    #[serde(rename = "@y")]
    pub y: f64,
    #[serde(rename = "@vx")]
    pub vx: f64,
    #[serde(rename = "@vy")]
    pub vy: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SpringData {
    #[serde(rename = "@a", deserialize_with = "deserialize_mass_id")]
    pub a: usize,
    #[serde(rename = "@b", deserialize_with = "deserialize_mass_id")]
    pub b: usize,
    #[serde(rename = "@restlength")]
    pub restlength: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MuscleData {
    #[serde(rename = "@a", deserialize_with = "deserialize_mass_id")]
    pub a: usize,
    #[serde(rename = "@b", deserialize_with = "deserialize_mass_id")]
    pub b: usize,
    #[serde(rename = "@restlength")]
    pub restlength: f64,
    #[serde(rename = "@amplitude")]
    pub amplitude: f64,
    #[serde(rename = "@phase")]
    pub phase: f64,
}

