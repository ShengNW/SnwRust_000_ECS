use anyhow::Result;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MEMORY_CSV_FILE: &str = "memories.csv";
const TODO_CSV_FILE: &str = "todos.csv";

#[derive(Component, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
struct CoreMind {
    energy: f32,
    mood: f32,
    memory_capacity: usize,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
struct MemoryData {
    id: u64,
    content: String,
    weight: f32,
    decay_rate: f32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct ShortTermMemory;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct LongTermMemory;

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
enum TodoLayer {
    Urgent,
    #[default]
    Daily,
    LifePlan,
}

impl TodoLayer {
    fn label(self) -> &'static str {
        match self {
            Self::Urgent => "Urgent",
            Self::Daily => "Daily",
            Self::LifePlan => "LifePlan",
        }
    }

    fn from_label(label: &str) -> Self {
        match label {
            "Urgent" => Self::Urgent,
            "LifePlan" => Self::LifePlan,
            _ => Self::Daily,
        }
    }

    fn all() -> [Self; 3] {
        [Self::Urgent, Self::Daily, Self::LifePlan]
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
struct TodoData {
    id: u64,
    title: String,
    progress: f32,
    deadline_days: u32,
    layer: TodoLayer,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct MindConfig {
    auto_decay: bool,
    decay_tick_per_second: f32,
    short_term_capacity: usize,
}

impl Default for MindConfig {
    fn default() -> Self {
        Self {
            auto_decay: true,
            decay_tick_per_second: 0.12,
            short_term_capacity: 8,
        }
    }
}

#[derive(Resource, Default)]
struct IdSeed {
    next_memory: u64,
    next_todo: u64,
}

#[derive(Resource)]
struct UiState {
    new_memory: String,
    new_memory_weight: f32,
    new_memory_decay: f32,
    new_todo: String,
    new_todo_deadline_days: u32,
    selected_todo_layer: TodoLayer,
    save_load_notice: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            new_memory: String::new(),
            new_memory_weight: 60.0,
            new_memory_decay: 0.15,
            new_todo: String::new(),
            new_todo_deadline_days: 3,
            selected_todo_layer: TodoLayer::Daily,
            save_load_notice: String::from("Ready"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MemoryCsvRecord {
    id: u64,
    layer: String,
    content: String,
    weight: f32,
    decay_rate: f32,
}

#[derive(Serialize, Deserialize)]
struct TodoCsvRecord {
    id: u64,
    layer: String,
    title: String,
    progress: f32,
    deadline_days: u32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SNW ECS Mind Studio".to_owned(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), WorldInspectorPlugin::new()))
        .register_type::<CoreMind>()
        .register_type::<MemoryData>()
        .register_type::<ShortTermMemory>()
        .register_type::<LongTermMemory>()
        .register_type::<TodoData>()
        .register_type::<TodoLayer>()
        .register_type::<MindConfig>()
        .init_resource::<MindConfig>()
        .init_resource::<IdSeed>()
        .init_resource::<UiState>()
        .add_systems(Startup, setup_world)
        .add_systems(
            Update,
            (memory_decay_system, enforce_short_term_capacity_system),
        )
        .add_systems(EguiPrimaryContextPass, mind_console_ui_system)
        .run();
}

fn setup_world(mut commands: Commands, mut id_seed: ResMut<IdSeed>) {
    commands.spawn(Camera2d);

    commands.spawn(CoreMind {
        energy: 78.0,
        mood: 68.0,
        memory_capacity: 8,
    });

    let _ = spawn_memory(
        &mut commands,
        &mut id_seed,
        "临时灵感：把记忆衰减和待办压力联动".to_owned(),
        72.0,
        0.18,
        true,
    );
    let _ = spawn_memory(
        &mut commands,
        &mut id_seed,
        "长期沉淀：Rust + ECS 是项目内核".to_owned(),
        92.0,
        0.01,
        false,
    );

    let _ = spawn_todo(
        &mut commands,
        &mut id_seed,
        "梳理本周可交付里程碑".to_owned(),
        TodoLayer::Urgent,
        1,
        0.0,
    );
    let _ = spawn_todo(
        &mut commands,
        &mut id_seed,
        "整理记忆 CSV 字段规范".to_owned(),
        TodoLayer::Daily,
        2,
        20.0,
    );
}

fn memory_decay_system(
    time: Res<Time>,
    config: Res<MindConfig>,
    mut memories: Query<&mut MemoryData, With<ShortTermMemory>>,
) {
    if !config.auto_decay {
        return;
    }

    let decay_step = config.decay_tick_per_second * time.delta_secs();

    for mut memory in &mut memories {
        let local_decay = (memory.decay_rate.max(0.0) + decay_step).max(0.0);
        memory.weight = (memory.weight - local_decay).clamp(0.0, 100.0);
    }
}

fn enforce_short_term_capacity_system(
    mut commands: Commands,
    config: Res<MindConfig>,
    memories: Query<(Entity, &MemoryData), With<ShortTermMemory>>,
) {
    let mut ranked: Vec<(Entity, f32)> = memories.iter().map(|(e, m)| (e, m.weight)).collect();
    let count = ranked.len();

    if count <= config.short_term_capacity {
        return;
    }

    ranked.sort_by(|a, b| a.1.total_cmp(&b.1));
    let overflow = count - config.short_term_capacity;

    for (entity, _) in ranked.into_iter().take(overflow) {
        commands.entity(entity).despawn();
    }
}

fn mind_console_ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut config: ResMut<MindConfig>,
    mut core_mind_query: Query<&mut CoreMind>,
    mut ui_state: ResMut<UiState>,
    mut id_seed: ResMut<IdSeed>,
    all_memories: Query<(
        Entity,
        &MemoryData,
        Option<&ShortTermMemory>,
        Option<&LongTermMemory>,
    )>,
    short_memories: Query<(Entity, &MemoryData), With<ShortTermMemory>>,
    todos: Query<(Entity, &TodoData)>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::left("mind_console")
        .resizable(true)
        .default_width(460.0)
        .show(ctx, |ui| {
            ui.heading("SNW ECS Mind Studio");
            ui.label("可视化创建/固化记忆、管理待办、导入导出 CSV");
            ui.separator();

            ui.heading("1) 全局心智参数");
            ui.checkbox(&mut config.auto_decay, "开启短期记忆自动衰减");
            ui.add(
                egui::Slider::new(&mut config.decay_tick_per_second, 0.0..=3.0)
                    .text("全局衰减强度/秒"),
            );

            let mut ui_capacity = i32::try_from(config.short_term_capacity).unwrap_or(8);
            if ui
                .add(egui::Slider::new(&mut ui_capacity, 1..=64).text("短期记忆容量"))
                .changed()
            {
                config.short_term_capacity = ui_capacity as usize;
            }

            if let Ok(mut core_mind) = core_mind_query.single_mut() {
                ui.add(egui::Slider::new(&mut core_mind.energy, 0.0..=100.0).text("能量"));
                ui.add(egui::Slider::new(&mut core_mind.mood, 0.0..=100.0).text("情绪"));
                core_mind.memory_capacity = config.short_term_capacity;
            }

            ui.separator();
            ui.heading("2) 记忆操作");
            ui.horizontal(|ui| {
                ui.label("内容");
                ui.text_edit_singleline(&mut ui_state.new_memory);
            });
            ui.add(
                egui::Slider::new(&mut ui_state.new_memory_weight, 0.0..=100.0).text("初始权重"),
            );
            ui.add(
                egui::Slider::new(&mut ui_state.new_memory_decay, 0.0..=2.0)
                    .text("记忆自身衰减参数"),
            );

            ui.horizontal(|ui| {
                if ui.button("新增短期记忆").clicked() {
                    let content = ui_state.new_memory.trim().to_owned();
                    if content.is_empty() {
                        ui_state.save_load_notice = "新增失败：记忆内容不能为空".to_owned();
                    } else {
                        let _ = spawn_memory(
                            &mut commands,
                            &mut id_seed,
                            content,
                            ui_state.new_memory_weight,
                            ui_state.new_memory_decay,
                            true,
                        );
                        ui_state.new_memory.clear();
                    }
                }

                if ui.button("新增长期记忆").clicked() {
                    let content = ui_state.new_memory.trim().to_owned();
                    if content.is_empty() {
                        ui_state.save_load_notice = "新增失败：记忆内容不能为空".to_owned();
                    } else {
                        let _ = spawn_memory(
                            &mut commands,
                            &mut id_seed,
                            content,
                            ui_state.new_memory_weight,
                            ui_state.new_memory_decay,
                            false,
                        );
                        ui_state.new_memory.clear();
                    }
                }
            });

            if ui.button("固化最强短期记忆 -> 长期记忆").clicked() {
                let strongest = short_memories
                    .iter()
                    .max_by(|(_, a), (_, b)| a.weight.total_cmp(&b.weight))
                    .map(|(entity, _)| entity);

                if let Some(entity) = strongest {
                    commands.entity(entity).remove::<ShortTermMemory>();
                    commands.entity(entity).insert(LongTermMemory);
                    ui_state.save_load_notice = "已固化 1 条记忆".to_owned();
                } else {
                    ui_state.save_load_notice = "没有可固化的短期记忆".to_owned();
                }
            }

            ui.separator();
            ui.heading("3) 待办操作");
            ui.horizontal(|ui| {
                ui.label("标题");
                ui.text_edit_singleline(&mut ui_state.new_todo);
            });

            egui::ComboBox::from_label("层级")
                .selected_text(ui_state.selected_todo_layer.label())
                .show_ui(ui, |ui| {
                    for layer in TodoLayer::all() {
                        ui.selectable_value(
                            &mut ui_state.selected_todo_layer,
                            layer,
                            layer.label(),
                        );
                    }
                });

            let mut deadline_days_i32 = ui_state.new_todo_deadline_days as i32;
            if ui
                .add(egui::Slider::new(&mut deadline_days_i32, 1..=365).text("截止天数"))
                .changed()
            {
                ui_state.new_todo_deadline_days = deadline_days_i32 as u32;
            }

            if ui.button("新增待办").clicked() {
                let title = ui_state.new_todo.trim().to_owned();
                if title.is_empty() {
                    ui_state.save_load_notice = "新增失败：待办标题不能为空".to_owned();
                } else {
                    let _ = spawn_todo(
                        &mut commands,
                        &mut id_seed,
                        title,
                        ui_state.selected_todo_layer,
                        ui_state.new_todo_deadline_days,
                        0.0,
                    );
                    ui_state.new_todo.clear();
                }
            }

            ui.separator();
            ui.heading("4) CSV 存取");
            ui.horizontal(|ui| {
                if ui.button("保存到 CSV").clicked() {
                    let memory_rows: Vec<MemoryCsvRecord> = all_memories
                        .iter()
                        .map(|(_, memory, is_short, _)| MemoryCsvRecord {
                            id: memory.id,
                            layer: if is_short.is_some() {
                                "short".to_owned()
                            } else {
                                "long".to_owned()
                            },
                            content: memory.content.clone(),
                            weight: memory.weight,
                            decay_rate: memory.decay_rate,
                        })
                        .collect();

                    let todo_rows: Vec<TodoCsvRecord> = todos
                        .iter()
                        .map(|(_, todo)| TodoCsvRecord {
                            id: todo.id,
                            layer: todo.layer.label().to_owned(),
                            title: todo.title.clone(),
                            progress: todo.progress,
                            deadline_days: todo.deadline_days,
                        })
                        .collect();

                    match save_snapshot(&memory_rows, &todo_rows) {
                        Ok(()) => {
                            ui_state.save_load_notice = format!(
                                "保存成功：{} memory / {} todo",
                                memory_rows.len(),
                                todo_rows.len()
                            );
                        }
                        Err(err) => {
                            ui_state.save_load_notice = format!("保存失败：{err}");
                        }
                    }
                }

                if ui.button("从 CSV 载入").clicked() {
                    match load_snapshot() {
                        Ok((memory_rows, todo_rows)) => {
                            if memory_rows.is_empty() && todo_rows.is_empty() {
                                ui_state.save_load_notice =
                                    "未找到 CSV 文件，或文件内容为空".to_owned();
                            } else {
                                let memory_entities: Vec<Entity> =
                                    all_memories.iter().map(|(e, _, _, _)| e).collect();
                                let todo_entities: Vec<Entity> =
                                    todos.iter().map(|(e, _)| e).collect();

                                for entity in memory_entities {
                                    commands.entity(entity).despawn();
                                }
                                for entity in todo_entities {
                                    commands.entity(entity).despawn();
                                }

                                let mut max_memory_id = 0u64;
                                for row in memory_rows {
                                    max_memory_id = max_memory_id.max(row.id);
                                    let memory = MemoryData {
                                        id: row.id,
                                        content: row.content,
                                        weight: row.weight,
                                        decay_rate: row.decay_rate,
                                    };
                                    if row.layer == "short" {
                                        commands.spawn((ShortTermMemory, memory));
                                    } else {
                                        commands.spawn((LongTermMemory, memory));
                                    }
                                }

                                let mut max_todo_id = 0u64;
                                for row in todo_rows {
                                    max_todo_id = max_todo_id.max(row.id);
                                    commands.spawn(TodoData {
                                        id: row.id,
                                        title: row.title,
                                        progress: row.progress,
                                        deadline_days: row.deadline_days,
                                        layer: TodoLayer::from_label(&row.layer),
                                    });
                                }

                                id_seed.next_memory = max_memory_id.saturating_add(1);
                                id_seed.next_todo = max_todo_id.saturating_add(1);
                                ui_state.save_load_notice = "CSV 载入成功".to_owned();
                            }
                        }
                        Err(err) => {
                            ui_state.save_load_notice = format!("载入失败：{err}");
                        }
                    }
                }
            });

            ui.label(format!("状态：{}", ui_state.save_load_notice));

            ui.separator();
            ui.heading("5) 运行态快照");
            let short_count = all_memories
                .iter()
                .filter(|(_, _, s, _)| s.is_some())
                .count();
            let long_count = all_memories
                .iter()
                .filter(|(_, _, _, l)| l.is_some())
                .count();
            ui.label(format!("短期记忆：{short_count}"));
            ui.label(format!("长期记忆：{long_count}"));
            ui.label(format!("待办总数：{}", todos.iter().count()));

            ui.collapsing("记忆列表", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for (_, memory, is_short, _) in &all_memories {
                            let layer = if is_short.is_some() { "S" } else { "L" };
                            ui.label(format!(
                                "[{layer}] #{} | w={:.1} d={:.2} | {}",
                                memory.id, memory.weight, memory.decay_rate, memory.content
                            ));
                        }
                    });
            });

            ui.collapsing("待办列表", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for (_, todo) in &todos {
                            ui.label(format!(
                                "[{}] #{} | {:.0}% | D+{} | {}",
                                todo.layer.label(),
                                todo.id,
                                todo.progress,
                                todo.deadline_days,
                                todo.title
                            ));
                        }
                    });
            });

