use common_game::components::forge::Forge;
use common_game::components::resource::{BasicResourceType, ComplexResourceRequest, Generator};
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
use ara_kees::planet::create_planet;
use crossbeam_channel::{Sender, Receiver, unbounded, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

// Handling the shared forge
use once_cell::sync::Lazy;
use std::sync::Mutex;
static FORGE: Lazy<Mutex<Forge>> = Lazy::new(|| Mutex::new(Forge::new().unwrap()));
fn get_forge() -> std::sync::MutexGuard<'static, Forge> {
    FORGE.lock().unwrap()
}

pub struct Environement {
    pub phandle: Option<JoinHandle<()>>,
    pub tx_otp: Sender<OrchestratorToPlanet>,
    pub rx_pto: Receiver<PlanetToOrchestrator>,
    pub tx_etp: Sender<ExplorerToPlanet>,
    pub rx_pte: Receiver<PlanetToExplorer>,
    pub tx_pte: Sender<PlanetToExplorer>,
}

impl Environement {
    pub fn new(start_ai: bool) -> Self {
        let (tx_otp, rx_otp) = unbounded::<OrchestratorToPlanet>();
        let (tx_pto, rx_pto) = unbounded::<PlanetToOrchestrator>();
        let (tx_etp, rx_etp) = unbounded::<ExplorerToPlanet>();
        let (tx_pte, rx_pte) = unbounded::<PlanetToExplorer>();

        let mut planet = create_planet(rx_otp, tx_pto.clone(), rx_etp, 1)
            .expect("Failed to create planet");

        let phandle = std::thread::spawn(move || {
            let _ = planet.run();
        });

        if start_ai {
            tx_otp
                .send(OrchestratorToPlanet::StartPlanetAI)
                .expect("Failed to send StartPlanetAI");
            rx_pto.recv().expect("Expected the start ack message");
        }

        Self {
            phandle: Some(phandle),
            tx_otp,
            rx_pto,
            tx_etp,
            rx_pte,
            tx_pte,
        }
    }

    pub fn send_otp(&self, msg: OrchestratorToPlanet) {
        self.tx_otp.send(msg).expect("Failed to send OTP message");
    }

    pub fn recv_pto(&self) -> Result<PlanetToOrchestrator, RecvTimeoutError> {
        self.rx_pto.recv_timeout(Duration::from_millis(150))
    }

    pub fn send_etp(&self, msg: ExplorerToPlanet) {
        self.tx_etp.send(msg).expect("Failed to send ETP message");
    }

    pub fn recv_pte(&self) -> Result<PlanetToExplorer, RecvTimeoutError> {
        self.rx_pte.recv_timeout(Duration::from_millis(150))
    }

    pub fn enable_explorer(&self) {
        self.send_otp(OrchestratorToPlanet::IncomingExplorerRequest {
            explorer_id: 1,
            new_mpsc_sender: self.tx_pte.clone(),
        });

        // Wait for ack from planet
        match self.recv_pto() {
            Ok(PlanetToOrchestrator::IncomingExplorerResponse { .. }) => {}
            Ok(msg) => panic!("Unexpected message while enabling explorer"),
            Err(e) => panic!("Explorer not enabled, recv error"),
        }
    }
}

// impl Drop for Environement {
//     fn drop(&mut self) {
//         // Try to stop the AI/planet gracefully
//         let _ = self.tx_otp.send(OrchestratorToPlanet::StopPlanetAI);
//
//         // Wait for the thread to finish
//         if let Some(handle) = self.phandle.take() {
//             if let Err(e) = handle.join() {
//                 eprintln!("Planet thread panicked: {:?}", e);
//             }
//         }
//     }
// }

#[test]
fn test_ai_disabled_behavior() {
    let env = Environement::new(false);

    env.send_otp(OrchestratorToPlanet::InternalStateRequest);

    match env.recv_pto() {
        Ok(PlanetToOrchestrator::Stopped { planet_id }) => {
            println!("Received expected Stopped message from planet {}", planet_id);
        }
        Ok(other) => panic!("Unexpected response (expected Stopped)"),
        Err(_) => panic!("No response received (expected Stopped)"),
    }
}



#[test]
fn test_orchestrator_sunray_ack() {
    let forge = get_forge();
    let sunray = forge.generate_sunray();
    let env = Environement::new(true);

    env.send_otp(OrchestratorToPlanet::Sunray(sunray));
    let resp = env.recv_pto().expect("Expected a response from planet");

    match resp {
        PlanetToOrchestrator::SunrayAck { planet_id } => {
            assert_eq!(planet_id, 1, "Unexpected planet id (should be 1)");
        }
        _ => panic!("Unexpected response (expected SunrayAck)"),
    }
}

