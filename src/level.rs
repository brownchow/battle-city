use crate::{
    common::{
        AnimationIndices, AnimationTimer, AppState, HomeDyingEvent, ENEMIES_PER_LEVEL,
        LEVEL_COLUMNS, LEVEL_ROWS, MAX_LEVELS, SPRITE_TREE_ORDER, TILE_SIZE,
    },
    enemy::{Enemy, LevelSpawnedEnemies},
    player::PlayerNo,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub const LEVEL_TRANSLATION_OFFSET: Vec3 = Vec3::new(
    -LEVEL_COLUMNS as f32 / 2.0 * TILE_SIZE,
    -LEVEL_ROWS as f32 / 2. * TILE_SIZE,
    0.0,
);

// 关卡地图元素
#[derive(Component, Clone, PartialEq, Eq, Debug, Default)]
pub enum LevelItem {
    #[default]
    None,
    // 石墙
    StoneWall,
    // 铁墙
    IronWall,
    // 树木
    Tree,
    // 水
    Water,
    // 家
    Home,
}

// 关卡player1位置标记
#[derive(Component, Default)]
pub struct Player1Marker;
// 关卡player2位置标记
#[derive(Component, Default)]
pub struct Player2Marker;
// 关卡敌人位置标记
#[derive(Component, Default)]
pub struct EnemiesMarker;

#[derive(Clone, Debug, Default, Bundle)]
pub struct ColliderBundle {
    pub collider: Collider, // 碰撞体
    pub rigid_body: RigidBody, // 刚体
}

#[derive(Clone, Debug, Default, Bundle)]
pub struct AnimationBundle {
    pub timer: AnimationTimer,
    pub indices: AnimationIndices,
}

// 精灵表参数说明:
// #[sprite_sheet("纹理路径", 图块宽度, 图块高度, 列数, 行数, 内边距, 间距, 图块索引)]
// map.bmp 精灵表布局: 7列 x 1行, 共7个图块, 每个图块32x32像素
// 图块索引: 0=石墙, 1=铁墙, 2=树, 3=水(帧1), 4=水(帧2), 5=家, 6=家(摧毁)

#[derive(Bundle, LdtkEntity, Default)]
pub struct StoneWallBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    // 精灵表索引0: 石墙(可被子弹摧毁)
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct IronWallBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    // 精灵表索引1: 铁墙(不可被摧毁,除非使用高级武器)
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 1)]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct WaterBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    // 精灵表索引3: 水(动画帧1, 与索引4交替播放形成水波动画)
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 3)]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    pub annimation_bundle: AnimationBundle,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct HomeBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    // 精灵表索引5: 基地(玩家需要保护的目标)
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 5)]
    sprite_sheet: Sprite,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct Player1MarkerBundle {
    marker: Player1Marker,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct Player2MarkerBundle {
    marker: Player2Marker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct EnemiesMarkerBundle {
    marker: EnemiesMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "StoneWall" | "IronWall" | "Water" | "Home" => ColliderBundle {
                // 碰撞体大小: TILE_SIZE/2=16 (32x32像素的半尺寸,用于Rapier2D物理引擎)
                collider: Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.),
                rigid_body: RigidBody::Fixed, // 固定刚体: 物体静止不动,仅参与碰撞检测
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<&EntityInstance> for AnimationBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "Water" => ColliderBundle {
                // 水域碰撞体大小与普通地图元素相同
                collider: Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.),
                rigid_body: RigidBody::Fixed,
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<&EntityInstance> for AnimationBundle {
    fn from(entity_instance: &EntityInstance) -> AnimationBundle {
        match entity_instance.identifier.as_ref() {
            "Water" => AnimationBundle {
                // 动画定时器: 0.2秒切换一次, TimerMode::Repeating表示循环播放
                timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                // 动画帧索引: 从索引3(第一帧)到索引4(第二帧), 循环切换形成水波效果
                indices: AnimationIndices { first: 3, last: 4 },
            },
            _ => AnimationBundle::default(),
        }
    }
}

impl From<&EntityInstance> for LevelItem {
    fn from(entity_instance: &EntityInstance) -> LevelItem {
        match entity_instance.identifier.as_ref() {
            "StoneWall" => LevelItem::StoneWall,
            "IronWall" => LevelItem::IronWall,
            "Tree" => LevelItem::Tree,
            "Water" => LevelItem::Water,
            "Home" => LevelItem::Home,
            _ => LevelItem::None,
        }
    }
}

/// 加载并设置游戏关卡
///
/// 此函数负责加载 levels.ldtk 文件并创建 LDTK 世界，
/// 是游戏地图生成的入口点。
///
/// # 参数
/// - `commands`: Bevy 命令系统，用于创建实体
/// - `asset_server`: 资源服务器，用于加载 ldtk 文件
/// - `q_ldtk_world`: 查询是否已存在 LDTK 世界，避免重复加载
///
/// # 功能
/// 1. 检查是否已经加载了 LDTK 世界，避免重复加载
/// 2. 加载 levels.ldtk 文件并创建 LDTK 世界实体
/// 3. 设置地图的初始位置偏移
pub fn setup_levels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_ldtk_world: Query<(), With<LdtkProjectHandle>>,
) {
    if q_ldtk_world.iter().len() > 0 {
        // 从Paused状态进入时无需再load ldtk
        return;
    }
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels.ldtk").into(),
        transform: Transform::from_translation(Vec3::ZERO + LEVEL_TRANSLATION_OFFSET),
        ..Default::default()
    });
}

