use egui::{emath::Numeric, Color32, Id, Label, RichText, Sense, Ui};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    Snarl,
};
use mapgen_core::mutations::{
    brush::{pulse::PulseBrushMutation, transition::TransitionBrushMutation},
    walker::{
        backwards::BackwardsWalkerMutation, left::LeftWalkerMutation, random::RandomWalkerMutation,
        right::RightWalkerMutation, straight::StraightWalkerMutation,
    },
};

use super::context::RenderableUi;

const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

#[derive(Debug, Clone, PartialEq)]
enum UiNode {
    StartNode,
    MutationNode(UiMutation),
    EndNode,
}

impl UiNode {
    fn title(&self) -> String {
        match self {
            UiNode::MutationNode(mutation) => mutation.title(),
            UiNode::StartNode => "Start".to_owned(),
            UiNode::EndNode => "End".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UiMutation {
    Brush(UiBrushMutation),
    Map(UiMapMutation),
    Walker(UiWalkerMutation),
}

impl UiMutation {
    fn title(&self) -> String {
        match self {
            UiMutation::Brush(mutation) => mutation.title(),
            UiMutation::Map(mutation) => mutation.title(),
            UiMutation::Walker(mutation) => mutation.title(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UiBrushMutation {
    Pulse(PulseBrushMutation),
    Transition(TransitionBrushMutation),
}

impl UiBrushMutation {
    fn title(&self) -> String {
        match self {
            UiBrushMutation::Pulse(_) => "Pulse".to_owned(),
            UiBrushMutation::Transition(_) => "Transition".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UiMapMutation {}

impl UiMapMutation {
    fn title(&self) -> String {
        unreachable!()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UiWalkerMutation {
    Straight(StraightWalkerMutation),
    Backwards(BackwardsWalkerMutation),
    Left(LeftWalkerMutation),
    Right(RightWalkerMutation),
    Random(RandomWalkerMutation),
}

impl UiWalkerMutation {
    fn title(&self) -> String {
        match self {
            UiWalkerMutation::Straight(_) => "Straight".to_owned(),
            UiWalkerMutation::Backwards(_) => "Backwards".to_owned(),
            UiWalkerMutation::Left(_) => "Left".to_owned(),
            UiWalkerMutation::Right(_) => "Right".to_owned(),
            UiWalkerMutation::Random(_) => "Random".to_owned(),
        }
    }
}

struct UiViewer;

impl SnarlViewer<UiNode> for UiViewer {
    fn title(&mut self, node: &UiNode) -> String {
        node.title()
    }

    fn outputs(&mut self, node: &UiNode) -> usize {
        match node {
            UiNode::EndNode => 0,
            _ => 1,
        }
    }

    fn inputs(&mut self, node: &UiNode) -> usize {
        match node {
            UiNode::StartNode => 0,
            _ => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<UiNode>,
    ) -> PinInfo {
        ui.label("Prev");
        PinInfo::circle().with_fill(UNTYPED_COLOR)
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<UiNode>,
    ) -> egui_snarl::ui::PinInfo {
        ui.label("Next");
        PinInfo::circle().with_fill(UNTYPED_COLOR)
    }

    fn has_body(&mut self, node: &UiNode) -> bool {
        true
    }

    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        inputs: &[egui_snarl::InPin],
        outputs: &[egui_snarl::OutPin],
        ui: &mut Ui,
        scale: f32,
        snarl: &mut Snarl<UiNode>,
    ) {
        let id = format!("{}_grid", snarl[node].title());

        match &mut snarl[node] {
            UiNode::StartNode => {}
            UiNode::MutationNode(mutation) => match mutation {
                UiMutation::Brush(mutation) => match mutation {
                    UiBrushMutation::Pulse(ref mut mutation) => {
                        egui::Grid::new(id).show(ui, |ui| {
                            field_numeric(ui, "BorderValue", &mut mutation.value_border);
                            field_numeric(ui, "ClimaxValue", &mut mutation.value_climax);
                            field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                        });
                    }
                    UiBrushMutation::Transition(ref mut mutation) => {
                        egui::Grid::new(id).show(ui, |ui| {
                            field_numeric(ui, "FromValue", &mut mutation.value_from);
                            field_numeric(ui, "ToValue", &mut mutation.value_to);
                            field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                        });
                    }
                },
                UiMutation::Map(mutation) => match mutation {
                    _ => {}
                },
                UiMutation::Walker(mutation) => match mutation {
                    UiWalkerMutation::Straight(ref mut mutation) => {
                        field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                    }
                    UiWalkerMutation::Backwards(ref mut mutation) => {
                        field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                    }
                    UiWalkerMutation::Left(ref mut mutation) => {
                        field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                    }
                    UiWalkerMutation::Right(ref mut mutation) => {
                        field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                    }
                    UiWalkerMutation::Random(ref mut mutation) => {
                        egui::Grid::new(id).show(ui, |ui| {
                            field_numeric(ui, "Seed", &mut mutation.seed);
                            field_numeric(ui, "OverallSteps", &mut mutation.overall_steps);
                        });
                    }
                },
            },
            UiNode::EndNode => {}
        }
    }

    fn input_color(
        &mut self,
        pin: &egui_snarl::InPin,
        style: &egui::Style,
        snarl: &mut Snarl<UiNode>,
    ) -> egui::Color32 {
        UNTYPED_COLOR
    }

    fn output_color(
        &mut self,
        pin: &egui_snarl::OutPin,
        style: &egui::Style,
        snarl: &mut Snarl<UiNode>,
    ) -> egui::Color32 {
        UNTYPED_COLOR
    }

    fn graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, _scale: f32, snarl: &mut Snarl<UiNode>) {
        // TODO: refactor it somehow lol

        const MARKER_TYPES: [&'static str; 2] = ["Start", "End"];
        const BRUSH_TYPES: [&'static str; 2] = ["Pulse", "Transition"];
        const MAP_TYPES: [&'static str; 0] = [];
        const WALKER_TYPES: [&'static str; 5] =
            ["Straight", "Backwards", "Left", "Right", "Random"];

        fn title(index: usize) -> &'static str {
            if index < MARKER_TYPES.len() {
                MARKER_TYPES[index]
            } else if index - MARKER_TYPES.len() < BRUSH_TYPES.len() {
                BRUSH_TYPES[index - MARKER_TYPES.len()]
            } else if (index - MARKER_TYPES.len() - BRUSH_TYPES.len()) < MAP_TYPES.len() {
                MAP_TYPES[index - MARKER_TYPES.len() - BRUSH_TYPES.len()]
            } else if (index - MARKER_TYPES.len() - BRUSH_TYPES.len() - MAP_TYPES.len())
                < WALKER_TYPES.len()
            {
                WALKER_TYPES[index - MARKER_TYPES.len() - BRUSH_TYPES.len() - MAP_TYPES.len()]
            } else {
                unreachable!()
            }
        }

        let mut selected = None;

        ui.label("Add Node");
        ui.separator();

        for i in 0..(MARKER_TYPES.len() + BRUSH_TYPES.len() + MAP_TYPES.len() + WALKER_TYPES.len())
        {
            if ui
                .add(
                    Label::new(RichText::new(title(i)).monospace())
                        .selectable(true)
                        .sense(Sense::click()),
                )
                .clicked()
            {
                selected = Some(i);
                ui.close_menu();
            }
        }

        // TODO: ugly
        if let Some(i) = selected {
            let node = match i {
                0 => UiNode::StartNode,
                1 => UiNode::EndNode,
                2 => UiNode::MutationNode(UiMutation::Brush(UiBrushMutation::Pulse(
                    Default::default(),
                ))),
                3 => UiNode::MutationNode(UiMutation::Brush(UiBrushMutation::Transition(
                    Default::default(),
                ))),

                4 => UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Straight(
                    Default::default(),
                ))),
                5 => UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Backwards(
                    Default::default(),
                ))),
                6 => UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Left(
                    Default::default(),
                ))),
                7 => UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Right(
                    Default::default(),
                ))),
                8 => UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Random(
                    Default::default(),
                ))),

                _ => unreachable!(),
            };

            snarl.insert_node(pos, node);
        }
    }

    #[inline]
    fn connect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<UiNode>,
    ) {
        match (&snarl[from.id.node], &snarl[to.id.node]) {
            (UiNode::StartNode, UiNode::EndNode) => {}
            (UiNode::StartNode, UiNode::MutationNode(_)) => {}
            (UiNode::MutationNode(_), UiNode::EndNode) => {}
            (
                UiNode::MutationNode(UiMutation::Brush(_)),
                UiNode::MutationNode(UiMutation::Brush(_)),
            ) => {}
            (
                UiNode::MutationNode(UiMutation::Map(_)),
                UiNode::MutationNode(UiMutation::Map(_)),
            ) => {}
            (
                UiNode::MutationNode(UiMutation::Walker(_)),
                UiNode::MutationNode(UiMutation::Walker(_)),
            ) => {}
            _ => return,
        }

        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        snarl.connect(from.id, to.id);
    }
}

pub struct BottomPanelUi {
    snarl: Snarl<UiNode>,
    style: SnarlStyle,
}

impl BottomPanelUi {
    pub fn new() -> Self {
        Self {
            snarl: Snarl::new(),
            style: SnarlStyle::new(),
        }
    }
}

impl RenderableUi for BottomPanelUi {
    fn ui_with(&mut self, ctx: &egui::Context) {
        egui::panel::TopBottomPanel::bottom("main_bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.snarl
                    .show(&mut UiViewer, &self.style, Id::new("node_graph"), ui);
            });
    }
}

fn field_numeric(ui: &mut Ui, name: impl Into<String>, value: &mut impl Numeric) {
    ui.label(name.into());
    ui.add(egui::DragValue::new(value));
    ui.end_row();
}
