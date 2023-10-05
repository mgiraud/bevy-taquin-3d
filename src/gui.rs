use bevy::{prelude::*, animation::RepeatAnimation};

use std::f32::consts::PI;

use crate::taquin::{TaquinShuffled, Taquin};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_gui)
            .add_systems(Update, taquin_shuffled_listener);
    }
}

#[derive(Component)]
pub struct MainMessage {
    shuffle_anim: Handle<AnimationClip>
}

fn taquin_shuffled_listener(
    taquin: Res<Taquin>,
    mut events: EventReader<TaquinShuffled>,
    mut main_message_query: Query<(&mut AnimationPlayer, &MainMessage)>
) {
    for _ in events.read() {
        if let Ok((mut player, message)) = main_message_query.get_single_mut() {
            player.play(message.shuffle_anim.clone_weak()).set_repeat(RepeatAnimation::Count(3)).replay();
        }
        if !taquin.is_solvable() {
            println!("PAS SOLVABLE");
        } else {
            println!("SOLVABLE");
        }
    }
}

fn setup_gui(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
) {

    let main_message_shuffle = Name::new("shuffle");
    let mut animation = AnimationClip::default();
    
    animation.add_curve_to_path(
        EntityPath {
            parts: vec![main_message_shuffle.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 0.05, 0.1, 0.15, 0.2],
            keyframes: Keyframes::Rotation(vec![
                Quat::IDENTITY,
                Quat::from_axis_angle(Vec3::Z, PI / 2.),
                Quat::from_axis_angle(Vec3::Z, PI / 2. * 2.),
                Quat::from_axis_angle(Vec3::Z, PI / 2. * 3.),
                Quat::IDENTITY,
            ]),
        },
    );
    let shuffle_anim = animations.add(animation);

    commands
    .spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        // left vertical fill (border)
        parent
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(20.),
                    border: UiRect::top(Val::Px(2.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.65, 0.65, 0.65, 0.2).into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    // Create a TextBundle that has a Text with a single section.
                    TextBundle::from_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        "Taquin",
                        TextStyle {
                            // This font is loaded and will be used instead of the default font.
                            font_size: 100.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ) // Set the alignment of the Text
                    .with_text_alignment(TextAlignment::Center)
                    // Set the style of the TextBundle itself.
                    .with_style(Style {
                        position_type: PositionType::Relative,
                        ..default()
                    })
                , MainMessage { shuffle_anim }, main_message_shuffle));
            });
        });
}
