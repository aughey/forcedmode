use std::future::Future;
pub mod mock;
mod video;

pub struct TransitionError<ME> {
    pub me: ME,
    pub error: anyhow::Error,
}

pub trait StandbyMode {
    // The associated types are the types that the StandbyMode can transition to.
    // Note, when transitioning back to StandbyMode, the associated type is forced
    // to be the same as the type implementing StandbyMode.
    type Configure: ConfigureMode<Standby = Self>;
    type Operate: OperateMode<Standby = Self>;

    /// Transition to the configure state.  On error, will return self back to us.
    fn configure(self) -> impl Future<Output = Result<Self::Configure, TransitionError<Self>>>
    where
        Self: Sized;
    /// Transition to the operate state.  On error, will return self back to us.
    fn operate(self) -> impl Future<Output = Result<Self::Operate, TransitionError<Self>>>
    where
        Self: Sized;

    /// The current state of the hardware.
    fn state(&self) -> &'static str {
        "standby"
    }
}

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
