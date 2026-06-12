use std::fs;

use bevy::{color::Color, ecs::{resource::Resource, world::FromWorld}, log::warn};
use hex_color::HexColor;
use serde::{Deserialize, Deserializer};

use crate::world::chemistry::Element;

#[derive(Debug, Deserialize)]
pub struct DensityMap{
    pub gas: f32,
    pub liquid: f32,
    pub solid: f32
}

#[derive(Debug, Deserialize)]
pub struct TransitionTempMap{
    pub boiling: f32,
    pub freezing: f32
}

#[derive(Debug, Deserialize)]
pub struct Chemical {
    pub name: String,
    pub symbol: String,
    pub id: usize,
    pub density: DensityMap,
    pub transition_temp: TransitionTempMap,
    pub heat_capacity: f32,
    #[serde(deserialize_with = "deserialize_color")]
    pub color: Color
}

#[derive(Resource, Debug, Deserialize)]
pub struct PeriodicTable {
    pub chemical: Vec<Chemical>
}

impl PeriodicTable {
    pub fn get_color(&self, element: &Element) -> Color {
        let Some(chem_info) = self.chemical
            .iter().find(|chemical| chemical.id == element.element) 
            else {
                return Color::BLACK;
            };
        chem_info.color
    }

    pub fn get_name(&self, element: &Element) -> String {
        let Some(chem_info) = self.chemical
            .iter().find(|chemical| chemical.id == element.element) 
            else {
                return "Unknown element".to_owned();
            };
        chem_info.name.clone()
    }

    pub fn get_element(&self, id: usize) -> Option<&Chemical> {
        self.chemical
            .iter()
            .find(|chemical| chemical.id == id)
    }

    pub fn create_default_element(&self) -> Chemical {
        warn!("Using default element");
        Chemical {
            name: "UNKNOWN ELEMENT".to_string(),
            symbol: "?".to_string(), 
            id: 0, 
            density: DensityMap { gas: 0.0, liquid: 0.0, solid: 0.0 },
            transition_temp: TransitionTempMap { boiling: 0.0, freezing: 0.0 },
            heat_capacity: 0.0,
            color: Color::BLACK
        }
    }
}

impl FromWorld for PeriodicTable {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        let contents = fs::read_to_string("assets/config/chemicals.toml").expect("Could not read periodic table config file");
        toml::from_str(&contents).expect("Could not deserialize periodic table config")
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let col = HexColor::parse(s.as_str());
    match col {
        Ok(hex_color) => Ok(Color::srgb_u8(hex_color.r, hex_color.g, hex_color.b)),
        Err(err) => Err(serde::de::Error::custom(
            "Invalid color - message: ".to_owned() + &err.to_string()
        ))
    }
}