#[test]
fn test_orchestrator_internal_state_request() {
    let env = Environement::new(true);

    env.send_otp(OrchestratorToPlanet::InternalStateRequest);
    let resp = env.recv_pto().expect("Expected a response from planet");

    match resp {
        PlanetToOrchestrator::InternalStateResponse { planet_id, planet_state } => {
            assert_eq!(planet_id, 1, "Unexpected planet id (should be 1)");
        }
        _ => panic!("Unexpected response (expected InternalStateResponse)"),
    }
}

#[test]
fn test_available_cells() {
    let forge = get_forge();
    let env = Environement::new(true);

    env.enable_explorer();
    env.send_otp(OrchestratorToPlanet::Sunray(forge.generate_sunray())); //send a sunray

    env.send_etp(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 1 });
    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
            assert_eq!(available_cells, 1, "One cell should be available");
        }
        _ => panic!("Unexpected response (expected SupportedResourceResponse)"),
    }
}

#[test]
fn test_explorer_supported_resources() {
    let env = Environement::new(true);

    env.enable_explorer();

    env.send_etp(ExplorerToPlanet::SupportedResourceRequest { explorer_id: 1 });
    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
            assert!(resource_list.contains(&BasicResourceType::Oxygen), "Oxygen missing");
            assert!(resource_list.contains(&BasicResourceType::Hydrogen), "Hydrogen missing");
            assert!(resource_list.contains(&BasicResourceType::Carbon), "Carbon missing");
            assert!(!resource_list.contains(&BasicResourceType::Silicon), "Silicon shouldn't be available");
        }
        _ => panic!("Unexpected response (expected SupportedResourceResponse)"),
    }
}

#[test]
fn test_failed_explorer_request_oxygen() {
    let env = Environement::new(true);

    env.enable_explorer();

    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Oxygen,
    });

    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            assert!(resource.is_none(), "Unexpected resource (none should be available)");
        }
        _ => panic!("Unexpected response (expected GenerateResourceResponse)"),
    }
}

#[test]
fn test_success_explorer_request_oxygen() {
    let forge = get_forge();
    let sunray = forge.generate_sunray();
    let env = Environement::new(true);

    env.enable_explorer();

    env.send_otp(OrchestratorToPlanet::Sunray(sunray));

    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Oxygen,
    });

    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            assert_eq!(resource.unwrap().get_type(), BasicResourceType::Oxygen, "Resource should be available and be Oxygen");
        }
        _ => panic!("Unexpected response (expected GenerateResourceResponse)"),
    }
}

#[test]
fn test_explorer_supported_combinations() {
    let env = Environement::new(true);

    env.enable_explorer();

    env.send_etp(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 1 });
    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
            assert!(combination_list.is_empty(), "Unexpected combination list, it should be empty");
        }
        _ => panic!("Unexpected response (expected SupportedCombinationResponse)"),
    }
}

#[test]
fn test_explorer_combination_request() {
    let forge = get_forge();
    let sunray1 = forge.generate_sunray();
    let sunray2 = forge.generate_sunray();
    let env = Environement::new(true);

    env.enable_explorer();
    env.send_otp(OrchestratorToPlanet::Sunray(sunray1));
    env.recv_pto().expect("..");

    env.enable_explorer();
    env.send_otp(OrchestratorToPlanet::Sunray(sunray2));
    env.recv_pto().expect("..");

    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Carbon,
    });
    let c1 = match env.recv_pte().unwrap() {
        PlanetToExplorer::GenerateResourceResponse { resource: Some(r) } => { r.to_carbon().unwrap() }
        _ => panic!(""),
    };

    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Carbon,
    });
    let c2 = match env.recv_pte().unwrap() {
        PlanetToExplorer::GenerateResourceResponse { resource: Some(r) } => { r.to_carbon().unwrap() }
        _ => panic!(""),
    };


    env.send_etp(ExplorerToPlanet::CombineResourceRequest { explorer_id: 1,
        msg: ComplexResourceRequest::Diamond(c1, c2)});

    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::CombineResourceResponse { complex_response } => {
            match complex_response {
                Err((msg, _, _)) => assert_eq!(msg, "Not supported", "Combination should not be supported yet"),
                Ok(_) => panic!("Expected Err for unsupported combination"),
            }
        }
        _ => panic!("Unexpected response (expected CombineResourceResponse)"),
    }
}


