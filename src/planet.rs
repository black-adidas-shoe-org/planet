use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::collections::HashSet;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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
        // check on the AI state
        if !self.is_on{
            return None
        }

        // match on msg type
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                // state

                if let Some(_) = state.charge_cell(sunray){
                    // cell charged
                }else{
                    // not charged, full cells
                }

                //send ack
                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest => {
                Some(PlanetToOrchestrator::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: state.to_dummy(),
                })
            }
            _ => None,
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: messages::ExplorerToPlanet,
    ) -> Option<messages::PlanetToExplorer> {
        // check on the AI state
        if !self.is_on{
            return None
        }

        //match on the message type
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                // no combination
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                resource, ..
            } => {
                match state.full_cell() {
                    Some((cell, _)) => match resource {
                        BasicResourceType::Silicon => {
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                        BasicResourceType::Oxygen => {
                            // generate the oxygen resource
                            match generator.make_oxygen(cell) {
                                Ok(o) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Oxygen(o)),
                                }),
                                Err(err) => {
                                    // TODO log the error
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: None,
                                    })
                                }
                            }
                        }
                        BasicResourceType::Hydrogen => {
                            match generator.make_hydrogen(cell) {
                                Ok(h) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Hydrogen(h)),
                                }),
                                Err(err) => {
                                    // TODO log the error
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: None,
                                    })
                                }
                            }
                        }
                        BasicResourceType::Carbon => {
                            match generator.make_carbon(cell) {
                                Ok(c) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Carbon(c)),
                                }),
                                Err(err) => {
                                    // TODO log the error
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: None,
                                    })
                                }
                            }
                        }
                    },
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                }
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                let basic = |res| GenericResource::BasicResources(res);
                let complex = |res| GenericResource::ComplexResources(res);

                // retrieve the explorer's generic resource
                let (res1, res2) = match msg {
                    ComplexResourceRequest::Water(h, o) => (basic(BasicResource::Hydrogen(h)), basic(BasicResource::Oxygen(o))),
                    ComplexResourceRequest::Diamond(c1, c2) => (basic(BasicResource::Carbon(c1)), basic(BasicResource::Carbon(c2))),
                    ComplexResourceRequest::Life(w, c) => (complex(ComplexResource::Water(w)), basic(BasicResource::Carbon(c))),
                    ComplexResourceRequest::Robot(s, l) => (basic(BasicResource::Silicon(s)), complex(ComplexResource::Life(l))),
                    ComplexResourceRequest::Dolphin(w, l) => (complex(ComplexResource::Water(w)), complex(ComplexResource::Life(l))),
                    ComplexResourceRequest::AIPartner(r, d) => (complex(ComplexResource::Robot(r)), complex(ComplexResource::Diamond(d))),
                };

                // send them back
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((String::from("Not supported"), res1, res2)),
                })
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        state.take_rocket()
    }

    fn start(&mut self, state: &PlanetState) {
        self.is_on = true;
    }
    fn stop(&mut self, state: &PlanetState) {
        self.is_on = false;
    }
}

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    id: u32,
    rx_orchestrator: mpsc::Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<messages::PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<messages::ExplorerToPlanet>,
) -> Result<Planet, String> {
    let ai = AI {
        is_on: false,
    };
    let gen_rules = vec![
        BasicResourceType::Oxygen,
        BasicResourceType::Hydrogen,
        BasicResourceType::Carbon,
    ];
    let comb_rules = vec![];

    // Construct the planet and return it
    let planet = Planet::new(
        id,
        PlanetType::D,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );

    planet
}
