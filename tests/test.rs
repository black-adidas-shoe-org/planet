use common_game::components::forge::Forge;
use common_game::components::resource::BasicResourceType;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
use planet::planet::create_planet;
use crossbeam_channel::{Sender, Receiver, unbounded, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

struct Environement {
    phandle: JoinHandle<()>,
    tx_otp: Sender<OrchestratorToPlanet>,
    rx_pto: Receiver<PlanetToOrchestrator>,
    tx_etp: Sender<ExplorerToPlanet>,
    rx_pte: Receiver<PlanetToExplorer>,
    tx_pte: Sender<PlanetToExplorer>,
    _tx_pto_keep: Sender<PlanetToOrchestrator>,
}

impl Environement {
    fn new(ai_state: bool) -> Self {
        let (tx_otp, rx_otp) = unbounded();
        let (tx_pto, rx_pto) = unbounded();
        let (tx_etp, rx_etp) = unbounded();
        let (tx_pte, rx_pte) = unbounded();

        let mut planet = create_planet(1, rx_otp, tx_pto.clone(), rx_etp).unwrap();

        // spawn the blocking run() in a thread
        let phandle = std::thread::spawn(move || {
            // if run() returns Err, ignore it for the test (or you can assert inside)
            let _ = planet.run();
        });

        // small pause so the planet thread starts and blocks in wait_for_start()
        std::thread::sleep(Duration::from_millis(20));

        // optionally start the AI immediately
        if ai_state {
            tx_otp
                .send(OrchestratorToPlanet::StartPlanetAI)
                .expect("send StartPlanetAI failed");
            // give a tiny moment for planet to process the Start
            std::thread::sleep(Duration::from_millis(10));
        }

        Self {
            phandle,
            tx_otp,
            rx_pto,
            tx_etp,
            rx_pte,
            tx_pte,
            _tx_pto_keep: tx_pto, // keep-alive
        }
    }
    fn send_otp(&self, msg: OrchestratorToPlanet) {
        self.tx_otp.send(msg).expect("send_otp failed");
    }

    fn recv_pto(&self) -> Result<PlanetToOrchestrator, RecvTimeoutError> {
        self.rx_pto.recv_timeout(Duration::from_millis(150))
    }

    fn send_etp(&self, msg: ExplorerToPlanet) {
        self.tx_etp.send(msg).expect("send_etp failed");
    }

    fn recv_pte(&self) -> Result<PlanetToExplorer, RecvTimeoutError> {
        self.rx_pte.recv_timeout(Duration::from_millis(150))
    }

    fn enable_explorer(&self) {
        self.send_otp(OrchestratorToPlanet::IncomingExplorerRequest {
            explorer_id: 1,
            new_mpsc_sender: self.tx_pte.clone(),
        });
        self.recv_pto().expect("Explorer not enabled"); // receive ack
    }
}

#[test]
fn test_ai_disabled_behavior() {
    println!("Building planet with AI offline...");
    let env = Environement::new(false);

    println!("Sending dummy request...");
    env.send_otp(OrchestratorToPlanet::InternalStateRequest);

    let got = env.recv_pto();
    assert!(got.is_err(), "Expected no response since AI is off");
}



#[test]
fn test_orchestrator_sunray_ack() {
    println!("Building planet and sunray...");
    let forge = Forge::new().unwrap();
    let sunray = forge.generate_sunray();
    let env = Environement::new(true);

    println!("Sending sunray...");
    env.send_otp(OrchestratorToPlanet::Sunray(sunray));
    let resp = env.recv_pto().unwrap();

    match resp {
        PlanetToOrchestrator::SunrayAck { planet_id } => {
            assert_eq!(planet_id, 1, "Expected id of the planet (1)");
        }
        _ => panic!("Expected SunrayAck"),
    }
}


#[test]
fn test_explorer_supported_resources() {
    println!("Building planet...");
    let env = Environement::new(true);

    println!("Enabling explorer...");
    env.enable_explorer();

    println!("Sending resource request...");
    env.send_etp(ExplorerToPlanet::SupportedResourceRequest { explorer_id: 1 });
    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
            assert!(resource_list.contains(&BasicResourceType::Oxygen));
            assert!(resource_list.contains(&BasicResourceType::Hydrogen));
            assert!(resource_list.contains(&BasicResourceType::Carbon));
            assert!(!resource_list.contains(&BasicResourceType::Silicon));
        }
        _ => panic!("Expected SupportedResourceResponse"),
    }
}

#[test]
fn test_failed_explorer_request_oxygen() {
    println!("Building planet...");
    let env = Environement::new(true);

    println!("Enabling explorer...");
    env.enable_explorer();

    println!("Sending resource request...");
    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Oxygen,
    });

    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            assert!(resource.is_none(), "No cells to generate resource");
        }
        _ => panic!("Expected GenerateResourceResponse"),
    }
}

#[test]
fn test_success_explorer_request_oxygen() {
    println!("Building planet...");
    let env = Environement::new(true);

    println!("Enabling explorer...");
    env.enable_explorer();

    println!("Charging cell...");
    let forge = Forge::new().unwrap();
    let sunray = forge.generate_sunray();
    env.send_otp(OrchestratorToPlanet::Sunray(sunray));

    println!("Sending resource request...");
    env.send_etp(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id: 1,
        resource: BasicResourceType::Oxygen,
    });

    let resp = env.recv_pte().unwrap();
    match resp {
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            assert_eq!(resource.unwrap().get_type(), BasicResourceType::Oxygen, "No cells to generate resource");
        }
        _ => panic!("Expected GenerateResourceResponse"),
    }
}