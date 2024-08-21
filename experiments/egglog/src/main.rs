use egglog::*;

const LIB: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lib.egg"));

fn main() {
    let mut egraph = EGraph::default();
    egraph.run_mode = RunMode::ShowDesugaredEgglog;
    egraph.set_underscores_for_desugaring(3);
    let desugared_str = egraph.parse_and_run_program(&LIB).unwrap().join("\n");

    println!("{}", desugared_str);
}
