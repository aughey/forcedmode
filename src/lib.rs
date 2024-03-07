use std::future::Future;
pub mod mock;
mod video;

/// An error that can occur when transitioning between states.
///
/// Since ownership of the value is passed into the transition function, if
/// there is an error, the value is returned to the caller through this
/// error type along with the error that occurred.
pub struct TransitionError<OWNER> {
    pub owner: OWNER,
    pub error: anyhow::Error,
}

/// A representation of a hardware device that can be in one of three states:
/// Standby, Configure, or Operate.  Transitions happen between Standby and
/// Configure, and Standby and Operate.  Both Configure and Operate can
/// transition back to Standby.
pub trait StandbyMode {
    // The associated types are the types that the StandbyMode can transition to.
    // Note, when transitioning back to StandbyMode, the associated type is forced
    // to be the same as the type implementing StandbyMode.
    type Configure: ConfigureMode<Standby = Self>;
    type Operate: OperateMode<Standby = Self>;

    /// Transition to the configure state.  On error, will return self back to the caller.
    fn configure(self) -> impl Future<Output = Result<Self::Configure, TransitionError<Self>>>
    where
        Self: Sized;
    /// Transition to the operate state.  On error, will return self back to the caller.
    fn operate(self) -> impl Future<Output = Result<Self::Operate, TransitionError<Self>>>
    where
        Self: Sized;

    /// The current state of the hardware.
    fn state(&self) -> &'static str {
        "standby"
    }
}

/// A representation of a hardware device that is in the Operate state.
pub trait OperateMode {
    /// The associated type is the type that the OperateMode can transition to.
    type Standby: StandbyMode<Operate = Self>;

    /// Transition back to standby.
    fn standby(self) -> impl Future<Output = Self::Standby>;

    /// The current state of the hardware.
    fn state(&self) -> &'static str {
        "operate"
    }
}

/// A representation of a hardware device that is in the Configure state.
pub trait ConfigureMode {
    /// The associated type is the type that the ConfigureMode can transition to.
    type Standby: StandbyMode<Configure = Self>;

    /// Transition back to standby.
    fn standby(self) -> impl Future<Output = Self::Standby>;

    /// The current state of the hardware.
    fn state(&self) -> &'static str {
        "configure"
    }
}
