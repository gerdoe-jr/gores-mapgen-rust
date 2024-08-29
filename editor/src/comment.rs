use std::fs::File;

use mapgen_core::{
    brush::Brush,
    generator::Generator,
    map::Map,
    mutations::{
        brush::{pulse::PulseBrushMutation, transition::TransitionBrushMutation},
        walker::straight::StraightWalkerMutation,
        MutationState, Mutator,
    },
    walker::Walker,
};

fn main() {
    let mut generator = Generator::new(200.0);

    let mut current_mutation = 0;
    let mut brush_mutations: Vec<Mutation<Brush>> = vec![
        Mutation::new(TransitionBrushMutation::new(1, 20, 200)),
        Mutation::new(PulseBrushMutation::new(20, 50, 200, 0.5)),
    ];

    let on_step = move |walker: &mut Walker, _map: &mut Map, brush: &mut Brush| {
        if current_mutation != brush_mutations.len() {
            brush_mutations[current_mutation].mutate(brush);

            current_mutation += brush_mutations[current_mutation].is_finished() as usize;
        }
        StraightWalkerMutation::new().mutate(walker);
    };

    generator.on_step(on_step);

    let mut map = generator.generate(vec![
        (0.0, 1.0),
        (0.2, 0.8),
        (0.4, 0.6),
        (0.6, 0.4),
        (0.8, 0.2),
        (1.0, 0.0),
    ]);

    let mut file = File::create("./out.map").unwrap();
    map.save(&mut file).unwrap();
}
