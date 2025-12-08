use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};
use common_game::protocols::messages;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::{Receiver, Sender};

// AI struct
struct BlackAdidasShoe{
    is_on: bool,
}

impl BlackAdidasShoe{
    pub fn new(is_on: bool)->Self{
        Self{
            is_on
        }
    }
}
fn exit_on_stopped_ai(is_on: bool, planet_id: u32)->bool{
    if !is_on{
        LogEvent::new(
            ActorType::Planet,
            planet_id,
            ActorType::Orchestrator,
            String::from("1"),
            EventType::MessageOrchestratorToPlanet,
            Channel::Error,
            Payload::from([("AI disabled".to_string(), "AI field `is_on` is false".to_string())])
        ).emit();
        true
    }
    else { false }
}
fn log_not_created_resource(err: String, planet_id: u32, explorer_id: u32){
    LogEvent::new(
        ActorType::Planet,
        planet_id,
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessageExplorerToPlanet,
        Channel::Error,
        Payload::from([("Cannot make resource".to_string(), err)])
    ).emit();
}
impl PlanetAI for BlackAdidasShoe {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        // check on the AI state
        if exit_on_stopped_ai(self.is_on, state.id()){
            return None
        }

        // match on msg type
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                // state

                if let Some(_) = state.charge_cell(sunray){
                    // cell charged
                    LogEvent::new(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Orchestrator,
                        String::from("1"),
                        EventType::MessageOrchestratorToPlanet,
                        Channel::Info,
                        Payload::from([("Cell charged".to_string(), "Cell recharged correctly".to_string())])
                    ).emit();
                }else{
                    // not charged, full cells
                    LogEvent::new(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Orchestrator,
                        String::from("1"),
                        EventType::MessageOrchestratorToPlanet,
                        Channel::Warning,
                        Payload::from([("Not able to charge cell".to_string(), "All cells are already charged".to_string())])
                    ).emit();
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
        if exit_on_stopped_ai(self.is_on, state.id()){
            return None
        }

        //match on the message type
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                // no combination
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
                                Ok(o) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Oxygen(o)),
                                }),
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
                                Ok(h) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Hydrogen(h)),
                                }),
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
                                Ok(c) => Some(PlanetToExplorer::GenerateResourceResponse {
                                    resource: Some(BasicResource::Carbon(c)),
                                }),
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

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        state.take_rocket()
    }

    fn start(&mut self, _state: &PlanetState) {
        self.is_on = true;
    }
    fn stop(&mut self, _state: &PlanetState) {
        self.is_on = false;
    }
}

// This is the group's "export" function. It will be called by
// the orchestrator to spawn your planet.
pub fn create_planet(
    rx_orchestrator: Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: Sender<messages::PlanetToOrchestrator>,
    rx_explorer: Receiver<messages::ExplorerToPlanet>,
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
