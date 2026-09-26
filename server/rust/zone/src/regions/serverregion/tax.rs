//! Tax data-контракты начисления и сбора налогов региона и скалярные
//! state-owner действия над `RegionParamState`: `AddTaxMoney` и
//! `CollectTodayTax`. Исходный владелец — `appserver/serverregion.h/.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`. Двухфазный
//! `CNetSession`-endpoint налогового диалога, лог сбора и публикация доли
//! superior-региону в World остаются у переходного владельца.
//!
//! Доля считается x87-усечением `rate*amount*0.01f`; ядро использует `f64` —
//! расхождение возможно лишь на границе целого при значениях вне 53-битной
//! мантиссы. Машинно доказанная странность: сумма рекурсивного `AddTaxMoney`
//! superior-а и поле #2 доли `0x6012E` в оригинале — неинициализированный
//! стековый DWORD; клей старого пакета пересылает вычисленную долю вместо мусора
//! и не воспроизводит бесконечную рекурсию на циклическом superior-графе
//! (visited-guard + escape `0x6012E`).
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#региональное-пространство

use crate::regions::regionparam::RegionParamState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionTaxAddition {
    pub region_id: i32,
    pub retained: u32,
    pub superior_region_id: Option<i32>,
    pub superior_share: u32,
    pub today_total_tax: u32,
    pub total_tax: u32,
    pub current_tax_rate: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionTaxCollection {
    pub region_id: i32,
    pub collected: u32,
    pub today_total_tax: u32,
    pub total_tax: u32,
    pub current_tax_rate: i32,
}

/// State-owner `AddTaxMoney`: доля superior использует исходное signed
/// wrapping-произведение и x87-усечение, вычитается до доставки, а локальный
/// дневной итог clamp-ится к legacy 4_000_000_000.
pub fn add_tax_money(param: &mut RegionParamState, amount: u32) -> RegionTaxAddition {
    let superior_region_id = (0 < param.superior_region_id).then_some(param.superior_region_id);
    let superior_share = superior_region_id.map_or(0, |_| {
        let product = param.turn_in_tax_rate.wrapping_mul(amount as i32);
        (f64::from(product) * f64::from(0.01_f32)).trunc() as i32 as u32
    });
    let retained = amount.wrapping_sub(superior_share);
    param.today_total_tax = param
        .today_total_tax
        .wrapping_add(retained)
        .min(4_000_000_000);
    RegionTaxAddition {
        region_id: param.region_id,
        retained,
        superior_region_id,
        superior_share,
        today_total_tax: param.today_total_tax,
        total_tax: param.total_tax,
        current_tax_rate: param.current_tax_rate,
    }
}

/// Exact `CollectTodayTax`: сначала переносит дневной итог в общий через
/// DWORD wrapping-add и legacy clamp, затем обнуляет дневной счётчик.
/// Лог и World-публикация принадлежат достигнутому `CGame` caller-у.
pub fn collect_today_tax(param: &mut RegionParamState) -> RegionTaxCollection {
    let collected = param.today_total_tax;
    param.total_tax = param.total_tax.wrapping_add(collected).min(4_000_000_000);
    param.today_total_tax = 0;
    RegionTaxCollection {
        region_id: param.region_id,
        collected,
        today_total_tax: param.today_total_tax,
        total_tax: param.total_tax,
        current_tax_rate: param.current_tax_rate,
    }
}
