use std::future::Future;
pub mod mock;
mod video;

pub struct TransitionError<ME> {
    pub me: ME,
    pub error: anyhow::Error,
}

pub trait StandbyMode {
    type Configure: ConfigureMode<Standby = Self>;
    fn configure(self) -> impl Future<Output = Result<Self::Configure, TransitionError<Self>>>
    where
        Self: Sized;
    type Operate: OperateMode<Standby = Self>;
    fn operate(self) -> impl Future<Output = Result<Self::Operate, TransitionError<Self>>>
    where
        Self: Sized;
    fn state(&self) -> &'static str {
        "standby"
    }
}

pub trait OperateMode {
    type Standby: StandbyMode<Operate = Self>;
    fn standby(self) -> impl Future<Output = Self::Standby>;
    fn state(&self) -> &'static str {
        "operate"
    }
}

pub trait ConfigureMode {
    type Standby: StandbyMode<Configure = Self>;
    fn standby(self) -> impl Future<Output = Self::Standby>;
    fn state(&self) -> &'static str {
        "configure"
    }
}
