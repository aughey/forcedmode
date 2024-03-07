use std::future::Future;

pub struct TransitionError<ME> {
    pub me: ME,
    pub error: anyhow::Error,
}

pub trait HardwareStandby {
    type Configure: HardwareConfigure<Standby = Self>;
    fn configure(self) -> Result<Self::Configure, TransitionError<Self>>
    where
        Self: Sized;
    type Operate: HardwareOperate<Standby = Self>;
    fn operate(self) -> impl Future<Output = Result<Self::Operate, TransitionError<Self>>>
    where
        Self: Sized;
    fn state(&self) -> &'static str {
        "standby"
    }
}

pub trait HardwareOperate {
    type Standby: HardwareStandby<Configure = Self>;
    fn standby(self) -> Self::Standby;
    fn state(&self) -> &'static str {
        "operate"
    }
}

pub trait HardwareConfigure {
    type Standby: HardwareStandby<Configure = Self>;
    fn standby(self) -> Self::Standby;
    fn state(&self) -> &'static str {
        "configure"
    }
}

pub struct MockHardware;
impl HardwareStandby for MockHardware {
    type Configure = MockHardware;
    fn configure(self) -> Result<Self::Configure, crate::TransitionError<Self>> {
        Ok(self)
    }
    type Operate = MockHardware;
    async fn operate(self) -> Result<Self::Operate, crate::TransitionError<Self>> {
        // sleep 2 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(self)
    }
}
impl HardwareOperate for MockHardware {
    type Standby = MockHardware;
    fn standby(self) -> Self::Standby {
        self
    }
}
impl HardwareConfigure for MockHardware {
    type Standby = MockHardware;
    fn standby(self) -> Self::Standby {
        self
    }
}
impl MockHardware {
    pub fn new() -> Self {
        MockHardware
    }
}

#[cfg(test)]
mod tests {
    use crate::{HardwareConfigure, HardwareOperate, HardwareStandby, MockHardware};

    async fn test_hardware(mut hardware: impl HardwareStandby) -> anyhow::Result<()> {
        let configure = hardware.configure().map_err(|me| me.error)?;
        hardware = configure.standby();
        let operate = hardware.operate().await.map_err(|me| me.error)?;
        hardware = operate.standby();
        drop(hardware);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock() -> anyhow::Result<()> {
        test_hardware(MockHardware::new()).await
    }

    struct Behavior<HW>
    where
        HW: HardwareStandby,
    {
        hardware: Option<HW>,
    }
    impl<HW> Behavior<HW>
    where
        HW: HardwareStandby,
    {
        fn new(hardware: HW) -> Self {
            Self {
                hardware: Some(hardware),
            }
        }
        async fn run(&mut self) -> anyhow::Result<()> {
            let configure = self
                .hardware
                .take()
                .ok_or_else(|| anyhow::anyhow!("hardware is missing"))?
                .configure()
                .map_err(|me| me.error)?;

            let operate = configure.standby().operate().await.map_err(|me| me.error)?;

            self.hardware = Some(operate.standby());

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_struct() {
        let mut behavior = Behavior::new(MockHardware::new());
        assert!(behavior.run().await.is_ok());
    }
}
