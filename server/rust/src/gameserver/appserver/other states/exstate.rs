//! Владелец обычных extended-state `CExState/CExStateNew` GameServer.
//! OnUpdateProperties +0x24 (0x005D95A0) обоих классов разрешает живого
//! sufferer-player; ненулевой WORD addon сначала складывается как DWORD
//! с переполнением, затем ограничивается INT_MAX. Нулевой addon не трогает
//! поле. WORD и element складываются с переполнением; visual/часов нет.
//!
//! Контракт подтверждён `gameserver.exe + GameServer.pdb`, исходными owner-ами
//! `other states/exstate.cpp`, `exstatenew.cpp` и caller-ом
//! `CMoveShape::Add/Del/GetExState*`. Rust enum заменяет два raw `CState*`, но
//! сохраняет state ID `0x32/0x33`, byte-layout `40/52`, wrapping DWORD clock,
//! replacement по type/level, property overlay и periodic item consumption.
//! Client-time различает два exact vtable-owner-а: original `CExState`
//! разделяет `CHBYState::GetRemainedTime` и возвращает `1` после истечения
//! ненулевого срока, а `CExStateNew` возвращает `0`.
//! Ниже сохранены исходные перегрузки и операции, не закрытые этим Begin/End.
//! Little-endian поля читает и пишет общий legacy codec поверх `bytes`.
//! AI обоих вариантов сравнивает абсолютный wrapping DWORD deadline строго
//! с текущим tick (`0x005d94e0/0x005d9fb0`); тот же порядок сохраняется для
//! periodic item consumption. Elapsed-сравнение меняло бы переход через ноль.
//! Persisted Serialize (`0x005d9510/0x005d9bb0`) заменяет keeptime остатком
//! прямо в живом объекте, не перезапуская started_ms. Клиентская проекция
//! только читает остаток; эти два пути нельзя объединять по побочным эффектам.
//! Payload хранится в общей арене CMoveShape; decode_at читает только
//! достигнутую фабрикой запись, сохраняя её вариант и serialized offset.
//! AddEx/AddExNew (0x004D1E40/0x004D20D0) сначала сохраняют параметры фабрики,
//! затем обходят все живые позиции того же ID с совпавшим WORD type или level:
//! direct End, свежий остаток той же позиции и его destructor, без уплотнения.
//! Только после этого Begin(this,this) (0x005D9780/0x005D9C40) проверяет sufferer,
//! читает один базовый clock, создаёт loop1 visual и делает Update(0) до append.
//! Успех завершает отдельный UpdateProperty; самостоятельного OnChangeStates нет.
//! Конструкторы 0x005D9230/0x005D9990 часов не читают: base timestamp равен нулю, New.last_item_tick
//! также ноль (0x005D99A0); из этих двух времён объектный Begin меняет только
//! базовый timestamp, не перезапуская item clock.
//! Vtable 0x0065E33C/0x0065E39C имеют End +0x1C = 0x005FD420:
//! optional visual Update(1), свежий GetSufferer и RemoveState, без base End
//! и без записи state.ended. Visual 0x005D9830/0x005D9CF0 проверяет свой ended
//! и фактического sufferer; base visual tail выполняется и при missing sufferer.
//! Технический cache новой записи имеет 44/56 байт вместе с ID и полный keepTime,
//! без игрового Serialize и часов. При save общей записи меняется только остаток:
//! исходные padding-байты загруженного tagExState не заменяются нулями.
//! Save один раз вызывает native remaining-getter в общем порядке состояний;
//! тот же результат пишет в запись и живой keepTime до следующего экземпляра.
//! ExNew.use_item0x005D9E50 использует фактического Sufferer и возвращает
//! реальное списанное количество. GS0128 отправляется через SendSystemInfo
//! 0x0042CD70: BF807(FFFFFFFF,CString), без второго цвета BF806.
//! Общий адаптер CGame заменяет небезопасный sprintf в buffer256 ограничением
//! 255 байт, NULL goods-name — пустой строкой, неверный non-player cast — нулём.
//! Это безопасные границы для native UB, а не native-контракт этих случаев.

use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;

pub(crate) const EX_STATE_ID: u32 = 0x32;
pub(crate) const EX_STATE_NEW_ID: u32 = 0x33;
const EX_STATE_BYTES: usize = 40;
const EX_STATE_NEW_BYTES: usize = 52;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExtendedStateKind {
    Original,
    New,
}

