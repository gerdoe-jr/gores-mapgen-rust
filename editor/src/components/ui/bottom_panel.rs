use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc};

use egui::{emath::Numeric, Color32, Id, Label, RichText, Sense, Ui};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    Snarl,
};
use mapgen_core::{
    brush::Brush,
    map::Map,
    mutations::{
        brush::{pulse::PulseBrushMutation, transition::TransitionBrushMutation},
        walker::{
            backwards::BackwardsWalkerMutation, left::LeftWalkerMutation,
            random::RandomWalkerMutation, right::RightWalkerMutation,
            straight::StraightWalkerMutation,
        },
        Mutator,
    },
    walker::Walker,
};

use crate::components::utils::generation::{
    DesignImageInfo, DesignInfo, DesignLayer, GenerationContext,
};

use super::context::RenderableUi;

const UNTYPED_COLOR: Color32 = Color32::from_rgb(0xb0, 0xb0, 0xb0);

#[derive(Debug, Clone, PartialEq)]
pub enum UiNode {
    GeneratorNode,
    MutationNode(UiMutation),
    LoopStartNode(Option<usize>),
    LoopEndNode,
}

impl Titled for UiNode {
    fn title(&self) -> &'static str {
        match self {
            UiNode::GeneratorNode => "Generator",
            UiNode::MutationNode(mutation) => mutation.title(),
            UiNode::LoopStartNode(_) => "LoopStart",
            UiNode::LoopEndNode => "LoopEnd"
        }
    }
}

impl UiNode {
    // TODO: it's less ugly, but maybe there's something better
    fn default_all_variants() -> Vec<UiNode> {
        vec![
            UiNode::GeneratorNode,
            UiNode::MutationNode(UiMutation::Brush(
                UiBrushMutation::Pulse(Default::default()),
            )),
            UiNode::MutationNode(UiMutation::Brush(UiBrushMutation::Transition(
                Default::default(),
            ))),
            UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Straight(
                Default::default(),
            ))),
            UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Backwards(
                Default::default(),
            ))),
            UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Left(
                Default::default(),
            ))),
            UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Right(
                Default::default(),
            ))),
            UiNode::MutationNode(UiMutation::Walker(UiWalkerMutation::Random(
                Default::default(),
            ))),
            UiNode::LoopStartNode(None),
            UiNode::LoopEndNode
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiMutation {
    Brush(UiBrushMutation),
    Map(UiMapMutation),
    Walker(UiWalkerMutation),
}

impl UiMutation {
    fn title(&self) -> &'static str {
        match self {
            UiMutation::Brush(mutation) => mutation.title(),
            UiMutation::Map(mutation) => mutation.title(),
            UiMutation::Walker(mutation) => mutation.title(),
        }
    }
}

pub trait ExtractMutation<GivenType> {
    type ExtractType: ExtractMutation<GivenType> + Titled;
    const INPUT: usize = 0;

    fn extract(&self) -> Option<Self::ExtractType> {
        None
    }
}

impl ExtractMutation<Brush> for UiMutation {
    type ExtractType = UiBrushMutation;

    fn extract(&self) -> Option<Self::ExtractType> {
        match self {
            UiMutation::Brush(mutation) => Some(mutation.clone()),
            _ => None,
        }
    }

    const INPUT: usize = 0;
}

impl ExtractMutation<Map> for UiMutation {
    type ExtractType = UiMapMutation;

    fn extract(&self) -> Option<Self::ExtractType> {
        match self {
            UiMutation::Map(mutation) => Some(mutation.clone()),
            _ => None,
        }
    }

    const INPUT: usize = 1;
}

impl ExtractMutation<Walker> for UiMutation {
    type ExtractType = UiWalkerMutation;

    fn extract(&self) -> Option<Self::ExtractType> {
        match self {
            UiMutation::Walker(mutation) => Some(mutation.clone()),
            _ => None,
        }
    }

    const INPUT: usize = 2;
}

impl ExtractMutation<Brush> for UiBrushMutation {
    type ExtractType = Box<dyn Mutator<Brush>>;

