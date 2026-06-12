use bevy::ecs::component::Component;
use serde::Deserialize;

use crate::resources::{molecule_table::MoleculeTable};

#[derive(Component, Clone, Copy, Debug, Deserialize)]
pub struct Element {
    pub element: usize,
    pub ratio: f32,
}

#[derive(Component, Clone, Deserialize)]
pub struct ChemicalComposition {
    pub components: Vec<Element>
}

impl ChemicalComposition {
    pub fn get_max_fraction_of_molecule(&self, id: usize, molecule_table: &MoleculeTable) -> Option<f32> {
        let Some(molecule) = molecule_table.get_molecule(id) else { return None; };

        let mut lowest_element_presence: f32 = 1.0;
        for element in &molecule.components {
            let demand: f32 = element.ratio;
            let Some(available) = self.components
                .iter().find(|e| e.element == element.element)
                else { return None; };

            lowest_element_presence = lowest_element_presence.min(available.ratio / demand);
        }
        Some(lowest_element_presence)
    }
}