pub fn spawn_ldtk_entity(
    mut commands: Commands,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    for (_entity, transform, entity_instance) in entity_query.iter() {
        if entity_instance.identifier == *"Tree" {
            let map_texture_handle = asset_server.load("textures/map.bmp");
            // 重新构建精灵表布局: 7列 x 1行, 图块大小32x32
            let map_texture_atlas =
                TextureAtlasLayout::from_grid(UVec2::new(32, 32), 7, 1, None, None);
            let map_texture_atlas_handle = texture_atlases.add(map_texture_atlas);

            let mut translation = transform.translation + LEVEL_TRANSLATION_OFFSET;
            translation.z = SPRITE_TREE_ORDER; // 树木渲染层级(确保显示在其他元素之上)
            commands.spawn((
                LevelItem::Tree,
                Sprite {
                    image: map_texture_handle,
                    texture_atlas: Some(TextureAtlas {
                        index: 2, // 精灵表索引2: 树木
                        layout: map_texture_atlas_handle,
                    }),
                    ..default()
                },
                Transform::from_translation(translation),
            ));
        }
    }
}

// 水动画播放
pub fn animate_water(
    time: Res<Time>,
    mut query: Query<(
        &LevelItem,
        &mut AnimationTimer,
        &AnimationIndices,
        &mut Sprite,
    )>,
) {
    for (level_item, mut timer, indices, mut sprite) in &mut query {
        if *level_item == LevelItem::Water {
            timer.0.tick(time.delta());
            if timer.0.just_finished() {
                // 切换到下一个sprite
                if let Some(atlas) = &mut sprite.texture_atlas {
                    // indices.last=4, indices.first=3
                    // 当到达最后一帧(4)时,重置回第一帧(3),形成循环动画
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    };
                }
            }
        }
    }
}

pub fn auto_switch_level(
    mut commands: Commands,
    q_enemies: Query<(), With<Enemy>>,
    q_players: Query<Entity, With<PlayerNo>>,
    q_level_items: Query<Entity, With<LevelItem>>,
    mut level_selection: ResMut<LevelSelection>,
    mut level_spawned_enemies: ResMut<LevelSpawnedEnemies>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    // 已生成的敌人数量达到最大值 并且 敌人全部阵亡，切换到下一关卡
    if level_spawned_enemies.0 == ENEMIES_PER_LEVEL && q_enemies.iter().len() == 0 {
        if let LevelSelection::Indices(LevelIndices { level, .. }) = *level_selection {
            if level as i32 == MAX_LEVELS - 1 {
                // TODO 游戏胜利
                info!("win the game!");
                app_state.set(AppState::StartMenu);
            } else {
                // 下一关卡
                info!("Switch to next level, index={}", level + 1);
                *level_selection = LevelSelection::index(level + 1);
                level_spawned_enemies.0 = 0;

                // 重新生成玩家
                for player in &q_players {
                    commands.entity(player).despawn_recursive();
                }
                for level_item in &q_level_items {
                    commands.entity(level_item).despawn_recursive();
                }
            }
        }
    }
}

pub fn animate_home(
    mut home_dying_er: EventReader<HomeDyingEvent>,
    mut q_level_items: Query<(&LevelItem, &mut Sprite)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for _ in home_dying_er.read() {
        for (level_item, mut sprite) in &mut q_level_items {
            if *level_item == LevelItem::Home {
                // 基地被摧毁时,切换到精灵表索引6(基地摧毁状态图)
                sprite.texture_atlas.as_mut().unwrap().index = 6;
                app_state.set(AppState::GameOver);
            }
        }
    }
}

pub fn cleanup_level_items(mut commands: Commands, q_level_items: Query<Entity, With<LevelItem>>) {
    for entity in &q_level_items {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn cleanup_ldtk_world(
    mut commands: Commands,
    q_ldtk_world: Query<Entity, With<LdtkProjectHandle>>,
) {
    for entity in &q_ldtk_world {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn reset_level_selection(mut level_selection: ResMut<LevelSelection>) {
    *level_selection = LevelSelection::index(0);
}
