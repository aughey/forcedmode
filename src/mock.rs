use crate::{ConfigureMode, OperateMode, StandbyMode};

#[derive(Default)]
pub struct MockHardware {
    /// A representation of some sort of unique identifier specific to this hardware
    id: u32,
}
pub struct MockOperate {
    id: u32,
}
pub struct MockConfig {
    id: u32,
}
impl StandbyMode for MockHardware {
    type Configure = MockConfig;
    async fn configure(self) -> Result<Self::Configure, crate::TransitionError<Self>> {
        Ok(MockConfig { id: self.id }) // <-- we'd usually transfer ownership of internal handles here
    }
    type Operate = MockOperate;
    async fn operate(self) -> Result<Self::Operate, crate::TransitionError<Self>> {
        // sleep 2 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(MockOperate { id: self.id })
    }
}
impl OperateMode for MockOperate {
    type Standby = MockHardware;
    async fn standby(self) -> Self::Standby {
        MockHardware { id: self.id }
    }
}
impl ConfigureMode for MockConfig {
    type Standby = MockHardware;
    async fn standby(self) -> Self::Standby {
        MockHardware { id: self.id }
    }
}

#[cfg(test)]
mod tests {
    use crate::{mock::MockHardware, ConfigureMode, OperateMode, StandbyMode};

    async fn test_hardware(mut hardware: impl StandbyMode) -> anyhow::Result<()> {
        let configure = hardware.configure().await.map_err(|me| me.error)?;
        hardware = configure.standby().await;
        let operate = hardware.operate().await.map_err(|me| me.error)?;
        hardware = operate.standby().await;
        drop(hardware);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock() -> anyhow::Result<()> {
        test_hardware(MockHardware::default()).await
    }

    struct Behavior<HW>
    where
        HW: StandbyMode,
    {
        hardware: Option<HW>,
    }
    impl<HW> Behavior<HW>
    where
        HW: StandbyMode,
    {
        fn new(hardware: HW) -> Self {
            Self {
                hardware: Some(hardware),
            }
        }
        async fn run(&mut self) -> anyhow::Result<()> {
            assert!(self.hardware.is_some());

            // Grab the hardware from our struct and replace it with None.
            let hardware = self
                .hardware
                .take()
                .ok_or_else(|| anyhow::anyhow!("hardware is missing"))?;

            assert!(self.hardware.is_none());
            // Need to be careful with hardware because it's our only handle to it.

            // In this block below we're going to transition it between modes
            // over and over again.  The last statement will return it back to
            // the original standby state and we'll store it back in our struct.
            let hardware = {
                // Transition to configure
                let configure = hardware.configure().await.map_err(|me| me.error)?;

                // go from configure, to standby, and to operate.  All in one go.
                let operate = configure
                    .standby()
                    .await
                    .operate()
                    .await
                    .map_err(|me| me.error)?;

                // we're done, transition back to standby
                operate.standby().await
            };

            // we're done, store it back in our struct.
            self.hardware = Some(hardware);

            // Demonstate that we have the hardware back.
            assert!(self.hardware.is_some());

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_struct() {
        let mut behavior = Behavior::new(MockHardware::default());
        assert!(behavior.run().await.is_ok());
    }
}