    fn extract(&self) -> Option<Self::ExtractType> {
        Some(match self {
            UiBrushMutation::Pulse(mutation) => Box::new(mutation.clone()),
            UiBrushMutation::Transition(mutation) => Box::new(mutation.clone()),
        })
    }
}

impl ExtractMutation<Map> for UiMapMutation {
    type ExtractType = Box<dyn Mutator<Map>>;

    fn extract(&self) -> Option<Self::ExtractType> {
        None
    }
}

impl ExtractMutation<Walker> for UiWalkerMutation {
    type ExtractType = Box<dyn Mutator<Walker>>;

    fn extract(&self) -> Option<Self::ExtractType> {
        Some(match self {
            UiWalkerMutation::Straight(mutation) => {
                println!("REAL: {}", mutation.overall_steps);
                Box::new(mutation.clone())
            }
            UiWalkerMutation::Backwards(mutation) => Box::new(mutation.clone()),
            UiWalkerMutation::Left(mutation) => Box::new(mutation.clone()),
            UiWalkerMutation::Right(mutation) => Box::new(mutation.clone()),
            UiWalkerMutation::Random(mutation) => Box::new(mutation.clone()),
        })
    }
}

impl<M> ExtractMutation<M> for Box<dyn Mutator<M>> {
    type ExtractType = ();
}

impl<M> ExtractMutation<M> for () {
    type ExtractType = ();
}

impl Titled for () {
    fn title(&self) -> &'static str {
        ""
    }
}

