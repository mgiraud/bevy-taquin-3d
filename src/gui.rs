use bevy::{prelude::*, animation::RepeatAnimation};

use std::f32::consts::PI;

use crate::taquin::{TaquinShuffled, Taquin, TaquinSolved};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_gui)
            .add_systems(Update, (taquin_shuffled_listener, taquin_solved_listener));
    }
}

#[derive(Component)]
pub struct MainMessage {
    shuffle_anim: Handle<AnimationClip>
}

#[derive(Component)]
pub struct ShuffleKey;

#[derive(Component)]
pub struct ShuffleKeyTargetSize(f32);


fn taquin_shuffled_listener(
    taquin: Res<Taquin>,
    mut events: EventReader<TaquinShuffled>,
    mut main_message_query: Query<(&mut AnimationPlayer, &MainMessage)>,
    mut shuffle_key_query: Query<&mut Style, With<ShuffleKey>>
) {
    for _ in events.read() {
        let Ok((mut player, message)) = main_message_query.get_single_mut() else {
            return;
        };
        if player.completions() == 0 || player.is_finished() {
            player.play(message.shuffle_anim.clone_weak()).set_repeat(RepeatAnimation::Count(3)).replay();
        }
        if !taquin.is_solvable() {
            println!("PAS SOLVABLE");
            if let Ok(mut style) = shuffle_key_query.get_single_mut() {
                style.display = Display::DEFAULT;
            };
        } else {
            println!("SOLVABLE");
            if let Ok(mut style) = shuffle_key_query.get_single_mut() {
                style.display = Display::None;
            };
        }
    }
}

fn taquin_solved_listener(
    mut events: EventReader<TaquinSolved>,
    mut shuffle_key_query: Query<&mut Style, With<ShuffleKey>>
) {
    for _ in events.read() {
        let Ok(mut style) = shuffle_key_query.get_single_mut() else {
            return;
        };
        style.display = Display::DEFAULT;
    }
}

fn setup_gui(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
    asset_server: Res<AssetServer>
) {

    let main_message_name = Name::new("shuffle");
    let mut animation = AnimationClip::default();
    
    animation.add_curve_to_path(
        EntityPath {
            parts: vec![main_message_name.clone()],
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
    let animation_player = AnimationPlayer::default();

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
                    TextBundle::from_section(
                        "Taquin",
                        TextStyle {
                            font_size: 100.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_text_alignment(TextAlignment::Center)
                    .with_style(Style {
                        position_type: PositionType::Relative,
                        ..default()
                    })
                , MainMessage { shuffle_anim }, main_message_name, animation_player));

                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            margin: UiRect::left(Val::VMin(5.)),
                            ..default()
                        },
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                    UiImage::new(asset_server.load("textures/icons/shuffle_key.png")),
                    ShuffleKey));
            });
        });
}
