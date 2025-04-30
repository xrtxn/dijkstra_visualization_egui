#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{App as EframeApp, CreationContext, NativeOptions, egui};
use egui::{Color32, Rect, WidgetText};
use egui_notify::Toasts;
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{BackgroundPattern, Grid, PinInfo, SnarlPin, SnarlStyle, SnarlViewer, WireStyle},
};

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

const NOTIFICATION_DURATION: u64 = 5;

// Define a simple node type
#[derive(PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
enum DijkstraNode {
    Start,
    Distance(HashMap<NodeId, i32>),
    Finish(HashMap<NodeId, i32>),
}

struct DijkstraViewer {
    stored_nodes: HashMap<NodeId, Rect>,
    toasts: Toasts,
    path_nodes: Vec<NodeId>,
}

impl DijkstraViewer {
    fn new() -> Self {
        Self {
            stored_nodes: HashMap::new(),
            toasts: Toasts::default(),
            path_nodes: Vec::new(),
        }
    }

    fn add_error_notification(&mut self, msg: impl Into<WidgetText>) {
        self.toasts
            .error(msg)
            .duration(Some(Duration::from_secs(NOTIFICATION_DURATION)));
    }

    fn add_success_notification(&mut self, msg: impl Into<WidgetText>) {
        self.toasts
            .success(msg)
            .duration(Some(Duration::from_secs(NOTIFICATION_DURATION)));
    }
}

impl SnarlViewer<DijkstraNode> for DijkstraViewer {
    fn title(&mut self, node: &DijkstraNode) -> String {
        match node {
            DijkstraNode::Start => "Start".to_string(),
            DijkstraNode::Distance(_) => "Distance".to_string(),
            DijkstraNode::Finish(_) => "Finish".to_string(),
        }
    }

    fn inputs(&mut self, node: &DijkstraNode) -> usize {
        match node {
            DijkstraNode::Start => 0,
            DijkstraNode::Distance(_) => 1, // Allow multiple inputs
            DijkstraNode::Finish(_) => 1,   // Allow multiple inputs
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) -> impl SnarlPin + 'static {
        match &snarl[pin.id.node] {
            DijkstraNode::Distance(values) => {
                let color: Color32;
                if self.path_nodes.contains(&pin.id.node) {
                    color = Color32::RED;
                } else {
                    color = Color32::BLUE;
                }

                // Display all remote nodes and their costs
                if !snarl.in_pin(pin.id).remotes.is_empty() {
                    ui.vertical(|ui| {
                        for remote in &snarl.in_pin(pin.id).remotes {
                            let remote_node = remote.node;
                            if let Some(&cost) = values.get(&remote_node) {
                                ui.label(format!("Node {}: cost {}", remote_node.0, cost));
                            }
                        }
                    });
                }

                PinInfo::triangle().with_fill(color)
            }
            DijkstraNode::Finish(hash_map) => {
                for node in self.path_nodes.iter() {
                    if hash_map.contains_key(node) {
                        ui.label(format!("Cost: {}", hash_map.get(node).unwrap()));
                        break;
                    }
                }
                PinInfo::triangle()
            }
            DijkstraNode::Start => unreachable!(),
        }
    }