impl<T> Titled for Box<dyn Mutator<T>> {
    fn title(&self) -> &'static str {
        "Mutator"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiBrushMutation {
    Pulse(PulseBrushMutation),
    Transition(TransitionBrushMutation),
}

impl Titled for UiBrushMutation {
    fn title(&self) -> &'static str {
        match self {
            UiBrushMutation::Pulse(_) => "Pulse",
            UiBrushMutation::Transition(_) => "Transition",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiMapMutation {}

impl Titled for UiMapMutation {
    fn title(&self) -> &'static str {
        unreachable!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiWalkerMutation {
    Straight(StraightWalkerMutation),
    Backwards(BackwardsWalkerMutation),
    Left(LeftWalkerMutation),
    Right(RightWalkerMutation),
    Random(RandomWalkerMutation),
}

impl Titled for UiWalkerMutation {
    fn title(&self) -> &'static str {
        match self {
            UiWalkerMutation::Straight(_) => "Straight",
            UiWalkerMutation::Backwards(_) => "Backwards",
            UiWalkerMutation::Left(_) => "Left",
            UiWalkerMutation::Right(_) => "Right",
            UiWalkerMutation::Random(_) => "Random",
        }
    }
}

pub trait Titled {
    fn title(&self) -> &'static str;
}

struct UiViewer {
    generation: Rc<RefCell<GenerationContext>>,
}

impl SnarlViewer<UiNode> for UiViewer {
    fn title(&mut self, node: &UiNode) -> String {
        node.title().to_owned()
    }

    fn outputs(&mut self, node: &UiNode) -> usize {
        match node {
            UiNode::GeneratorNode => 0,
            UiNode::MutationNode(_) => 1,
            UiNode::LoopStartNode(_)
            | UiNode::LoopEndNode => 1
        }
    }

    fn inputs(&mut self, node: &UiNode) -> usize {
        match node {
            UiNode::GeneratorNode => 3,
            UiNode::MutationNode(_) => 1,
            UiNode::LoopStartNode(_)
            | UiNode::LoopEndNode => 1
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
            UiNode::GeneratorNode => {
                if ui.button("Proceed").clicked() {
                    let mut image_infos = HashMap::new();

                    image_infos.insert(
                        DesignLayer::Freeze,
                        DesignImageInfo::new("data/mapres/entities.png", 1),
                    );
                    image_infos.insert(
                        DesignLayer::Hookable,
                        DesignImageInfo::new("data/mapres/jungle_main.png", 2),
                    );
                    image_infos.insert(
                        DesignLayer::Unhookable,
                        DesignImageInfo::new("data/mapres/entities.png", 3),
                    );

                    let design = DesignInfo::new(image_infos);
                    self.generation.borrow_mut().set_scale_factor(200.0);
                    self.generation.borrow_mut().generate(
                        snarl,
                        node,
                        &design,
                        vec![
                            (0.0, 1.0),
                            (0.2, 0.8),
                            (0.4, 0.6),
                            (0.6, 0.4),
                            (0.8, 0.2),
                            (1.0, 0.0),
                        ],
                    );
                }
            }
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
            UiNode::LoopStartNode(count) => {
                if ui.button("Toggle endless").clicked() {
                    match count {
                        Some(_) => *count = None,
                        None => *count = Some(1),
                    }
                }
                if let Some(count) = count {
                    field_numeric(ui, "CountValue", count);
                }
            }
            UiNode::LoopEndNode => {}
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
        let all_variants = UiNode::default_all_variants();

        let mut selected = None;

        ui.label("Add Node");
        ui.separator();

        for i in 0..all_variants.len() {
            if ui
                .add(
                    Label::new(RichText::new(all_variants[i].title()).monospace())
                        .sense(Sense::click()),
                )
                .clicked()
            {
                selected = Some(i);
                ui.close_menu();
            }
        }

        if let Some(i) = selected {
            let node = all_variants[i].clone();

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
            (UiNode::MutationNode(mutation), UiNode::GeneratorNode) => {
                let eh_stop_it;

                match mutation {
                    UiMutation::Brush(_) => {
                        eh_stop_it = to.id.input == <UiMutation as ExtractMutation<Brush>>::INPUT
                    }
                    UiMutation::Map(_) => {
                        eh_stop_it = to.id.input == <UiMutation as ExtractMutation<Map>>::INPUT
                    }
                    UiMutation::Walker(_) => {
                        eh_stop_it = to.id.input == <UiMutation as ExtractMutation<Walker>>::INPUT
                    }
                }

                if !eh_stop_it {
                    return;
                }
            }
            (UiNode::GeneratorNode, UiNode::MutationNode(mutation)) => {
                let eh_stop_it;

                match mutation {
                    UiMutation::Brush(_) => {
                        eh_stop_it = from.id.output == <UiMutation as ExtractMutation<Brush>>::INPUT
                    }
                    UiMutation::Map(_) => {
                        eh_stop_it = from.id.output == <UiMutation as ExtractMutation<Map>>::INPUT
                    }
                    UiMutation::Walker(_) => {
                        eh_stop_it =
                            from.id.output == <UiMutation as ExtractMutation<Walker>>::INPUT
                    }
                }

                if eh_stop_it {
                    return;
                }
            }
            (UiNode::LoopStartNode(_) | UiNode::LoopEndNode, UiNode::MutationNode(_)) => {},
            (UiNode::LoopStartNode(_) | UiNode::LoopEndNode, UiNode::GeneratorNode) => {}
            (UiNode::MutationNode(_), UiNode::LoopStartNode(_) | UiNode::LoopEndNode) => {}
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
    viewer: UiViewer,
}

impl BottomPanelUi {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();

        snarl.insert_node(
            egui::pos2(-190.0, 0.0),
            UiNode::MutationNode(UiMutation::Brush(UiBrushMutation::Pulse(
                PulseBrushMutation::new(1, 20, 200, 0.5),
            ))),
        );
        snarl.insert_node(egui::pos2(240.0, 0.0), UiNode::GeneratorNode);

        Self {
            snarl,
            style: SnarlStyle::new(),
            viewer: UiViewer {
                generation: Rc::new(RefCell::new(GenerationContext::new())),
            },
        }
    }

    pub fn get_generation_handle(&self) -> Rc<RefCell<GenerationContext>> {
        self.viewer.generation.clone()
    }
}

impl RenderableUi for BottomPanelUi {
    fn ui_with(&mut self, ctx: &egui::Context) {
        egui::panel::TopBottomPanel::bottom("main_bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.snarl
                    .show(&mut self.viewer, &self.style, Id::new("node_graph"), ui);
            });
    }
}

fn field_numeric(ui: &mut Ui, name: impl Into<String>, value: &mut impl Numeric) {
    let drag_value = egui::DragValue::new(value);
    ui.label(name.into());
    ui.add(drag_value);
    ui.end_row();
}
