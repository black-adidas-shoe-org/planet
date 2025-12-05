use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, Generator, Hydrogen, Oxygen, Silicon};
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
use std::sync::{Arc, mpsc};
use std::sync::mpsc::channel;
use std::time::SystemTime;

// Group-defined AI struct
pub struct AI {
    is_on: bool,
    is_alive: bool
}

impl AI{

    fn run(
    ){
        /*
        PLANET AI PURPOSE
        Read messages from each channel, and call the handle_msg method each time that
        a message arrives, and stop this behavior when it get killed.
         */


    }
}

//IMPORTANT:
/*
TODO
We need to check simultaneously 2 channels, so we should generate 2 threads for each fifo.
This leads us to implement mutex for the state
 */
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
                Some(PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::StopPlanetAI(_) => {
                self.is_on = false;
                Some(PlanetToOrchestrator::StopPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest(_) => {
                //Some(PlanetToOrchestrator::InternalStateResponse {   
                //    planet_id: state.id(),
                //    planet_state: std::sync::Arc::new(state),          // IMPOSSIBILE passare lo state, non implementa il Copy trait
                //    timestamp: SystemTime::now(),
                //})
                None
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
        match msg {
            SupportedResourceRequest =>{ 
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: Some(generator.all_available_recipes()) })
            },
            SupportedCombinationRequest =>{ 
                // no combination
                None
            },
            ExplorerToPlanet::GenerateResourceRequest {explorer_id, resource} =>{ 
                let cell = (state.cells_iter_mut().find(|c| c.is_charged()));

                match cell {
                    Some(c) => {
                        match resource {
                            BasicResourceType::Silicon=> None,
                            BasicResourceType::Oxygen => {
                                let r = generator.make_oxygen(c);
                                match r {
                                    Ok(o) => Some(PlanetToExplorer::GenerateResourceResponse { resource: Some(BasicResource::Oxygen(o))}),
                                    Err(_) => None,
                                }
                            },
                            BasicResourceType::Hydrogen=> {
                                let r = generator.make_hydrogen(c);
                                match r {
                                    Ok(o) => Some(PlanetToExplorer::GenerateResourceResponse { resource: Some(BasicResource::Hydrogen(o))}),
                                    Err(_) => None,
                                }
                            },
                            BasicResourceType::Carbon=> {
                                let r = generator.make_carbon(c);
                                match r {
                                    Ok(o) => Some(PlanetToExplorer::GenerateResourceResponse { resource: Some(BasicResource::Carbon(o))}),
                                    Err(_) => None,
                                }
                            }
                        }
                    },
                    None => None,
                }

            },
            AvailableEnergyCellRequest =>{ 
                let mut cells:u32 = 0 ;
                state.cells_iter().for_each(|c|  {if c.is_charged() { cells += 1 }});

                Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: cells })
                
            },
            CombineResourceRequest =>{
                // no combination
                None
            },
            InternalStateRequest =>{ 
                // Some(messages::PlanetToExplorer::InternalStateResponse  { planet_state: state })
                None
            }
        }   
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        // I think this should be enough
        state.take_rocket()
    }

    fn start(&mut self, state: &PlanetState) { 
        if !self.is_alive{
            //the planet has been destroyed
            return;
        }
        /* startup code */






        self.is_on = true;

    }
    fn stop(&mut self, state: &PlanetState) {
        if !self.is_alive{
            //the planet has been destroyed
            return;
        }

        /* stop code */
        self.is_on = false;
    }
}

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    rx_orchestrator: mpsc::Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<messages::PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<messages::ExplorerToPlanet>,
    tx_explorer: mpsc::Sender<messages::PlanetToExplorer>,
) -> Result<Planet<AI>, String> {
    let id = 104;
    let ai = AI { is_on: false, is_alive: true};
    let gen_rules = vec![BasicResourceType::Oxygen, BasicResourceType::Hydrogen, BasicResourceType::Carbon];
    let comb_rules = vec![];

    // Construct the planet and return it
    let planet = Planet::new(
        id,
        PlanetType::D,
        ai,
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        (rx_explorer, tx_explorer),
    );

    planet
}