            ui.separator();
            ui.small("提示：按 `~` 键可打开/关闭 bevy-inspector-egui 的世界检查器");
        });
}

fn spawn_memory(
    commands: &mut Commands,
    id_seed: &mut IdSeed,
    content: String,
    weight: f32,
    decay_rate: f32,
    short_term: bool,
) -> u64 {
    let id = id_seed.next_memory;
    id_seed.next_memory = id_seed.next_memory.saturating_add(1);

    let memory = MemoryData {
        id,
        content,
        weight: weight.clamp(0.0, 100.0),
        decay_rate: decay_rate.clamp(0.0, 5.0),
    };

    if short_term {
        commands.spawn((ShortTermMemory, memory));
    } else {
        commands.spawn((LongTermMemory, memory));
    }

    id
}

fn spawn_todo(
    commands: &mut Commands,
    id_seed: &mut IdSeed,
    title: String,
    layer: TodoLayer,
    deadline_days: u32,
    progress: f32,
) -> u64 {
    let id = id_seed.next_todo;
    id_seed.next_todo = id_seed.next_todo.saturating_add(1);

    commands.spawn(TodoData {
        id,
        title,
        progress: progress.clamp(0.0, 100.0),
        deadline_days: deadline_days.max(1),
        layer,
    });

    id
}

