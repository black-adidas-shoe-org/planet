use std::os::linux::raw::stat;

use common_game::components::planet::{DummyPlanetState, Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};

// AI struct
struct BlackAdidasShoe{
    is_on: bool,
}

const ORCH_ID:u32 = 1u32; // to be moved in orch
impl BlackAdidasShoe{
    pub fn new(is_on: bool)->Self{
        Self{
            is_on
        }
    }
}
fn exit_on_stopped_ai(is_on: bool, planet_id: u32)->bool{
    if !is_on{
        log_ai_state(String::from("AI field `is_on` is false"), planet_id);
        true
    }
    else { false }
}




// Begin of Log functions
fn log_ai_state(msg: String, planet_id: u32) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("AI state".to_string(), msg)])
    ).emit();
}


fn log_explorer_transit(msg: String, planet_id: u32) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Explorer Transit".to_string(), msg)])
    ).emit();
}

fn log_resource_created(msg: String, planet_id: u32, explorer_id: u32){
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessageExplorerToPlanet,
        Channel::Error,
        Payload::from([("Resource created".to_string(), msg)])
    ).emit();
}

fn log_not_created_resource(err: String, planet_id: u32, explorer_id: u32){
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessageExplorerToPlanet,
        Channel::Error,
        Payload::from([("Cannot make resource".to_string(), err)])
    ).emit();
}
fn log_internal_state(msg: String, planet_id: u32) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Internal state".to_string(), msg)])
    ).emit();
}
fn log_asteroid_impact(msg: String, planet_id: u32) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Asteroid Impact".to_string(), msg)])
    ).emit();
}

fn log_cell_charge(msg: String, planet_id:u32){
    LogEvent::new(
        Some(Participant::new(ActorType::Planet,planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessageOrchestratorToPlanet,
        Channel::Info,
        Payload::from([("Cell charge".to_string(), msg)])
    ).emit();
}
fn log_supported_resources(msg: String, planet_id:u32, explorer_id: u32){
    LogEvent::new(
        Some(Participant::new(ActorType::Planet,planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessagePlanetToExplorer,
        Channel::Info,
        Payload::from([("Supported resources".to_string(), msg)])
    ).emit();
}
fn log_generation_rules(msg: String, planet_id:u32, explorer_id: u32){
    LogEvent::new(
        Some(Participant::new(ActorType::Planet,planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessagePlanetToExplorer,
        Channel::Info,
        Payload::from([("Generation rules".to_string(), msg)])
    ).emit();
}




// End of Log functions

impl PlanetAI for BlackAdidasShoe {
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray
    ){
        if let Some(_) = state.charge_cell(sunray){
            // cell charged
            log_cell_charge(String::from("Cell recharged correctly"), state.id());
        }else{
            // not charged, full cells
            log_cell_charge(String::from("All cells are already charged"), state.id());
        }
    }


    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        log_asteroid_impact(String::from("No rocket usable to defend"), state.id());
        state.take_rocket()
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator
    ) -> DummyPlanetState {
        log_internal_state(String::from("Internal state"), state.id());
        state.to_dummy()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet
    ) -> Option<PlanetToExplorer> {
        // check on the AI state
        if exit_on_stopped_ai(self.is_on, state.id()){ // already logging into this function
            return None
        }

        //match on the message type
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                log_supported_resources(String::from("Supported resources: Carbon, Oxygen, Hydrogen"), state.id(), explorer_id);
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                // no combination
                log_generation_rules(String::from("No supported combination"), state.id(), explorer_id);
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => {
                match state.full_cell() {
                    Some((cell, _)) => match resource {
                        BasicResourceType::Silicon => {
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                        BasicResourceType::Oxygen => {
                            // generate the oxygen resource
                            match generator.make_oxygen(cell) {
                                Ok(o) => {
                                    log_resource_created(String::from("Oxygen created"), state.id(), explorer_id);
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: Some(BasicResource::Oxygen(o)),
                                    })
                                },
                                Err(err) => {
                                    log_not_created_resource(err, state.id(), explorer_id);
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: None,
                                    })
                                }
                            }
                        }
                        BasicResourceType::Hydrogen => {
                            match generator.make_hydrogen(cell) {
                                Ok(h) => {
                                    log_resource_created(String::from("Hydrogen created"), state.id(), explorer_id);
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: Some(BasicResource::Hydrogen(h)),
                                    })
                                },
                                Err(err) => {
                                    log_not_created_resource(err, state.id(), explorer_id);
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: None,
                                    })
                                }
                            }
                        }
                        BasicResourceType::Carbon => {
                            match generator.make_carbon(cell) {
                                Ok(c) => {
                                    log_resource_created(String::from("Carbon created"), state.id(), explorer_id);
                                    Some(PlanetToExplorer::GenerateResourceResponse {
                                        resource: Some(BasicResource::Carbon(c)),
                                    })
                                },
                                Err(err) => {
                                    log_not_created_resource(err, state.id(), explorer_id);
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
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
                let mut charged_cells: u32 = 0;
                state.cells_iter().for_each(|cell|
                    if cell.is_charged() { charged_cells += 1}
                );
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: charged_cells,
                })
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id: _, msg } => {
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

    fn on_explorer_arrival(&mut self, state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator, _explorer_id: ID) {
        log_explorer_transit(String::from("Explorer arrived"), state.id());
        //TODO do we need to do something?
    }

    fn on_explorer_departure(&mut self, state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator, _explorer_id: ID) {
        log_explorer_transit(String::from("Explorer left"), state.id());
        //TODO do we need to do something?
    }

    fn on_start(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        log_ai_state(String::from("AI enabled"), state.id());
        self.is_on = true;
    }

    fn on_stop(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        log_ai_state(String::from("AI disabled"), state.id());
        self.is_on = false;
    }
}

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32
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
