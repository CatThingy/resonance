use bevy::prelude::*;

use crate::{director::Rounds, GameState};

#[derive(Component)]
pub struct Root;

#[derive(Component)]
pub struct MenuButton;

pub struct Plugin;

impl Plugin {
    fn init(mut cmd: Commands, assets: Res<AssetServer>, mut rounds: ResMut<Rounds>) {
        cmd.spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(Root)
        .with_children(|root| {
            root.spawn(TextBundle {
                text: Text::from_section(
                    format!("Rounds survived: {}", rounds.0 - 1),
                    TextStyle {
                        font: assets.load("FiraSans-Light.ttf"),
                        font_size: 72.0,
                        color: Color::hex("bc53ff").unwrap(),
                    },
                ),
                style: Style {
                    margin: UiRect {
                        top: Val::Px(20.0),
                        bottom: Val::Px(20.0),
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                    },

                    ..default()
                },
                ..default()
            });
            root.spawn((
                ButtonBundle {
                    image: UiImage {
                        texture: assets.load("menu.png"),
                        ..default()
                    },
                    style: Style {
                        size: Size {
                            width: Val::Px(120.0),
                            height: Val::Px(120.0),
                        },
                        ..default()
                    },
                    ..default()
                },
                MenuButton,
            ));
        });
        rounds.0 = 0;
    }

    fn cleanup(mut cmd: Commands, q_root: Query<Entity, With<Root>>) {
        for entity in &q_root {
            cmd.entity(entity).despawn_recursive();
        }
    }

    fn handle_menu_click(
        mut next_state: ResMut<NextState<GameState>>,
        q_button: Query<&Interaction, (Changed<Interaction>, With<MenuButton>)>,
        mouse: Res<Input<MouseButton>>,
    ) {
        if mouse.just_released(MouseButton::Left) {
            for button in &q_button {
                if button == &Interaction::Hovered {
                    next_state.set(GameState::MainMenu);
                }
            }
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::init.in_schedule(OnEnter(GameState::GameOver)))
            .add_system(Self::cleanup.in_schedule(OnExit(GameState::GameOver)))
            .add_system(Self::handle_menu_click.run_if(in_state(GameState::GameOver)));
    }
}
