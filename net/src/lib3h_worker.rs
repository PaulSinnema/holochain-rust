//! provides worker that makes use of lib3h

use crate::connection::{
    net_connection::{NetHandler, NetWorker},
    NetResult,
};
use lib3h::{
    dht::mirror_dht::MirrorDht,
    engine::{RealEngine, RealEngineConfig},
    transport_wss::TransportWss,
};

use lib3h_protocol::{network_engine::NetworkEngine, protocol_client::Lib3hClientProtocol};

/// A worker that makes use of lib3h / NetworkEngine.
/// It adapts the Worker interface with Lib3h's NetworkEngine's interface.
/// Handles `Protocol` and translates `JsonProtocol` to `Lib3hProtocol`.
/// TODO: currently uses MirrorDht, will need to expand workers to use different
/// generics.
#[allow(non_snake_case)]
pub struct Lib3hWorker {
    handler: NetHandler,
    can_send_P2pReady: bool,
    net_engine: RealEngine<TransportWss<std::net::TcpStream>, MirrorDht>,
}

/// Constructors
impl Lib3hWorker {
    /// Create a new worker connected to the lib3h NetworkEngine
    pub fn new(handler: NetHandler, real_config: RealEngineConfig) -> NetResult<Self> {
        Ok(Lib3hWorker {
            handler,
            can_send_P2pReady: true,
            net_engine: RealEngine::new(
                Box::new(lib3h_sodium::SodiumCryptoSystem::new()),
                real_config,
                "FIXME",
                MirrorDht::new_with_config,
            )?,
        })
    }
}

impl NetWorker for Lib3hWorker {
    /// We got a message from core
    /// -> forward it to the NetworkEngine
    fn receive(&mut self, data: Lib3hClientProtocol) -> NetResult<()> {
        println!("Lib3hWorker.receive(): {:?}", data);
        // Post Lib3hClient messages only
        self.net_engine.post(data.clone())?;
        // Done
        Ok(())
    }

    /// Check for messages from our NetworkEngine
    fn tick(&mut self) -> NetResult<bool> {
        // println!("Lib3hWorker.tick()");
        // Send p2pReady on first tick
        if self.can_send_P2pReady {
            self.can_send_P2pReady = false;
        }
        // Tick the NetworkEngine and check for incoming protocol messages.
        let (did_something, output) = self.net_engine.process()?;
        if did_something {
            for msg in output {
                self.handler.handle(Ok(msg))?;
            }
        }
        Ok(did_something)
    }

    /// Set the advertise as worker's endpoint
    fn endpoint(&self) -> Option<String> {
        Some(self.net_engine.advertise().to_string())
    }
}

#[cfg(test)]
mod tests {
    // FIXME
}
