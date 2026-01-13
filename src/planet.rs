use crate::ai::BlackAdidasShoe;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::BasicResourceType;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: ID
) -> Result<Planet, String> {
    let ai = BlackAdidasShoe::new(false);
    let gen_rules = vec![
        BasicResourceType::Oxygen,
        BasicResourceType::Hydrogen,
        BasicResourceType::Carbon,
    ];
    let comb_rules = vec![];

    // Construct the planet and return it
    let planet = Planet::new(
        planet_id,
        PlanetType::D,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );
    planet
}
