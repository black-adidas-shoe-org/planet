use common_game::components::planet::{PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, Generator};
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
use std::sync::mpsc;

use super::AI;

struct Environement {
    ai_state: bool,
    planet_state: PlanetState,
    generator: Generator,
    combinator: Combinator,
    etp: mpsc::Receiver<ExplorerToPlanet>,
    pte: mpsc::Sender<PlanetToExplorer>,
    otp: mpsc::Receiver<OrchestratorToPlanet>,
    pto: mpsc::Sender<PlanetToOrchestrator>
}

impl Environement {
    fn new(ai_state: bool) -> Environement {
        Self {
            ai_state: ai_state,
            planet_state: PlanetState::new(1, PlanetType::),
            generator: Generator::new(),
            combinator: Combinator::new(),
            etp: mpsc::channel().1,
            pte: mpsc::channel().0,
            otp: mpsc::channel().1,
            pto: mpsc::channel().0
        }
    }
}