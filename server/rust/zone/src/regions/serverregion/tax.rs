//! Tax data-контракты начисления и сбора налогов региона исторического
//! GameServer (порция 1) и скалярные state-owner действия над
//! `RegionParamState` (порция 3): `AddTaxMoney` `0x000826E0` и
//! `CollectTodayTax` `0x0007D9B0`. Исходный владелец —
//! `appserver/serverregion.h/.cpp`; точная пара `GameServer/gameserver.exe +
//! GameServer/GameServer.pdb` (SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, RSDS
//! `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, совпадение подтверждено
//! оснасткой `.local/evidence/symbols.py identity`). Двухфазный
//! `CNetSession`-endpoint налогового диалога, лог сбора и публикация доли
//! superior-региону в World остаются у переходного владельца до своих порций.
//!
//! Машинные статусы порций (прямой дизассембл тел точной пары):
//!
//! | функция | RVA | статус |
//! |---|---|---|
//! | `add_tax_money` | `0x000826E0` | `VERIFIED_DISASSEMBLY` |
//! | `collect_today_tax` | `0x0007D9B0` | `VERIFIED_DISASSEMBLY` |
//!
//! `add_tax_money` сверен по всему телу (`0x004826E0-0x00482816`, vtable
//! `+0xC8`): superior-gate по `[+0x228] > 0`, signed `imul` low-32 доли
//! (`0x00482715`), делитель-усечение через x87 `fmul` float-константой `0.01f`
//! из `0x64DBD0` и `fnstcw`-переключением rounding в truncate (`0x0048271D-
//! 0x00482743`), вычитание доли до доставки (`sub ebp, [esp+0x10]`),
//! clamp `0xEE6B2800` обеими ветками `cmp/jae` (`0x004827E5-0x004827F4`) и
//! хвостовой вызов `UpdateTaxToWorldServer` `0x0007C0C0`. Оговорка точности:
//! оригинал считает `rate*amount*0.01f` в 80-битном x87, ядро — в `f64`;
//! расхождение возможно лишь на границе целого при значениях, не
//! представимых в 53-битной мантиссе. Машинно доказанная странность на
//! glue-стороне: сумма, передаваемая рекурсивному `AddTaxMoney` superior-а
//! (вызов virtual `+0xC8` на найденном в `CGame`-карте регионе) и поле #2
//! сообщения доли `0x6012E` — неинициализированный стековый DWORD `[esp+0xC]`
//! (`0x0048277E`, записи в этот слот в теле нет). Клей старого пакета
//! пересылает вычисленную долю вместо мусора и не воспроизводит бесконечную
//! рекурсию на циклическом superior-графе (visited-guard + escape `0x6012E`).
//!
//! `collect_today_tax` сверен по всему телу (`0x0047D9B0-0x0047DAE2`, vtable
//! `+0xCC`): порядок `total = clamp(total + today, 0xEE6B2800)` → `today = 0`
//! (`0x0047D9E2-0x0047DA11`), форматирование `GS0236` аргументами `(имя,
//! collected, today = 0)` (`0x0047DA41-0x0047DA76`), `PutStringToFile("war")`
//! `0x0001CEB0` (`0x0047DAAF`) и хвостовой `UpdateTaxToWorldServer`
//! (`0x0047DAB7`). Тело `UpdateTaxToWorldServer` `0x0007C0C0` тоже сверено:
//! `0x6012D` с полями region ID `[+8]`, today `[+0x224]`, total `[+0x220]`,
//! ставка через virtual `GetTaxRate` `+0xB8`, `Send(false)` — тот же порядок
//! полей реализует достигнутый клей старого `CGame`.

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
