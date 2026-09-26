//! Клиентская проекция живых состояний и runtime-план их visual, перенесённые
//! из переходного Game (`appserver/states/state.cpp`, клиентские getters
//! `CState::Serialize`-семейства) в Zone skills. Источник: пара
//! `gameserver.exe` SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ `GameServer.pdb` RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2
//! (CodeView RSDS GUID+age совпадают; на этом шаге identity снято заново).
//! Конвенция адресов: S_PUB32 хранит (segment, offset), `.text` = сегмент 1 по
//! RVA 0x1000, поэтому RVA = offset + 0x1000, VA = RVA + 0x400000.
//!
//! Порядок полей записи (ID, time, additional, имя Team) принадлежит проходу
//! `CMoveShape::AddToByteArray_ForClient` (паблик off 0xCCD30 → RVA 0xCDD30 →
//! VA 0x004CDD30; сам двухпроходный писатель — соседний `snapshot`, волна
//! Z-M3); здесь только значения time/additional/team_name каждого из 56
//! вариантов enum-каталога и план loop/updated `CVisualEffect` после runtime
//! Begin.
//!
//! Писатель сверен машинным обходом полного тела (VERIFIED_DISASSEMBLY):
//! байт-флаг из virtual [this+0xD0] (!= 0 даёт 0), DWORD числа непустых
//! позиций m_vStates; оба прохода (счёт и запись) идут по индексам вперёд с
//! пропуском NULL — порядок записей равен порядку позиций арены. На каждое
//! состояние: DWORD ID читается напрямую из поля [state+4] (не виртуал),
//! DWORD time — вызов vtable+0x30, DWORD additional — vtable+0x38; слоты
//! доказаны совпадением адресов с пабликами `?GetRemainedTime@CState@@...`/
//! `?GetAdditionalData@CState@@...` и четырьмя derived-переопределениями.
//! Имя дописывается только при ID == 0x186A6: __RTDynamicCast
//! (RTTI CState 0x69FFE0 → CTeamState 0x69FFF8) и
//! `?GetTeamName@CTeamState@@QAEPADXZ` (паблик off 0x1BEAB0 → RVA 0x1BFAB0 →
//! VA 0x005BFAB0; SSO-ветвь cap [ecx+0x50] ≥ 0x10 → ptr [ecx+0x3c], иначе
//! inline ecx+0x3c) с append C-строки через lstrlenA. `IsEnded@CState`
//! (RVA 0xD8380: `return [ecx+0x30]`) писателем не вызывается — пишутся все
//! непустые позиции. `CPlayer::AddToByteArray_ForClient` (RVA 0x4A480)
//! сначала вызывает базовый проход, затем дописывает player-поля вне записей
//! состояний. Знаковой обработки time («time < 0») нет ни в писателе, ни в
//! getters: пишется DWORD как есть; обёртывание в «отрицательный» остаток
//! возможно только гонкой второго чтения часов и воспроизводится zone тем же
//! двойным чтением. Прежние записи шапки «VA карты 0x005DAD00» для
//! `?Serialize@CState@@...` (off 0x1DAD00) и «0x1BF290» для GetTeamName были
//! без учёта смещения секции/ошибочны: верны RVA 0x1DBD00 / паблик 0x1BEAB0.
//!
//! Значения time (vtable+0x30) сверены с дизассемблом, VERIFIED_DISASSEMBLY:
//! - базовый `CState::GetRemainedTime` RVA 0x201200 = `xor eax,eax; ret` (0;
//!   общий ICF-слитый адрес — тот же код у `GetAdditionalData@CState` и десятков
//!   пустых virtual'ов; «0x600200» из более ранних записей — тот же паблик без
//!   смещения секции). Унаследован: Agility/Natural/Rapture, TaiJi,
//!   EnlargeFullMiss/MaxHp/MaxMp, Origin, MeteorArrow, EnergyHolding,
//!   SoulCollect, Swordship×4, WuXing×5, AutomaticRestore(Hp/Mp, +Fight),
//!   Particular, Team, TianShenXiaFan, Ride → нулевой time/additional каталога.
//! - формула timed (deadline = [ecx+0x2C] + keep; now ≥ deadline → 0, иначе
//!   deadline − now со вторым чтением timeGetTime по IAT 0x64B264) =
//!   `timed_client_state_time`: RVA 0x1F2CD0 (keep +0x38; Heal/Heal2, Blind,
//!   BoaLock, BossBlueQuake, Cure, DaubPoison, Hearten, KnightCut, KnockOut,
//!   Roar, Rush/Rush2, Seal, SpiderWeb, Strike, Restore×2 consumable,
//!   Life/Machine/Mana/Promotion щиты, AutoProtect, UseGoods×5, SuperHeal×2),
//!   RVA 0x1D5F30 (keep +0x3C; Agility2, Callosity/2, BossBlueFury, Pillar,
//!   ImproveExp), RVA 0x205E10 (keep +0x40; Fury, RageBreak, Weak, Wangsheng,
//!   Po/Yu боевой феи ×8), RVA 0x206320 (keep +0x60; BloodLoss, Kerosene,
//!   LeafCut×3, PoisonArrow, SpiderPoison, SpriteBurn), RVA 0x201480
//!   (keep +0x48; GodBless/2), RVA 0x207E00 (keep +0x50; PoisonFog).
//!   Смещения полей — C-layout владельцев в `effects`; формула едина.
//! - RVA 0x1DA030 (CHBYState и общий CExState) = `change_body_client_state_time`
//!   (1 после истечения ненулевого срока, до трёх чтений часов) — ChangeBody и
//!   Extended::Original; RVA 0x1D6320 (CExStateNew и CNotDisappearAfterDead) =
//!   `guarded_client_state_time` (нулевой срок → 0 без чтения часов) —
//!   Extended::New и Undead.
//!
//! Значения additional (vtable+0x38) сверены с дизассемблом,
//! VERIFIED_DISASSEMBLY по получателям и DISCREPANCY в одной группе:
//! - база RVA 0x201200 → 0 для всех классов вне пяти переопределений;
//! - `return [ecx+0x38]`: MeteorArrow (RVA 0xF96E0; поле — текущие стрелы,
//!   AddMeteorArrow RVA 0x1F6970 пишет с потолком [ecx+0x3C]) и Particular
//!   (тот же адрес; ctor RVA 0xF9440 сохраняет аргумент-additional в +0x38);
//! - `return [ecx+0x3C]`: SoulCollect (RVA 0xD7090, ICF-слит с именованным
//!   `?GetNumSouls@CSoulCollectState@@...`; AddSoul RVA 0x1E1BC0
//!   инкрементирует +0x3C с потолком +0x40);
//! - `([ecx+0x38] << 16) | [ecx+0x3C]`: Ride (RVA 0xF8D50; раскладка type/level
//!   по CRideState::Serialize RVA 0xF8F60);
//! - Team (RVA 0x1BFDD0): `(([ecx+0x68] != 0) << 16) | count`, где [ecx+0x68] —
//!   длина std::string пароля (ctor RVA 0x1BFE60: имя string по +0x38, пароль
//!   по +0x54), count — число элементов списка живого CTeam (count-fn
//!   RVA 0x107590) по id из `CPlayer` [+0xB20] через синглтон-менеджер и RTTI
//!   CMoveShape→CPlayer / CSession→CTeam; любой обрыв цепочки даёт 1. Zone
//!   получает число параметром; hub по умолчанию передаёт 1. Принадлежность
//!   считанного контейнера именно членам команды — INFERRED (по классу CTeam);
//!   остальная структура записи Team VERIFIED_DISASSEMBLY.
//! - DISCREPANCY (открыто, поведение не менялось): DefenseShield Life/Machine/
//!   Mana в zone пишут additional = `life()`, а vtable+0x38 всех трёх классов —
//!   базовый 0 (Promotion совпадает: 0; time всех четырёх — timed RVA 0x1F2CD0,
//!   совпадает). Оригинал остаток щита в клиентский снимок не пишет; вопрос
//!   передан владельцу, исправление — отдельным решением.
//!
//! Итог по 56 вариантам: time VERIFIED_DISASSEMBLY у всех 56; additional
//! VERIFIED_DISASSEMBLY у 55 — Team с оговоркой INFERRED о природе младшего
//! слова (численно совпадает по структуре и дефолту 1); единственное
//! расхождение — additional щитов Life/Machine/Mana в DefenseShield.
//! PARTIAL числовых property-формул боевой феи относится к владельцу
//! `effects/battlefairy.rs`, а не к этой записи. Идентичность классов
//! zone ↔ C++ подтверждена vtable/ctor-пабликами в шапках модулей `effects`;
//! отображение перечислено у каждого варианта в `storage`.
//!
//! Единый enum-каталог payload (`StateData`) живёт рядом в `storage` арены
//! состояний; оба match ниже идут прямо по `&StateData` — сварочный шов
//! `StateClientPayload`/`StatePayloadView` снят шагом B переноса арены.

