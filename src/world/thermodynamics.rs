use bevy::prelude::*;

use crate::world::celestial::IntensityMap;

#[derive(Component, Clone, Default)]
pub struct HeatMap ( pub IntensityMap );