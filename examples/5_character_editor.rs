//! Challenge 5: a character editor.
//!
//! - A 3D character (simple shapes) on the left, the editor UI on the right.
//! - Buttons pick hat, head and colour; the chosen one is highlighted.
//! - The colour list has more options than fit, so it scrolls (mouse wheel, or move focus
//!   into it with the keyboard / D-pad and it follows).
//! - A text field at the top names the character.
//! - Every change in the UI changes the 3D scene: the controls are bound to the `Look`
//!   resource, and one system rebuilds the character when `Look` changes.
//!
//! Run with: `cargo run --example 5_character_editor`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const HATS: [&str; 4] = ["None", "Cap", "Top hat", "Crown"];
const HEADS: [&str; 2] = ["Round", "Square"];
const COLOURS: [(&str, [f32; 3]); 12] = [
    ("Crimson", [0.80, 0.15, 0.20]),
    ("Tangerine", [0.95, 0.55, 0.15]),
    ("Sunflower", [0.95, 0.80, 0.20]),
    ("Lime", [0.55, 0.85, 0.25]),
    ("Forest", [0.15, 0.50, 0.25]),
    ("Teal", [0.15, 0.65, 0.65]),
    ("Sky", [0.40, 0.70, 0.95]),
    ("Royal", [0.25, 0.30, 0.85]),
    ("Violet", [0.55, 0.30, 0.80]),
    ("Rose", [0.90, 0.45, 0.65]),
    ("Slate", [0.40, 0.45, 0.50]),
    ("Snow", [0.92, 0.92, 0.95]),
];

#[derive(Resource, Default)]
struct Look {
    name: String,
    hat: usize,
    head: usize,
    colour: usize,
}

/// Every mesh that makes up the character, so a rebuild can clear them.
#[derive(Component)]
struct Part;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .insert_resource(Look { colour: 6, ..default() })
        .bind_resource::<Look>()
        .add_systems(Startup, (stage, editor.spawn()))
        .add_systems(Update, rebuild_character)
        .run();
}

fn editor() -> impl SceneList {
    let colours: Vec<&str> = COLOURS.iter().map(|c| c.0).collect();
    bsn_list! {
        @scenes::anchored(ScreenAnchor::Right, 0.0)
        Children [
            @scenes::card()
            Node { height: percent(100), width: px(380), border_radius: {BorderRadius::ZERO} }
            Children [
                @scenes::text_input("Name your hero") AutoFocus
                @scenes::bind_input(|l: &mut Look, text| l.name = text.to_string())
                --
                @scenes::header("") @scenes::bind_text(|l: &Look| {
                    if l.name.is_empty() { "Nameless".into() } else { l.name.clone() }
                })
                --
                @scenes::caption("Hat")
                --
                @scenes::choice(&HATS, 0) @scenes::bind_choice(|l: &Look| l.hat, |l, i| l.hat = i)
                --
                @scenes::caption("Head")
                --
                @scenes::choice(&HEADS, 0) @scenes::bind_choice(|l: &Look| l.head, |l, i| l.head = i)
                --
                @scenes::caption("Colour")
                --
                // Bounded on purpose: more colours than fit, so the list scrolls.
                @scenes::scrollable_list_bounded(4.0, 200.0)
                Children [
                    @scenes::choice(&colours, 0)
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::NoWrap,
                        row_gap: px(4),
                    }
                    @scenes::bind_choice(|l: &Look| l.colour, |l, i| l.colour = i)
                ]
            ]
        ]
    }
}

/// Throw the old character away and build the new one whenever `Look` changes.
fn rebuild_character(
    look: Res<Look>,
    parts: Query<Entity, With<Part>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !look.is_changed() {
        return;
    }
    for part in &parts {
        commands.entity(part).despawn();
    }
    let [r, g, b] = COLOURS.get(look.colour).map_or([0.5; 3], |c| c.1);
    let body = materials.add(Color::srgb(r, g, b));
    let skin = materials.add(Color::srgb(0.95, 0.80, 0.65));
    let hat = materials.add(Color::srgb(0.15, 0.12, 0.10));
    let gold = materials.add(Color::srgb(0.95, 0.75, 0.20));
    let mut spawn = |mesh: Mesh, material: Handle<StandardMaterial>, at: Vec3| {
        commands.spawn((
            Part,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(at),
        ));
    };
    spawn(Capsule3d::new(0.45, 0.9).into(), body, Vec3::new(0.0, 0.9, 0.0));
    let head = Vec3::new(0.0, 2.0, 0.0);
    if look.head == 1 {
        spawn(Cuboid::from_length(0.62).into(), skin, head);
    } else {
        spawn(Sphere::new(0.36).into(), skin, head);
    }
    match look.hat {
        1 => spawn(Cylinder::new(0.38, 0.12).into(), hat, head + Vec3::Y * 0.35),
        2 => spawn(Cylinder::new(0.28, 0.6).into(), hat, head + Vec3::Y * 0.6),
        3 => spawn(Cylinder::new(0.3, 0.22).into(), gold, head + Vec3::Y * 0.42),
        _ => {}
    }
}

fn stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Looking a little to the right of the character puts it in the left half of the
    // window, clear of the editor panel.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.6, 2.0, 5.0).looking_at(Vec3::new(1.6, 1.2, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.2, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb(0.30, 0.30, 0.36))),
    ));
}
