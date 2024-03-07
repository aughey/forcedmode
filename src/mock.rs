use crate::{ConfigureMode, OperateMode, StandbyMode};

#[derive(Default)]
pub struct MockHardware;
pub struct MockOperate;
pub struct MockConfig;
impl StandbyMode for MockHardware {
    type Configure = MockConfig;
    async fn configure(self) -> Result<Self::Configure, crate::TransitionError<Self>> {
        Ok(MockConfig {}) // <-- we'd usually transfer ownership of internal handles here
    }
    type Operate = MockOperate;
    async fn operate(self) -> Result<Self::Operate, crate::TransitionError<Self>> {
        // sleep 2 seconds
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(MockOperate {})
    }
}
impl OperateMode for MockOperate {
    type Standby = MockHardware;
    async fn standby(self) -> Self::Standby {
        MockHardware {}
    }
}
impl ConfigureMode for MockConfig {
    type Standby = MockHardware;
    async fn standby(self) -> Self::Standby {
        MockHardware {}
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
            let configure = self
                .hardware
                .take()
                .ok_or_else(|| anyhow::anyhow!("hardware is missing"))?
                .configure()
                .await
                .map_err(|me| me.error)?;

            let operate = configure
                .standby()
                .await
                .operate()
                .await
                .map_err(|me| me.error)?;

            self.hardware = Some(operate.standby().await);

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_struct() {
        let mut behavior = Behavior::new(MockHardware::default());
        assert!(behavior.run().await.is_ok());
    }
}
