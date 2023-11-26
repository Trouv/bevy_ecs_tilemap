use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Component)]
struct CanvasCamera;
#[derive(Component)]
struct Canvas;
const WIDTH: f32 = 256.;
const HEIGHT: f32 = 256.;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    #[cfg(all(not(feature = "atlas"), feature = "render"))] array_texture_loader: Res<
        ArrayTextureLoader,
    >,
) {
    commands.spawn((Camera2dBundle::default(), RenderLayers::layer(1)));

    let size = Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    let canvas = images.add(image);

    commands.spawn((
        SpriteBundle {
            texture: canvas.clone(),
            transform: Transform::from_scale(Vec3::splat(2.)),
            ..default()
        },
        Canvas,
        RenderLayers::layer(1),
    ));

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };

    // Create a tilemap entity a little early.
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    // Eventually, we will insert the `TilemapBundle` bundle on the entity, which
    // will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

    // To begin creating the map we will need a `TileStorage` component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
    // per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(map_size);

    // Spawn the elements of the tilemap.
    // Alternatively, you can use helpers::filling::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });

    // Add atlas to array texture loader so it's preprocessed before we need to use it.
    // Only used when the atlas feature is off and we are using array textures.
    #[cfg(all(not(feature = "atlas"), feature = "render"))]
    {
        array_texture_loader.add(TilemapArrayTexture {
            texture: TilemapTexture::Single(asset_server.load("tiles.png")),
            tile_size,
            ..Default::default()
        });
    }
}

fn swap_texture_or_hide(
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TilemapTexture, &mut Visibility)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let texture_a = TilemapTexture::Single(asset_server.load("tiles.png"));
        let texture_b = TilemapTexture::Single(asset_server.load("tiles2.png"));
        for (mut tilemap_tex, _) in &mut query {
            if *tilemap_tex == texture_a {
                *tilemap_tex = texture_b.clone();
            } else {
                *tilemap_tex = texture_a.clone();
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::H) {
        for (_, mut visibility) in &mut query {
            *visibility = match *visibility {
                Visibility::Inherited | Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden => Visibility::Visible,
            };
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            primary_window: Some(Window {
                title: String::from(
                    "Basic Example - Press Space to change Texture and H to show/hide tilemap.",
                ),
                ..Default::default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, swap_texture_or_hide)
        .add_systems(Update, add_camera)
        .run();
}

fn add_camera(
    camera: Query<&CanvasCamera>,
    mut commands: Commands,
    mut frame_count: Local<usize>,
    canvas_query: Query<&Handle<Image>, With<Canvas>>,
) {
    if *frame_count > 10 {
        if camera.is_empty() {
            let canvas = canvas_query.single();
            println!("spawning rendertarget camera");
            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: -1,
                        target: RenderTarget::Image(canvas.clone()),
                        ..default()
                    },
                    ..default()
                },
                CanvasCamera,
            ));
        }
    } else {
        *frame_count += 1;
    }
}