    fn outputs(&mut self, node: &DijkstraNode) -> usize {
        match node {
            DijkstraNode::Start => 1,       // Allow multiple outputs
            DijkstraNode::Distance(_) => 1, // Allow multiple outputs
            DijkstraNode::Finish(_) => 0,
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<DijkstraNode>,
    ) -> impl SnarlPin + 'static {
        match &snarl[pin.id.node] {
            DijkstraNode::Distance(_) => {
                let color: Color32;
                if self.path_nodes.contains(&pin.id.node) {
                    color = Color32::RED;
                } else {
                    color = Color32::BLUE;
                }
                PinInfo::circle().with_fill(color)
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
        if snarl.nodes().all(|node| match node {
            DijkstraNode::Start => false,
            _ => true,
        }) {
            if ui.button("Start").clicked() {
                snarl.insert_node(pos, DijkstraNode::Start);
                ui.close_menu();
            }
        }
        if ui.button("Value").clicked() {
            snarl.insert_node(pos, DijkstraNode::Distance(HashMap::new()));
            ui.close_menu();
        }
        if snarl.nodes().all(|node| match node {
            DijkstraNode::Finish(_) => false,
            _ => true,
        }) {
            if ui.button("Finish").clicked() {
                snarl.insert_node(pos, DijkstraNode::Finish(HashMap::new()));
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
        ui.label("Node Options");
        if ui.button("Remove").clicked() {
            self.stored_nodes.remove(&node);
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<DijkstraNode>) {
        // Allow all valid connections
        match (&snarl[from.id.node], &snarl[to.id.node]) {
            (DijkstraNode::Start, DijkstraNode::Distance(_)) => {
                snarl.connect(from.id, to.id);
            }
            (DijkstraNode::Distance(_), DijkstraNode::Distance(_)) => {
                // Allow connections between distance nodes
                snarl.connect(from.id, to.id);
            }
            (DijkstraNode::Distance(_), DijkstraNode::Finish(_)) => {
                snarl.connect(from.id, to.id);
            }
            _ => {}
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
            // Update all connections with distances
            for (node_id, node_rect) in self.stored_nodes.iter() {
                match &snarl[*node_id] {
                    DijkstraNode::Start => {}
                    DijkstraNode::Distance(_) => {
                        let mut costs = HashMap::new();
                        // Check all inputs to this node
                        for input_idx in 0..10 {
                            // Check all possible input pins
                            let ip = InPinId {
                                node: *node_id,
                                input: input_idx,
                            };

                            // For each connected input, calculate distance
                            for remote in snarl.in_pin(ip).remotes.iter() {
                                let parent_node_rect = self.stored_nodes.get(&remote.node);
                                if let Some(parent_node) = parent_node_rect {
                                    let dist: f32 = ((node_rect.left_center().x
                                        - parent_node.right_center().x)
                                        .powi(2)
                                        + (node_rect.left_center().y
                                            - parent_node.right_center().y)
                                            .powi(2))
                                    .sqrt();

                                    // Calculate cost from distance
                                    let cost = (dist.round() as i32) / 10;
                                    costs.insert(remote.node, if cost < 1 { 1 } else { cost });
                                }
                            }
                        }

                        // Update the node with all costs
                        if !costs.is_empty() {
                            snarl.get_node_info_mut(*node_id).unwrap().value =
                                DijkstraNode::Distance(costs);
                        }
                    }
                    DijkstraNode::Finish(_) => {
                        let mut costs = HashMap::new();
                        // Check all inputs to this node
                        for input_idx in 0..10 {
                            // Check all possible input pins
                            let ip = InPinId {
                                node: *node_id,
                                input: input_idx,
                            };

                            // For each connected input, calculate distance
                            for remote in snarl.in_pin(ip).remotes.iter() {
                                let parent_node_rect = self.stored_nodes.get(&remote.node);
                                if let Some(parent_node) = parent_node_rect {
                                    let dist: f32 = ((node_rect.left_center().x
                                        - parent_node.right_center().x)
                                        .powi(2)
                                        + (node_rect.left_center().y
                                            - parent_node.right_center().y)
                                            .powi(2))
                                    .sqrt();

                                    // Calculate cost from distance
                                    let cost = (dist.round() as i32) / 10;
                                    costs.insert(remote.node, if cost < 1 { 1 } else { cost });
                                }
                            }
                        }

                        // Update the node with all costs
                        if !costs.is_empty() {
                            snarl.get_node_info_mut(*node_id).unwrap().value =
                                DijkstraNode::Finish(costs);
                        }
                    }
                }
            }
        }
    }
}

// Priority queue element for Dijkstra's algorithm
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct State {
    cost: i32,
    node: NodeId,
}

// Implement Ord for our priority queue
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.node.cmp(&other.node))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implement the eframe::App trait
struct DijkstraApp {
    snarl: Snarl<DijkstraNode>,
    style: SnarlStyle,
    viewer: DijkstraViewer,
    auto_recalc: bool,
    total_cost: i32,
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
            auto_recalc: false,
            total_cost: 1,
        }
    }

    fn run_dijkstra(&mut self) -> Result<Vec<NodeId>, String> {
        let mut start_node = None;
        let mut finish_node = None;

        // Find start and finish nodes
        for (node_id, node) in self.snarl.nodes_ids_data() {
            match node.value {
                DijkstraNode::Start => start_node = Some(node_id),
                DijkstraNode::Finish(_) => finish_node = Some(node_id),
                _ => {}
            }
        }

        let start = start_node.ok_or("Start node not found".to_string())?;
        let finish = finish_node.ok_or("Finish node not found".to_string())?;

        // Manual implementation of Dijkstra's algorithm
        let mut dist: HashMap<NodeId, i32> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

        // Initialize distances to infinity (i32::MAX)
        for (node_id, _) in self.snarl.nodes_ids_data() {
            dist.insert(node_id, i32::MAX);
        }

        // Distance to start node is 0
        dist.insert(start, 0);
        priority_queue.push(State {
            cost: 0,
            node: start,
        });

        // Process nodes
        while let Some(State { cost, node }) = priority_queue.pop() {
            // If we reached the target node, we're done
            if node == finish {
                break;
            }

            // Skip if we already found a better path
            if cost > dist[&node] {
                continue;
            }

            // Process outgoing connections from all output pins
            for output_idx in 0..10 {
                // Check all possible output pins
                let op = OutPinId {
                    node,
                    output: output_idx,
                };

                for remote in self.snarl.out_pin(op).remotes {
                    let edge_cost = match &self.snarl[remote.node] {
                        DijkstraNode::Distance(costs) => {
                            // Get cost from the hashmap that stores costs from connected nodes
                            *costs.get(&node).unwrap_or(&1)
                        }
                        DijkstraNode::Finish(hash_map) => {
                            // If the node is a finish node, we need to get the cost from the hash map
                            *hash_map.get(&node).unwrap_or(&0)
                        }
                        _ => 0, // Default cost for other node types
                    };

                    let next = State {
                        cost: cost + edge_cost,
                        node: remote.node,
                    };

                    // If we found a better path
                    if next.cost < dist[&remote.node] {
                        dist.insert(remote.node, next.cost);
                        prev.insert(remote.node, node);
                        priority_queue.push(next);
                    }
                }
            }
        }

        // Reconstruct the path if one exists
        if prev.contains_key(&finish) || finish == start {
            let mut path = Vec::new();
            let mut current = finish;
            path.push(current);

            while current != start {
                current = prev[&current];
                path.insert(0, current);
            }

            self.total_cost = dist[&finish];

            Ok(path)
        } else {
            Err("No path found".to_string())
        }
    }
}

impl EframeApp for DijkstraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.viewer.toasts.show(ctx);
        egui::SidePanel::left("controls").show(ctx, |ui| {
            if ui.button("Save").clicked() {
                // Serialize the snarl data to a string using JSON
                let serialized = serde_json::to_string_pretty(&self.snarl).unwrap_or_else(|err| {
                    self.viewer
                        .add_error_notification(format!("Failed to serialize data: {}", err));
                    String::new()
                });

                // Save the serialized data to a file
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(".json")
                    .add_filter("JSON", &["json"])
                    .set_directory(&std::env::current_dir().unwrap().join("saved"))
                    .save_file()
                {
                    std::fs::write(&path, serialized).unwrap_or_else(|err| {
                        self.viewer
                            .add_error_notification(format!("Failed to save route: {}", err));
                    });
                }
            }
            if ui.button("Load").clicked() {
                // Load the serialized data from a file
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_directory(&std::env::current_dir().unwrap().join("saved"))
                    .pick_file()
                {
                    let serialized = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                        self.viewer
                            .add_error_notification(format!("Failed to read file: {}", err));
                        String::new()
                    });

                    // Deserialize the snarl data from the string
                    self.snarl = serde_json::from_str(&serialized).unwrap_or_else(|err| {
                        self.viewer.add_error_notification(format!(
                            "Failed to deserialize snarl: {}",
                            err
                        ));
                        Snarl::new()
                    });
                }
            }
        });

        egui::Window::new("Kalkulátor").show(ctx, |ui| {
            ui.label("Actions");
            if ui.button("Remove all").clicked() {
                for (node_id, _) in self.snarl.clone().nodes_ids_data() {
                    self.viewer.stored_nodes.remove(&node_id);
                    self.snarl.remove_node(node_id);
                }
                self.viewer.path_nodes.clear();
            }

            if ui.button("Clear Dijkstra Path").clicked() {
                self.viewer.path_nodes.clear();
            }

            if ui.button("Run Dijkstra Algorithm").clicked() {
                self.viewer.path_nodes.clear();
                match self.run_dijkstra() {
                    Ok(path) => {
                        self.viewer.path_nodes = path;
                        self.viewer.add_success_notification(format!(
                            "Path found! Total cost: {}",
                            self.total_cost
                        ));
                    }
                    Err(err) => {
                        self.viewer.add_error_notification(err);
                    }
                }
            }
            if ui
                .button(format!("Auto recalc - {}", self.auto_recalc))
                .clicked()
            {
                self.auto_recalc = !self.auto_recalc;
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl.show(&mut self.viewer, &self.style, "salty", ui);
        });

        if self.auto_recalc {
            self.viewer.path_nodes.clear();
            match self.run_dijkstra() {
                Ok(path) => {
                    self.viewer.path_nodes = path;
                }
                Err(_) => {}
            }
        }
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
