use bevy::prelude::*;

#[derive(Component)]
pub struct Marker;

#[derive(Resource, Default)]
pub struct Markers {
    pub tl : Vec3,
    pub tr : Vec3,
    pub bl : Vec3,
    pub br : Vec3,
}

impl Markers {
    pub fn is_ready(&self) -> bool {
        self.tl != Vec3::default() && self.tr != Vec3::default() && self.bl != Vec3::default() && self.br != Vec3::default()
    }

    pub fn inner_width(&self) -> f32 {
        self.tr.x - self.tl.x
    }

    pub fn inner_height(&self) -> f32 {
        self.tr.y - self.br.y
    }
}

pub fn setup_markers(
    mut markers: ResMut<Markers>,
    query: Query<(&Name, &GlobalTransform), With<Marker>>
) {
    for (name, global_transform) in query.iter() {
        match name.as_str() {
            "TL" => markers.tl = global_transform.translation(),
            "TR" => markers.tr = global_transform.translation(),
            "BL" => markers.bl = global_transform.translation(),
            "BR" => markers.br = global_transform.translation(),
            _ => (),
        };
    }
}