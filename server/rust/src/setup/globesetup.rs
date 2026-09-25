//! Общий глобальный setup `CGlobeSetup` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей и World-адаптер JJC.

pub(crate) use nebokrai_shared::resources::{
    GlobePlayerPropertyCoefficients, GlobeSetupDecodeError,
    GlobeSetupSnapshot, GlobeStiffenSetup,
};

use crate::worldserver::appworld::jjcsystem::JjcRunConfig;

pub(crate) trait GlobeSetupJjcWorldConfig {
    fn jjc_run_config_world(&self) -> JjcRunConfig;
}

impl GlobeSetupJjcWorldConfig for GlobeSetupSnapshot {
    fn jjc_run_config_world(&self) -> JjcRunConfig {
        let (
            use_jjc,
            rank_interval_seconds,
            pk_timeout_seconds,
            region_id_min,
            region_id_max,
            max_regions_in_use,
        ) = self.jjc_run_config_fields();
        JjcRunConfig {
            use_jjc,
            rank_interval_seconds,
            pk_timeout_seconds,
            region_id_min,
            region_id_max,
            max_regions_in_use,
        }
    }
}
