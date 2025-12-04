use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages;
use common_game::protocols::messages::OrchestratorToPlanet::Sunray;
use common_game::protocols::messages::PlanetToOrchestrator::{
    AsteroidAck, InternalStateResponse, StartPlanetAIResult, StopPlanetAIResult,
};
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::os::linux::raw::stat;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::time::SystemTime;

// Group-defined AI struct
pub struct AI {
    is_on: bool,
}

impl PlanetAI for AI {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: messages::OrchestratorToPlanet,
    ) -> Option<messages::PlanetToOrchestrator> {
        // match on msg type
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                // state
                let cell_opt = state.cells_iter_mut().find(|c| !c.is_charged());
                match cell_opt {
                    None => {
                        // all charged
                    }
                    Some(cell) => {
                        cell.charge(sunray);
                    }
                }

                //send ack
                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::Asteroid(_) => {
                //send the ack
                Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket: None,
                })
            }
            OrchestratorToPlanet::StartPlanetAI(_) => {
                self.is_on = true;
                Some(OrchestratorToPlanet::SrartPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::StopPlanetAI(_) => {
                self.is_on = false;
                Some(OrchestratorToPlanet::StopPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest(_) => {
                Some(OrchestratorToPlanet::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: Arc::clone(state),
                    timestamp: SystemTime::now(),
                })
            }
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: messages::ExplorerToPlanet,
    ) -> Option<messages::PlanetToExplorer> {
        // your handler code here...
        None
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        // your handler code here...
        None
    }

    fn start(&mut self, state: &PlanetState) { /* startup code */
    }
    fn stop(&mut self, state: &PlanetState) { /* stop code */
    }
}

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    rx_orchestrator: mpsc::Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<messages::PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<messages::ExplorerToPlanet>,
    tx_explorer: mpsc::Sender<messages::PlanetToExplorer>,
) -> Planet<AI> {
    let id = 1;
    let ai = AI { is_on: true };
    let gen_rules = vec![/* your recipes */];
    let comb_rules = vec![/* your recipes */];

    // Construct the planet and return it
    let planet = Planet::new(
        id,
        PlanetType::A,
        ai,
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        (rx_explorer, tx_explorer),
    )
    .unwrap(); // Don't call .unwrap()! You should do error checking instead.
    planet
}
