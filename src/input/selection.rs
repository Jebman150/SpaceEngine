use bevy::prelude::*;

use crate::{ui::sidebar::SelectedBody, world::celestial::Celestial};

pub fn selected_body_change(
    input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedBody>,
    bodies: Query<(&Celestial, Entity), With<Celestial>>
) {
    let Some(current) = selected.0 else {
        info!("Selected first entity");
        selected.0 = Some(bodies.iter().collect::<Vec<(&Celestial, Entity)>>()[0].1);
        return;
    };

    let current_index = 
        bodies.iter().enumerate()
        .find(|(_, (_, e))| e.index_u32() == current.index_u32())
        .unwrap_or((0, bodies.iter().collect::<Vec<(&Celestial, Entity)>>()[0]))
        .0;

     
    if input.just_pressed(KeyCode::ArrowLeft) {
        let next_index = current_index.wrapping_sub(1);
        let Some(prev) = bodies.iter().nth(next_index) else {
            let Some(new_selection) = bodies.iter().last() else { return; };
            selected.0 = Some(new_selection.1);
            return;
        };
        selected.0 = Some(prev.1);
    } else if input.just_pressed(KeyCode::ArrowRight) {
        let next_index = current_index.wrapping_add(1);
        let Some(prev) = bodies.iter().nth(next_index) else {
            let Some(new_selection) = bodies.iter().nth(0) else { return; };
            selected.0 = Some(new_selection.1);
            return;
        };
        selected.0 = Some(prev.1);
    } else {
        return;
    };
}