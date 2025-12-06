use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, Generator};
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
use std::sync::mpsc::{Sender,Receiver,channel};
use common_game::components::sunray::Sunray;
use planet::planet::{create_planet, AI};

struct Environement {
    planet: Planet,
    tx_otp: Sender<OrchestratorToPlanet>,
    rx_pto: Receiver<PlanetToOrchestrator>,
    tx_etp: Sender<ExplorerToPlanet>,
    rx_pte: Receiver<PlanetToExplorer>,
}

impl Environement {
    fn new(ai_state: bool) -> Self {
        let (tx_otp, rx_otp) = channel();
        let (tx_pto, rx_pto) = channel();
        let (tx_etp, rx_etp) = channel();
        let (tx_pte, rx_pte) = channel();

        let mut planet = create_planet(1, rx_otp, tx_pto, rx_etp).unwrap();

        if ai_state {
            planet.ai.start(planet.state());
            // TODO
            //  non si può accendere l'AI...
        }

        Self {
            planet,
            tx_otp,
            rx_pto,
            tx_etp,
            rx_pte
        }
    }

    // Helper functions to simulate message sending/receiving
    fn send_orchestrator(&self, msg: OrchestratorToPlanet) {
        self.tx_otp.send(msg).unwrap();
    }

    fn recv_orchestrator(&self) -> PlanetToOrchestrator {
        self.rx_pto.recv().unwrap()
    }

    fn send_explorer(&self, msg: ExplorerToPlanet) {
        self.tx_etp.send(msg).unwrap();
    }

    fn recv_explorer(&self) -> PlanetToExplorer {
        self.rx_pte.recv().unwrap()
    }
}

#[test]
fn test_orchestrator_sunray_ack() {
    let env = Environement::new(true);
    env.send_orchestrator(OrchestratorToPlanet::Sunray(Sunray::default()));
    let resp = env.recv_orchestrator();
    match resp {
        PlanetToOrchestrator::SunrayAck { planet_id } => {
            assert_eq!(planet_id, 1);
        }
        _ => panic!("Expected SunrayAck"),
    }
}

#[test]
fn test_explorer_supported_resources() {
    let env = Environement::new(true);
    env.send_explorer(ExplorerToPlanet::SupportedResourceRequest { explorer_id: 1 });
    let resp = env.recv_explorer();
    match resp {
        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
            assert!(resource_list.contains(&BasicResourceType::Oxygen));
            assert!(resource_list.contains(&BasicResourceType::Hydrogen));
            assert!(resource_list.contains(&BasicResourceType::Carbon));
        }
        _ => panic!("Expected SupportedResourceResponse"),
    }
}

#[test]
fn test_explorer_generate_oxygen() {
    let env = Environement::new(true);
    env.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Oxygen,
    });
    let resp = env.recv_explorer();
    match resp {
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            assert!(resource.is_some(), "Oxygen generation should return a resource");
        }
        _ => panic!("Expected GenerateResourceResponse"),
    }
}

#[test]
fn test_ai_disabled_behavior() {
    let env = Environement::new(false); // AI disabled
    env.send_orchestrator(OrchestratorToPlanet::InternalStateRequest);
    assert!(env.rx_pto.try_recv().is_err());
}