fn save_snapshot(memory_rows: &[MemoryCsvRecord], todo_rows: &[TodoCsvRecord]) -> Result<()> {
    let data_dir = data_dir();
    std::fs::create_dir_all(&data_dir)?;

    let mut memory_writer = csv::Writer::from_path(data_dir.join(MEMORY_CSV_FILE))?;
    for row in memory_rows {
        memory_writer.serialize(row)?;
    }
    memory_writer.flush()?;

    let mut todo_writer = csv::Writer::from_path(data_dir.join(TODO_CSV_FILE))?;
    for row in todo_rows {
        todo_writer.serialize(row)?;
    }
    todo_writer.flush()?;

    Ok(())
}

fn load_snapshot() -> Result<(Vec<MemoryCsvRecord>, Vec<TodoCsvRecord>)> {
    let data_dir = data_dir();
    let memory_path = data_dir.join(MEMORY_CSV_FILE);
    let todo_path = data_dir.join(TODO_CSV_FILE);

    let mut memory_rows = Vec::new();
    if memory_path.exists() {
        let mut reader = csv::Reader::from_path(memory_path)?;
        for row in reader.deserialize() {
            memory_rows.push(row?);
        }
    }

    let mut todo_rows = Vec::new();
    if todo_path.exists() {
        let mut reader = csv::Reader::from_path(todo_path)?;
        for row in reader.deserialize() {
            todo_rows.push(row?);
        }
    }

    Ok((memory_rows, todo_rows))
}

fn data_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
}

#[cfg(test)]
mod tests {
    use super::TodoLayer;

    #[test]
    fn todo_layer_parse_roundtrip() {
        for layer in TodoLayer::all() {
            let parsed = TodoLayer::from_label(layer.label());
            assert_eq!(layer, parsed);
        }
    }
}