use crate::effects::{CVisualEffect, DefenseShieldState, GOD_BLESS_STATE_2_ID};

use super::storage::StateData;

/// Заимствованная клиентская проекция одного живого экземпляра, без DB Serialize.
/// Только Team дописывает имя после общей тройки ID/time/additional.
#[derive(Clone, Copy, Debug, Default)]
pub struct StateClientRecord<'a> {
    pub time: i32,
    pub additional: u32,
    pub team_name: Option<&'a [u8]>,
}

impl StateClientRecord<'_> {
    fn timed(time: i32) -> Self {
        Self { time, ..Self::default() }
    }
}

/// Клиентская запись одного состояния из `AddToByteArray_ForClient`: 56
/// вариантов в порядке прежнего callback-каталога переходного Game. Пустая
/// client-clause оригинального каталога даёт нулевую запись; часы и состав
/// команды получает только вариант, читавший их в исходных getters.
pub fn state_client_record<'a>(
    state: &'a StateData,
    team_member_count: usize,
    now: &mut dyn FnMut() -> u32,
) -> StateClientRecord<'a> {
    match state {
        StateData::PersistentAgility(_) => StateClientRecord::default(),
        StateData::TaiJi(_) => StateClientRecord::default(),
        StateData::EnlargeFullMiss(_) => StateClientRecord::default(),
        StateData::EnlargeMaxHp(_) => StateClientRecord::default(),
        StateData::EnlargeMaxMp(_) => StateClientRecord::default(),
        StateData::Origin(_) => StateClientRecord::default(),
        StateData::MeteorArrow(state) => StateClientRecord { additional: state.additional_data() as u32, ..StateClientRecord::default() },
        StateData::EnergyHolding(_) => StateClientRecord::default(),
        StateData::SoulCollect(state) => StateClientRecord { additional: state.souls() as u32, ..StateClientRecord::default() },
        StateData::Swordship(_) => StateClientRecord::default(),
        StateData::WuXing(_) => StateClientRecord::default(),
        StateData::Agility2(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Callosity(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Hearten(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::RageBreak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Pillar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::TianShenXiaFan(_) => StateClientRecord::default(),
        StateData::Wangsheng(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Blind(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Rush(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Rush2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::KnockOut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::KnightCut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::SpiderWeb(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Seal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Strike(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Heal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::PoisonArrow(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::SpiderPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::SpriteBurn(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::BloodLoss(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut3(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Kerosene(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Cure(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::BossBlueQuake(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BoaLock(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::GodBless(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Roar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Weak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Fury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BossBlueFury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::PoisonFog(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BattleFairyAttribute(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::DefenseShield(state) => {
            // Машинная сверка `AddToByteArray_ForClient` (RVA 0xCDD30):
            // vtable+0x38 (GetAdditionalData) у CLife/CMachine/CMana указывает
            // на базовую `xor eax,eax; ret` (RVA 0x201200) — оригинал остаток
            // щита в клиентский снимок арены не пишет. Значение `life`
            // принадлежит layout DB-`Serialize`, а не клиентской записи.
            let (time, additional) = match state {
                DefenseShieldState::Life(state) => (state.client_time(now), 0),
                DefenseShieldState::Machine(state) => (state.client_time(now), 0),
                DefenseShieldState::Mana(state) => (state.client_time(now), 0),
                DefenseShieldState::Promotion(state) => (state.client_time(now), 0),
            };
            StateClientRecord { time, additional, team_name: None }
        }
        StateData::DaubPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::AutomaticRestore(_) => StateClientRecord::default(),
        StateData::ConsumableRestore(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Particular(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
        StateData::Team(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(team_member_count), team_name: Some(state.team_name()) },
        StateData::Script(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::ChangeBody(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Extended(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Undead(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Ride(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
    }
}

/// План visual зарегистрированного после runtime Begin состояния: пара
/// loop/updated прежнего callback-каталога переходного Game. Варианты без
/// visual-ресурса возвращают None; прочие получают общий базовый план,
/// one-shot — немедленное завершение через update.
pub fn registered_runtime_state_visual(state: &StateData) -> Option<CVisualEffect> {
    let (loop_value, updated) = match state {
        StateData::PersistentAgility(_) => Some((1, false)),
        StateData::TaiJi(_) => None,
        StateData::EnlargeFullMiss(_) => None,
        StateData::EnlargeMaxHp(_) => None,
        StateData::EnlargeMaxMp(_) => None,
        StateData::Origin(_) => None,
        StateData::MeteorArrow(_) => Some((1, false)),
        StateData::EnergyHolding(_) => Some((1, false)),
        StateData::SoulCollect(_) => Some((1, false)),
        StateData::Swordship(_) => None,
        StateData::WuXing(_) => None,
        StateData::Agility2(_) => Some((0, true)),
        StateData::Callosity(_) => Some((1, false)),
        StateData::Hearten(_) => Some((1, false)),
        StateData::RageBreak(_) => Some((1, false)),
        StateData::Pillar(_) => Some((1, false)),
        StateData::TianShenXiaFan(_) => Some((1, false)),
        StateData::Wangsheng(_) => Some((1, false)),
        StateData::Blind(_) => Some((1, true)),
        StateData::Rush(_) => Some((1, true)),
        StateData::Rush2(_) => Some((1, true)),
        StateData::KnockOut(_) => Some((1, false)),
        StateData::KnightCut(_) => Some((1, false)),
        StateData::SpiderWeb(_) => Some((1, false)),
        StateData::Seal(_) => Some((1, false)),
        StateData::Strike(_) => Some((1, false)),
        StateData::Heal(_) => Some((1, false)),
        StateData::PoisonArrow(_) => Some((1, false)),
        StateData::SpiderPoison(_) => Some((1, false)),
        StateData::SpriteBurn(_) => Some((1, false)),
        StateData::BloodLoss(_) => Some((1, false)),
        StateData::LeafCut(_) => Some((1, false)),
        StateData::LeafCut2(_) => Some((1, false)),
        StateData::LeafCut3(_) => Some((1, false)),
        StateData::Kerosene(_) => Some((1, false)),
        StateData::Cure(_) => Some((1, false)),
        StateData::BossBlueQuake(_) => Some((1, false)),
        StateData::BoaLock(_) => Some((1, false)),
        StateData::GodBless(state) => Some((if state.skill_id() == GOD_BLESS_STATE_2_ID { 0 } else { 1 }, false)),
        StateData::Roar(_) => Some((1, false)),
        StateData::Weak(_) => Some((1, false)),
        StateData::Fury(_) => Some((1, false)),
        StateData::BossBlueFury(_) => Some((1, false)),
        StateData::PoisonFog(_) => Some((1, false)),
        StateData::BattleFairyAttribute(_) => Some((1, false)),
        StateData::DefenseShield(state) => {
            let once = matches!(state, DefenseShieldState::Promotion(_));
            Some((if once { 0 } else { 1 }, once))
        }
        StateData::DaubPoison(_) => Some((1, false)),
        StateData::AutomaticRestore(_) => Some((1, false)),
        StateData::ConsumableRestore(_) => Some((1, false)),
        StateData::Particular(_) => Some((1, true)),
        StateData::Team(_) => Some((1, true)),
        StateData::Script(state) => Some((if state.is_auto_protect() { 1 } else { 0 }, false)),
        StateData::ChangeBody(_) => Some((1, false)),
        StateData::Extended(_) => Some((1, false)),
        StateData::Undead(_) => Some((1, false)),
        StateData::Ride(_) => Some((1, false)),
    }?;
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(loop_value);
    if updated { visual.update_visual_effect(); }
    Some(visual)
}
