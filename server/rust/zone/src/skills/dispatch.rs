//! Форма цели и снимок ожидающей команды навыка Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp/.h,
//! appserver/ai/baseai.cpp/.h и appserver/ai/playerai.cpp/.h.
//! HasTarget VA 0x004C7DD0; сравнение object/point в CPlayerAI
//! VA 0x0050A13A–0x0050A15C и 0x0050A349–0x0050A367.
//! Чтение ID команды и очистка старшего бита: VA 0x00488BAD–0x00488BE4
//! и 0x00488E73–0x00488EAE.
//! Выбор формы цели: VA 0x00488D20–0x00488E20,
//! 0x00488FE6–0x00489109 и 0x0048949D–0x00489547.
//! Enum и полный Eq — внутренняя модель Rust; ключ ожидающего запроса
//! не включает уровень боевого духа и GUID цели.

use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;

const PLAYER_TYPE: i32 = 400;
const SKILL_ID_MASK: u32 = i32::MAX as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSkillRequest {
    pub raw_skill_id: i32,
    pub target_type: i32,
    pub target_id: i32,
    pub target_x: i32,
    pub target_y: i32,
}

impl PlayerSkillRequest {
    pub const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Вычисленные Game признаки доступности цели, а не поля клиентской команды.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerSkillRequestFacts {
    pub symbol_attackable: bool,
    pub player_ai_available: bool,
    pub object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairySkillRequest {
    pub raw_skill_id: i32,
    pub target_type: i32,
    pub target_id: i32,
    pub property_offset: i32,
    pub target_x: i32,
    pub target_y: i32,
}

impl BattleFairySkillRequest {
    pub const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Вычисленные Game признаки доступности цели, а не поля клиентской команды.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BattleFairySkillRequestFacts {
    pub symbol_attackable: bool,
    pub player_ai_available: bool,
    pub object_target_available: bool,
}

/// Поля цели после возможной подстановки self-target из определения навыка.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillTarget {
    pub object_type: i32,
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillTargetForm {
    SelfTarget,
    Point { x: i32, y: i32 },
    Object { target: ShapeIdentity },
}

impl SkillTarget {
    /// Нулевые type/id ведут к координатной перегрузке только при двух
    /// ненулевых координатах; иначе вызывается объектная перегрузка на себя.
    pub const fn form(self) -> SkillTargetForm {
        if self.object_type == 0 || self.object_id == 0 {
            if self.x == 0 || self.y == 0 {
                SkillTargetForm::SelfTarget
            } else {
                SkillTargetForm::Point {
                    x: self.x,
                    y: self.y,
                }
            }
        } else {
            SkillTargetForm::Object {
                target: ShapeIdentity {
                    object_type: self.object_type,
                    id: self.object_id,
                    ex_id: CGuid::GUID_INVALID,
                },
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerSkillDispatch {
    SelfTarget {
        skill_id: u32,
        player_id: i32,
    },
    Point {
        skill_id: u32,
        x: i32,
        y: i32,
    },
    Object {
        skill_id: u32,
        target: ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillDispatch {
    SelfTarget {
        skill_id: u32,
        skill_level: i32,
        player_id: i32,
    },
    Point {
        skill_id: u32,
        skill_level: i32,
        x: i32,
        y: i32,
    },
    Object {
        skill_id: u32,
        skill_level: i32,
        target: ShapeIdentity,
    },
}

macro_rules! skill_dispatch_request {
    ($($dispatch:ty),+ $(,)?) => {$(
        impl $dispatch {
            const fn pending_request_key(self) -> (u32, u8, i32, i32) {
                match self {
                    Self::SelfTarget { skill_id, player_id, .. } => (skill_id, 2, PLAYER_TYPE, player_id),
                    Self::Point { skill_id, x, y, .. } => (skill_id, 1, x, y),
                    Self::Object { skill_id, target, .. } =>
                        (skill_id, 2, target.object_type, target.id),
                }
            }

            pub const fn skill_id(self) -> u32 {
                self.pending_request_key().0
            }

            pub fn same_pending_request(self, other: Self) -> bool {
                self.pending_request_key() == other.pending_request_key()
            }

            pub const fn object_target(self) -> Option<ShapeIdentity> {
                match self {
                    Self::SelfTarget { player_id, .. } => Some(ShapeIdentity {
                        object_type: PLAYER_TYPE,
                        id: player_id,
                        ex_id: CGuid::GUID_INVALID,
                    }),
                    Self::Object { target, .. } => Some(target),
                    Self::Point { .. } => None,
                }
            }

            /// CBaseAI::HasTarget (0x004C7DD0): знак важен для type/id,
            /// координаты проверяются только на ноль, не на границы региона.
            pub const fn has_target(self) -> bool {
                match self {
                    Self::Point { x, y, .. } => x != 0 && y != 0,
                    _ => match self.object_target() {
                        Some(target) => target.object_type > 0 && target.id > 0,
                        None => false,
                    },
                }
            }
        }
    )+};
}

skill_dispatch_request!(PlayerSkillDispatch, BattleFairySkillDispatch);

impl PlayerSkillDispatch {
    /// При отсутствии текущего CSkill OnSchedule выбирает default owner,
    /// но сохраняет уже извлечённую цель; ожидающий FIFO не меняется.
    pub const fn with_skill_id(mut self, selected: u32) -> Self {
        match &mut self {
            Self::SelfTarget { skill_id, .. }
            | Self::Point { skill_id, .. }
            | Self::Object { skill_id, .. } => *skill_id = selected,
        }
        self
    }
}

impl BattleFairySkillDispatch {
    pub const fn skill_level(self) -> i32 {
        match self {
            Self::SelfTarget { skill_level, .. }
            | Self::Point { skill_level, .. }
            | Self::Object { skill_level, .. } => skill_level,
        }
    }
}
