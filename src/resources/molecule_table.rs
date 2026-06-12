use std::fs;

use bevy::{color::Color, ecs::{resource::Resource, world::FromWorld}};
use hex_color::HexColor;
use serde::{Deserialize, Deserializer};

use crate::world::chemistry::Element;

#[derive(Debug, Deserialize)]
pub struct Molecule {
    pub name: String,
    pub formula: String,
    pub id: usize,
    pub components: Vec<Element>,
    #[serde(deserialize_with = "deserialize_color")]
    pub color: Color
}

#[derive(Resource, Debug, Deserialize)]
pub struct MoleculeTable {
    pub molecule: Vec<Molecule>
}

impl MoleculeTable {
    pub fn get_molecule(&self, id: usize) -> Option<&Molecule> {
        self.molecule
            .iter()
            .find(|molecule| molecule.id == id)
    }
}

impl FromWorld for MoleculeTable {
    fn from_world(_: &mut bevy::ecs::world::World) -> Self {
        let contents = fs::read_to_string("assets/config/molecules.toml").expect("Could not read molecule table config file");
        toml::from_str(&contents).expect("Could not deserialize molecule table config")
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