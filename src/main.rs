#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, App, CreationContext, NativeOptions};
use egui_snarl::{
    ui::{BackgroundPattern, Grid, PinInfo, SnarlPin, SnarlStyle, SnarlViewer, WireStyle}, InPin, OutPin, Snarl
};

// Define a simple node type
#[derive(PartialEq, Clone, serde::Serialize, serde::Deserialize)]
enum DijkstraNode {
    Start,
    Value(u32),
    Finish,
}

// Implement the SnarlViewer trait to define how nodes are displayed and connected
struct DijkstraViewer;

impl SnarlViewer<DijkstraNode> for DijkstraViewer {
    fn title(&mut self, node: &DijkstraNode) -> String {
        match node {
            DijkstraNode::Start => "Start".to_string(),
            DijkstraNode::Value(_) => "Value".to_string(),
            DijkstraNode::Finish => "Finish".to_string(),
        }
    }

    fn inputs(&mut self, node: &DijkstraNode) -> usize {
        match node {
            DijkstraNode::Start => 0,
            DijkstraNode::Value(_) => 1,
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
                    match snarl[remote.node] {
                        DijkstraNode::Value(value) => {
                            ui.label(format!("Value: {}", value));
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
            DijkstraNode::Value(_) => 1,
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
        match snarl[pin.id.node] {
            DijkstraNode::Value(mut value) => {
                ui.add(egui::DragValue::new(&mut value));
                snarl[pin.id.node] = DijkstraNode::Value(value); // Update the value in the snarl
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
        if ui.button("Start").clicked() {
            snarl.insert_node(pos, DijkstraNode::Start);
            ui.close_menu();
        }
        if ui.button("Value").clicked() {
            snarl.insert_node(pos, DijkstraNode::Value(1));
            ui.close_menu();
        }
        if ui.button("Finish").clicked() {
            snarl.insert_node(pos, DijkstraNode::Finish);
            ui.close_menu();
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<DijkstraNode>) {
        if let (DijkstraNode::Start, DijkstraNode::Value(_)) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
        if let (DijkstraNode::Value(_), DijkstraNode::Value(_)) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
        if let (DijkstraNode::Value(_), DijkstraNode::Finish) =
            (&snarl[from.id.node], &snarl[to.id.node])
        {
            snarl.connect(from.id, to.id);
        }
    }
}

// Implement the eframe::App trait
struct MyApp {
    snarl: Snarl<DijkstraNode>,
    style: SnarlStyle,
}

impl MyApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let mut ss = SnarlStyle::new();
        ss.pin_placement = Some(egui_snarl::ui::PinPlacement::Edge);
        ss.wire_style = Some(WireStyle::Line);
        ss.max_scale = Some(1.0);
        ss.bg_pattern = Some(BackgroundPattern::Grid(Grid::new(egui::vec2(30.0, 30.0), 0.0)));
        MyApp {
            snarl: Snarl::new(),
            style: ss,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.label("Add node");
            if ui.button("Start").clicked() && self.snarl.nodes().all(|node| *node != DijkstraNode::Start) {
                self.snarl.insert_node(egui::Pos2::new(0.0, 0.0), DijkstraNode::Start);
            }
        });
        egui::Window::new("Kalkulátor").show(ctx, |ui| {
            ui.label("Összeadás!");
            if ui.button("Press me").clicked() {
                for node in self.snarl.nodes() {
                    match node {
                        DijkstraNode::Value(value) => {
                            println!("Value: {}", value);
                        }
                        _ => {}
                    }
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl
                .show(&mut DijkstraViewer, &self.style, "salty", ui);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Visualize dijkstra's algorithm",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