impl ExtendedStateKind {
    pub(crate) const fn state_id(self) -> u32 {
        match self {
            Self::Original => EX_STATE_ID,
            Self::New => EX_STATE_NEW_ID,
        }
    }

    const fn parameter_bytes(self) -> usize {
        match self {
            Self::Original => EX_STATE_BYTES,
            Self::New => EX_STATE_NEW_BYTES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtendedState {
    pub(crate) kind: ExtendedStateKind,
    pub(crate) state_type: u16,
    pub(crate) level: u32,
    pub(crate) keep_time_ms: u32,
    pub(crate) maximum_hp: u16,
    pub(crate) maximum_mp: u16,
    pub(crate) minimum_attack: u16,
    pub(crate) maximum_attack: u16,
    pub(crate) element_modify: u16,
    pub(crate) defense: u16,
    pub(crate) element_resistance: u16,
    pub(crate) cch: u16,
    pub(crate) full_miss: u16,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) hit: u16,
    pub(crate) dodge: u16,
    pub(crate) item_index: u32,
    pub(crate) item_amount: u32,
    pub(crate) frequency_ms: u32,
    pub(crate) started_ms: u32,
    pub(crate) last_item_tick_ms: u32,
    serialized_offset: Option<usize>,
}

impl ExtendedState {
    pub(crate) fn from_factory(
        kind: ExtendedStateKind,
        level: u32,
        factory: &CSkillFactory,
    ) -> Option<Self> {
        if level == 0 {
            return None;
        }
        let properties = factory.query_skill_base_properties(kind.state_id(), level as i32)?;
        let p = |usage| properties.query_property(usage);
        Some(Self {
            kind,
            state_type: p(20_010) as u16,
            level,
            keep_time_ms: p(10_002),
            maximum_hp: p(118) as u16,
            maximum_mp: p(119) as u16,
            minimum_attack: p(116) as u16,
            maximum_attack: p(117) as u16,
            element_modify: p(115) as u16,
            defense: p(109) as u16,
            element_resistance: p(112) as u16,
            cch: p(108) as u16,
            full_miss: p(127) as u16,
            attack_avoid: p(128) as u16,
            element_avoid: p(129) as u16,
            hit: p(20_001) as u16,
            dodge: p(110) as u16,
            item_index: (kind == ExtendedStateKind::New)
                .then(|| p(50_001))
                .unwrap_or(0),
            item_amount: (kind == ExtendedStateKind::New)
                .then(|| p(50_002))
                .unwrap_or(0),
            frequency_ms: (kind == ExtendedStateKind::New)
                .then(|| p(6_001))
                .unwrap_or(0),
            started_ms: 0,
            last_item_tick_ms: 0,
            serialized_offset: None,
        })
    }

    pub(crate) fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
    }

    pub(crate) fn decode_at(payload: &[u8], offset: usize, now_ms: u32) -> Option<Self> {
        let Some(state_id) = read_u32(payload, offset) else {
            return None;
        };
        let kind = match state_id {
            EX_STATE_ID => ExtendedStateKind::Original,
            EX_STATE_NEW_ID => ExtendedStateKind::New,
            _ => return None,
        };
        let base = offset.checked_add(4)?;
        if base.checked_add(kind.parameter_bytes())? > payload.len() {
            return None;
        }
        let Some(level) = read_u32(payload, base + 4) else {
            return None;
        };
        if level == 0 {
            return None;
        }
        Some(Self {
            kind,
            state_type: read_u16(payload, base).unwrap_or_default(),
            level,
            keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
            maximum_hp: read_u16(payload, base + 12).unwrap_or_default(),
            maximum_mp: read_u16(payload, base + 14).unwrap_or_default(),
            minimum_attack: read_u16(payload, base + 16).unwrap_or_default(),
            maximum_attack: read_u16(payload, base + 18).unwrap_or_default(),
            element_modify: read_u16(payload, base + 20).unwrap_or_default(),
            defense: read_u16(payload, base + 22).unwrap_or_default(),
            element_resistance: read_u16(payload, base + 24).unwrap_or_default(),
            cch: read_u16(payload, base + 26).unwrap_or_default(),
            full_miss: read_u16(payload, base + 28).unwrap_or_default(),
            attack_avoid: read_u16(payload, base + 30).unwrap_or_default(),
            element_avoid: read_u16(payload, base + 32).unwrap_or_default(),
            hit: read_u16(payload, base + 34).unwrap_or_default(),
            dodge: read_u16(payload, base + 36).unwrap_or_default(),
            item_index: (kind == ExtendedStateKind::New)
                .then(|| read_u32(payload, base + 40).unwrap_or_default())
                .unwrap_or(0),
            item_amount: (kind == ExtendedStateKind::New)
                .then(|| read_u32(payload, base + 44).unwrap_or_default())
                .unwrap_or(0),
            frequency_ms: (kind == ExtendedStateKind::New)
                .then(|| read_u32(payload, base + 48).unwrap_or_default())
                .unwrap_or(0),
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            serialized_offset: Some(offset),
        })
    }

    pub(crate) const fn state_id(&self) -> u32 {
        self.kind.state_id()
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + self.kind.parameter_bytes()))
    }

    pub(crate) fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_state_time(&self, now: &mut dyn FnMut() -> u32) -> u32 {
        use crate::gameserver::appserver::states::state::{
            change_body_client_time, extended_client_time,
        };
        match self.kind {
            ExtendedStateKind::Original => change_body_client_time(self.started_ms, self.keep_time_ms, now),
            ExtendedStateKind::New => extended_client_time(self.started_ms, self.keep_time_ms, now),
        }
    }

    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        let deadline = self.started_ms.wrapping_add(self.keep_time_ms);
        match self.kind {
            ExtendedStateKind::Original => {
                if self.keep_time_ms != 0 && deadline <= now_ms {
                    1
                } else if deadline <= now_ms {
                    0
                } else {
                    deadline.wrapping_sub(now_ms)
                }
            }
            ExtendedStateKind::New => {
                if self.keep_time_ms == 0 || deadline <= now_ms {
                    0
                } else {
                    deadline.wrapping_sub(now_ms)
                }
            }
        }
    }

    pub(crate) fn item_due(&mut self, now_ms: u32) -> bool {
        if self.kind == ExtendedStateKind::New && self.last_item_tick_ms == 0 {
            self.last_item_tick_ms = self.started_ms;
        }
        self.kind == ExtendedStateKind::New
            && self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.last_item_tick_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    pub(crate) fn restart_item_clock(&mut self, now_ms: u32) {
        self.last_item_tick_ms = now_ms;
    }

    pub(crate) fn encoded_for_install(&self) -> Vec<u8> {
        let mut payload = vec![0; 4 + self.kind.parameter_bytes()];
        let base = 4;
        write_u32(&mut payload, 0, self.state_id());
        write_u16(&mut payload, base, self.state_type);
        write_u32(&mut payload, base + 4, self.level);
        write_u32(&mut payload, base + 8, self.keep_time_ms);
        for (position, value) in [
            (12, self.maximum_hp),
            (14, self.maximum_mp),
            (16, self.minimum_attack),
            (18, self.maximum_attack),
            (20, self.element_modify),
            (22, self.defense),
            (24, self.element_resistance),
            (26, self.cch),
            (28, self.full_miss),
            (30, self.attack_avoid),
            (32, self.element_avoid),
            (34, self.hit),
            (36, self.dodge),
        ] {
            write_u16(&mut payload, base + position, value);
        }
        if self.kind == ExtendedStateKind::New {
            write_u32(&mut payload, base + 40, self.item_index);
            write_u32(&mut payload, base + 44, self.item_amount);
            write_u32(&mut payload, base + 48, self.frequency_ms);
        }
        payload
    }

    pub(crate) fn update_serialized_record(&self, payload: &mut [u8], offset: usize, remaining: u32) {
        if offset.checked_add(4 + self.kind.parameter_bytes())
            .is_some_and(|end| end <= payload.len()) {
            write_u32(payload, offset + 12, remaining);
        }
    }

    pub(crate) fn commit_serialized_time(&mut self, remaining: u32) {
        self.keep_time_ms = remaining;
    }

    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле CExState");
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле CExState");
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp


// ============================================================================
// FUNCTION: CExState::CExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:24
// RVA: 0x001D92B0
// ADDRESS: 005d92b0
// PROTOTYPE: undefined __thiscall CExState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::~CExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:33
// RVA: 0x001D9340
// ADDRESS: 005d9340
// PROTOTYPE: void __thiscall ~CExState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:113
// RVA: 0x001D9350
// ADDRESS: 005d9350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:131
// RVA: 0x001D9410
// ADDRESS: 005d9410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:168
// RVA: 0x001D94E0
// ADDRESS: 005d94e0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:202
// RVA: 0x001D9510
// ADDRESS: 005d9510
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:215
// RVA: 0x001D9550
// ADDRESS: 005d9550
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
