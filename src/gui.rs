use bevy::{prelude::*, animation::RepeatAnimation};

use std::f32::consts::PI;

use crate::taquin::{TaquinShuffled, TaquinSolved, TileMoved};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_gui)
            .add_systems(Update, (
                taquin_shuffled_listener.run_if(on_event::<TaquinShuffled>()),
                on_taquin_solved_reset_gui.run_if(on_event::<TaquinSolved>()),
                on_tile_moved_increase_counter.run_if(on_event::<TileMoved>()),
            ));
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

#[derive(Component, Default)]
pub struct MoveCounter(usize);

impl MoveCounter {
    pub fn incr(&mut self) {
        self.0 += 1;
    }

    pub fn reset(&mut self){
        self.0 = 0;
    }
}

impl From<&mut MoveCounter> for String {
    fn from(item: &mut MoveCounter) -> Self {
        item.0.to_string()
    }
}

fn taquin_shuffled_listener(
    mut main_message_query: Query<(&mut AnimationPlayer, &MainMessage)>,
    mut shuffle_key_query: Query<&mut Style, With<ShuffleKey>>,
    mut move_counter_query: Query<(&mut Text, &mut MoveCounter)>
) {
    let Ok((mut player, message)) = main_message_query.get_single_mut() else {
        return;
    };

    if player.completions() == 0 || player.is_finished() {
        player.play(message.shuffle_anim.clone_weak()).set_repeat(RepeatAnimation::Count(3)).replay();
    }

    if let Ok(mut style) = shuffle_key_query.get_single_mut() {
        style.display = Display::None;
    };

    if let Ok((mut text, mut counter)) = move_counter_query.get_single_mut() {
        counter.reset();
        text.sections[0].value = counter.as_mut().into();
    };
}

fn on_taquin_solved_reset_gui(
    mut shuffle_key_query: Query<&mut Style, With<ShuffleKey>>,
    mut move_counter_query: Query<(&mut Text, &mut MoveCounter)>
) {
    let Ok(mut style) = shuffle_key_query.get_single_mut() else {
        return;
    };
    style.display = Display::DEFAULT;
    if let Ok((mut text, mut counter)) = move_counter_query.get_single_mut() {
        counter.reset();
        text.sections[0].value = counter.as_mut().into();
    };
}

fn on_tile_moved_increase_counter(
    mut move_counter_query: Query<(&mut Text, &mut MoveCounter)>
) {
    if let Ok((mut text, mut counter)) = move_counter_query.get_single_mut() {
        counter.incr();
        text.sections[0].value = counter.as_mut().into();
    };
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
            align_items: AlignItems::Stretch,
            flex_direction: FlexDirection::Column,
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
                    padding: UiRect::new(Val::Px(10.), Val::ZERO, Val::Px(10.), Val::ZERO),
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "0",
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
                ,
                MoveCounter::default()));
            });

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
                    }), 
                    MainMessage { shuffle_anim }, main_message_name, animation_player
            ));

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
                    ShuffleKey
                ));
            });
        });
}
