#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{App as EframeApp, CreationContext, NativeOptions, egui};
use egui::Rect;
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, Snarl,
    ui::{BackgroundPattern, Grid, PinInfo, SnarlPin, SnarlStyle, SnarlViewer, WireStyle},
};
use std::collections::{HashMap, HashSet};

// Define a simple node type
#[derive(PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
enum DijkstraNode {
    Start,
    Distance(Vec<i32>),
    Finish,
}

struct DijkstraViewer {
    highlighted_nodes: HashSet<NodeId>,
    stored_nodes: HashMap<NodeId, Rect>,
}

impl DijkstraViewer {
    fn new() -> Self {
        Self {
            highlighted_nodes: HashSet::new(),
            stored_nodes: HashMap::new(),
        }
    }

    fn toggle_highlight(&mut self, node_id: NodeId) {
        if self.highlighted_nodes.contains(&node_id) {
            self.highlighted_nodes.remove(&node_id);
        } else {
            self.highlighted_nodes.insert(node_id);
        }
    }

    fn is_highlighted(&self, node_id: NodeId) -> bool {
        self.highlighted_nodes.contains(&node_id)
    }
}

impl SnarlViewer<DijkstraNode> for DijkstraViewer {
    fn title(&mut self, node: &DijkstraNode) -> String {
        match node {
            DijkstraNode::Start => "Start".to_string(),
            DijkstraNode::Distance(_) => "Distance".to_string(),
            DijkstraNode::Finish => "Finish".to_string(),
        }
    }

    fn inputs(&mut self, node: &DijkstraNode) -> usize {
        match node {
            DijkstraNode::Start => 0,
            DijkstraNode::Distance(_) => 1,
            DijkstraNode::Finish => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node] {
            DijkstraNode::Start => {
                if let Some(remote) = pin.remotes.iter().next() {
                    match &snarl[remote.node] {
                        DijkstraNode::Distance(values) => {
                            ui.label(format!("Values: {:?}", values));
                        }
                        _ => {
                            ui.label("Invalid input");
                        }
                    }
                } else {
                    ui.label("No input");
                }
                PinInfo::default()
            }
            _ => PinInfo::default(),
        }
    }

    fn outputs(&mut self, node: &DijkstraNode) -> usize {
        match node {
            DijkstraNode::Start => 1,
            DijkstraNode::Distance(_) => 1,
            DijkstraNode::Finish => 0,
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) -> impl SnarlPin + 'static {
        match &snarl[pin.id.node] {
            DijkstraNode::Distance(values) => {
                ui.label(format!("Values: {:?}", values));
                PinInfo::default()
            }
            _ => PinInfo::default(),
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<DijkstraNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) {
        ui.label("Add node");
        if snarl.nodes().all(|node| *node != DijkstraNode::Start) {
            if ui.button("Start").clicked() {
                snarl.insert_node(pos, DijkstraNode::Start);
                ui.close_menu();
            }
        }
        if ui.button("Value").clicked() {
            snarl.insert_node(pos, DijkstraNode::Distance(vec![1]));
            ui.close_menu();
        }
        if snarl.nodes().all(|node| *node != DijkstraNode::Finish) {
            if ui.button("Finish").clicked() {
                snarl.insert_node(pos, DijkstraNode::Finish);
                ui.close_menu();
            }
        }
    }

    fn has_node_menu(&mut self, _node: &DijkstraNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) {
        ui.label("Add node");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<DijkstraNode>) {
        if let (DijkstraNode::Start, DijkstraNode::Distance(_)) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
        if let (DijkstraNode::Distance(_), DijkstraNode::Distance(_)) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
        if let (DijkstraNode::Distance(_), DijkstraNode::Finish) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
    }

    fn has_node_style(
        &mut self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        _snarl: &Snarl<DijkstraNode>,
    ) -> bool {
        true
    }
    fn apply_node_style(
        &mut self,
        style: &mut egui::Style,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        _snarl: &Snarl<DijkstraNode>,
    ) {
        // Set node style based on highlight state
        if self.is_highlighted(node) {
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 0, 0);
            // style.visuals.widgets.noninteractive.fg_stroke =
            // egui::Stroke::new(1.0, egui::Color32::WHITE);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(255, 0, 0);
            // style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(210, 110, 110);
        } else {
            // create basic a wire gradient for nodes input to output
            // don't use the distance value
            match _snarl[node] {
                DijkstraNode::Start => {
                    // Green color for start nodes
                    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 180, 50);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 200, 70);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 220, 90);
                },
                DijkstraNode::Distance(_) => {
                    // Blue color for intermediate nodes
                    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 120, 200);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 140, 220);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 160, 240);
                },
                DijkstraNode::Finish => {
                    // Orange color for end nodes
                    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(220, 120, 50);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(240, 140, 70);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 160, 90);
                }
            }
        }
    }
    fn final_node_rect(
        &mut self,
        node: NodeId,
        _ui_rect: egui::Rect,
        graph_rect: egui::Rect,
        _ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) {
        self.stored_nodes.insert(node, graph_rect);
        if self.stored_nodes.len() == snarl.nodes().count() {
            for (node_id, node_rect) in self.stored_nodes.iter() {
                let op = InPinId {
                    node: *node_id,
                    input: 0,
                };
                // Végig megyünk minden bemeneti pin-en
                for remote in snarl.in_pin(op).remotes {
                    // Kiírjuk a távolságot
                    let parent_node_rect = self.stored_nodes.get(&remote.node);
                    if let Some(parent_node) = parent_node_rect {
                        let dist: f32 = ((node_rect.left_center().x
                            - parent_node.right_center().x)
                            .powi(2)
                            + (node_rect.left_center().y - parent_node.right_center().y).powi(2))
                        .sqrt();
                        // Beállítjuk a távolságot a csomópontban
                        snarl.get_node_info_mut(*node_id).unwrap().value =
                            DijkstraNode::Distance(vec![(dist.round() as i32) / 10]);
                    }
                }
            }
        }
    }
}

