// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MPL-2.0
//

//! Controls the state machine of the system
//!
//! It supports following states, and transitions, as shown in the
//! below diagram:
//!
//! ```text
//!           .--------------.
//!           |              v
//! Park <- Idle -> Poll -> Probe -> Download -> Install -> Reboot
//!           ^      ^        '          '          '
//!           '      '        '          '          '
//!           '      `--------'          '          '
//!           `---------------'          '          '
//!           `--------------------------'          '
//!           `-------------------------------------'
//! ```

#[macro_use]
mod macros;
mod download;
mod idle;
mod install;
mod park;
mod poll;
mod probe;
mod reboot;

pub use self::{
    download::Download, idle::Idle, install::Install, park::Park, poll::Poll, probe::Probe,
    reboot::Reboot,
};

use failure::Error;
use firmware::Metadata;
use runtime_settings::RuntimeSettings;
use settings::Settings;

pub trait StateChangeImpl {
    fn to_next_state(self) -> Result<StateMachine, Error>;
}

/// Holds the `State` type and common data, which is available for
/// every state transition.
#[derive(Debug, PartialEq)]
pub struct State<S>
where
    State<S>: StateChangeImpl,
{
    /// System settings.
    settings: Settings,

    /// Runtime settings.
    runtime_settings: RuntimeSettings,

    /// Firmware metadata.
    firmware: Metadata,

    /// Package UID applied
    applied_package_uid: Option<String>,

    /// State type with specific data and methods.
    state: S,
}

/// The struct representing the state machine.
#[derive(Debug, PartialEq)]
pub enum StateMachine {
    /// Park state
    Park(State<Park>),

    /// Idle state
    Idle(State<Idle>),

    /// Poll state
    Poll(State<Poll>),

    /// Probe state
    Probe(State<Probe>),

    /// Download state
    Download(State<Download>),

    /// Install state
    Install(State<Install>),

    /// Reboot state
    Reboot(State<Reboot>),
}

impl StateMachine {
    pub fn new(settings: Settings, runtime_settings: RuntimeSettings, firmware: Metadata) -> Self {
        StateMachine::Idle(State {
            settings,
            runtime_settings,
            firmware,
            applied_package_uid: None,
            state: Idle {},
        })
    }

    pub fn run(self) {
        self.step()
    }

    fn step(self) {
        match self.move_to_next_state() {
            Ok(StateMachine::Park(_)) => {
                debug!("Parking state machine.");
                return;
            }
            Ok(s) => s.run(),
            Err(e) => error!("{}", e),
        }
    }

    fn move_to_next_state(self) -> Result<StateMachine, Error> {
        match self {
            StateMachine::Park(s) => Ok(s.to_next_state()?),
            StateMachine::Idle(s) => Ok(s.to_next_state()?),
            StateMachine::Poll(s) => Ok(s.to_next_state()?),
            StateMachine::Probe(s) => Ok(s.to_next_state()?),
            StateMachine::Download(s) => Ok(s.to_next_state()?),
            StateMachine::Install(s) => Ok(s.to_next_state()?),
            StateMachine::Reboot(s) => Ok(s.to_next_state()?),
        }
    }
}
