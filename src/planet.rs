use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::protocols::messages;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;

// Group-defined AI struct
pub struct AI {
    is_on: bool,
    is_alive: bool,
    //I put here the sender since we receive it from a message. To be discussed
    //Then we have some communication channels in the planet and some in the AI struct.
    //It sucks, but now that we have the control on the explorer sender, we could also implement multiple explorer for planet!

    explorer_sender: Option<mpsc::Sender<PlanetToExplorer>>
    // planet: Rc<RefCell<Planet>>??
}

impl AI
where
    AI: PlanetAI
{

    fn run(
    ){
        /*
        PLANET AI PURPOSE
        Read messages from each channel, and call the handle_msg method each time that
        a message arrives, and stop this behavior when it get killed.
         */


    }

    fn listen_for_orchestrator(&mut self, sender: Sender<PlanetToOrchestrator>, receiver: Receiver<OrchestratorToPlanet>){
        for msg in receiver{
            //here we receive orch messages, we have to bind them to the proper handle messages
            //in order to call the handle function I need to have the PlanetState. Ho do we get it?
            //Should we pass a mutable reference of the planet to the AI struct?
            //I hope there is a bette way that Luca will figure out :)
            // self.handle_orchestrator_msg(
            //
            // )
        }
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
                })
            }
            OrchestratorToPlanet::Asteroid(_) => {
                match self.handle_asteroid(state, generator, combinator) {
                    None => {
                        //destroyed
                        Some(PlanetToOrchestrator::AsteroidAck {
                            planet_id: state.id(),
                            destroyed: true,
                        })
                    }
                    Some(_) => {
                        // we will never go in this case with our planet
                        Some(PlanetToOrchestrator::AsteroidAck {
                            planet_id: state.id(),
                            destroyed: false,
                        })
                    }
                }
            }
            OrchestratorToPlanet::StartPlanetAI => {
                self.is_on = true;
                Some(PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                })
            }
            OrchestratorToPlanet::StopPlanetAI => {
                self.is_on = false;
                Some(PlanetToOrchestrator::StopPlanetAIResult {
                    planet_id: state.id(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest => {
                Some(PlanetToOrchestrator::InternalStateResponse {
                   planet_id: state.id(),
                   planet_state: state.to_dummy(),
                })
            },
            OrchestratorToPlanet::IncomingExplorerRequest{
                explorer_id, new_mpsc_sender
            } => {
                //if there is another explorer, return Err
                self.explorer_sender = Some(new_mpsc_sender.clone());
                //TODO
                Some(PlanetToOrchestrator::IncomingExplorerResponse {
                    planet_id: state.id(),
                    res: Err("Error".to_string())
                })
            },
            OrchestratorToPlanet::OutgoingExplorerRequest {
                explorer_id
            } =>{
                //Why shouldn't we permit an explorer to go out?
                //How do we drop the sender inside here?
                Some(PlanetToOrchestrator::OutgoingExplorerResponse {
                    planet_id: state.id(),
                    res: Ok(())
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
        match msg {
            ExplorerToPlanet::SupportedResourceRequest{
                explorer_id,
            } => {
                Some(PlanetToExplorer::SupportedResourceResponse { resource_list: generator.all_available_recipes() })
            },
            ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id
            } =>{
                // no combination
                Some(PlanetToExplorer::SupportedCombinationResponse{
                    combination_list: combinator.all_available_recipes(),
                })
            },
            ExplorerToPlanet::GenerateResourceRequest {explorer_id, resource} =>{ 
                let cell = (state.cells_iter_mut().find(|c| c.is_charged()));
                //TODO
                /*
                Here we are not returning a msg to sent to the explorer, we are returning only None
                 */
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
            ExplorerToPlanet::AvailableEnergyCellRequest{
                explorer_id
            } =>{
                let mut cells:u32 = 0 ;
                state.cells_iter().for_each(|c|  {if c.is_charged() { cells += 1 }});
                Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: cells })
                
            },
            ExplorerToPlanet::CombineResourceRequest{
                explorer_id, msg
            } =>{
                // no combination
                // TODO
                // here we send an error and send back the resource that the exp sended to the planet
                // How do we do that? What is the msg param?
                /*
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "No combination supported".to_string(),
                    )),
                })*/
                None
            },
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
    id: u8,
    rx_orchestrator: mpsc::Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<messages::PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<messages::ExplorerToPlanet>,
) -> Result<Planet, String> {
    let ai = AI { is_on: false, is_alive: true, explorer_sender: None};
    let gen_rules = vec![BasicResourceType::Oxygen, BasicResourceType::Hydrogen, BasicResourceType::Carbon];
    let comb_rules = vec![];

    // Construct the planet and return it
    let planet = Planet::new(
        id as u32,
        PlanetType::D,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );

    planet
}