// Implement the eframe::App trait
struct DijkstraApp {
    snarl: Snarl<DijkstraNode>,
    style: SnarlStyle,
    viewer: DijkstraViewer,
}

impl DijkstraApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let mut ss = SnarlStyle::new();
        ss.collapsible = Some(false);
        ss.pin_placement = Some(egui_snarl::ui::PinPlacement::Edge);
        ss.wire_style = Some(WireStyle::Bezier5);
        ss.max_scale = Some(1.0);
        ss.bg_pattern = Some(BackgroundPattern::Grid(Grid::new(
            egui::vec2(30.0, 30.0),
            0.0,
        )));
        ss.wire_width = Some(2.0);
        DijkstraApp {
            snarl: Snarl::new(),
            style: ss,
            viewer: DijkstraViewer::new(),
        }
    }
}

impl EframeApp for DijkstraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            if ui.button("Save").clicked() {
                // Serialize the snarl data to a string using JSON
                let serialized = serde_json::to_string_pretty(&self.snarl).unwrap_or_else(|err| {
                    eprintln!("Failed to serialize snarl: {}", err);
                    String::new()
                });

                // Save the serialized data to a file
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(".json")
                    .add_filter("JSON", &["json"])
                    .set_directory(&std::env::current_dir().unwrap())
                    .save_file()
                {
                    std::fs::write(&path, serialized).unwrap_or_else(|err| {
                        eprintln!("Failed to write to file: {}", err);
                    });
                }
            }
            if ui.button("Load").clicked() {
                // Load the serialized data from a file
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_directory(&std::env::current_dir().unwrap())
                    .pick_file()
                {
                    let serialized = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                        eprintln!("Failed to read file: {}", err);
                        String::new()
                    });

                    // Deserialize the snarl data from the string
                    self.snarl = serde_json::from_str(&serialized).unwrap_or_else(|err| {
                        eprintln!("Failed to deserialize snarl: {}", err);
                        Snarl::new()
                    });
                }
            }
        });
        egui::Window::new("Kalkulátor").show(ctx, |ui| {
            ui.label("Actions");
            if ui.button("Toggle Node Styles").clicked() {
                // Find value nodes and toggle their highlighting
                for (node_id, _) in self.snarl.nodes_ids_data() {
                    self.viewer.toggle_highlight(node_id);
                }
            }
            if ui.button("Remove all").clicked() {
                for (node_id, _) in self.snarl.clone().nodes_ids_data() {
                    self.snarl.remove_node(node_id);
                }
            }
            if ui.button("Calc distance").clicked() {
                for (pos, node) in self.snarl.clone().nodes_pos() {
                    println!("Node: {:?}", pos);
                    println!("Node: {:?}", node);
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl.show(&mut self.viewer, &self.style, "salty", ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Visualize dijkstra's algorithm",
        native_options,
        Box::new(|cc| Ok(Box::new(DijkstraApp::new(cc)))),
    )
}
