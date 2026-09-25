//! Постройка CBuild (0x44C), её свойства, footprint и сериализация.
//! Источник: gameserver.exe + GameServer.pdb, appserver/build.h/.cpp.
//!
//! Определения данных и скалярные правила перенесены в Zone
//! `regions/build`; здесь переходный агрегат `CBuild` с прежними
//! сигнатурами, реэкспорт семейства для старого пакета и evidence-блок.
//!
//! Единственный CMoveShape участвует в общем registry, поиске боевых целей
//! и клиентском wire. Vec<u8>/Drop заменяют строку и служебное владение C++.
//! BuildBlockUpdate применяет регион: постройка освобождает footprint при
//! action 6, ворота также при 7. Координаты сохраняют преобразование
//! i32 → f32 → trunc i32; GetTargetPath выбирает ближайшую точку footprint.
//! Action-callbacks AI пусты. OnDied обычной постройки (`0x001DD9D0`, slot
//! `+0x178`) сохраняет script-запуск через `stRunScript`/`RunScript` при
//! убийце-игроке (тип `0x190`) и пустой в этой сборке virtual
//! `CServerRegion::OnSymbolDestroy`; вызова CountryWarSys в его теле нет —
//! победу обрабатывает CountryWarSys отдельно (`on_flag_destroy` `0x000EBE60`
//! вызывается из `OnCountryMessage`). У ворот OnDied пустой: slot `+0x178`
//! vtable CCityGate указывает общий пустой thunk (VA `0x00485540`).
//! Конструктор регистрирует базовую атаку и CFightDefense в общей арене.
//! Отдельный одноаргументный обработчик CBuild::OnBeenAttacked (`0x001DD6B0`,
//! slot `+0x1B0`) не имеет вызывателей в этой сборке: ни одного
//! `call [reg+0x1B0]`, ни одного прямого вызова `0x001DD6B0`, ссылки только из
//! vtable CBuild/CCityGate; попадание через двухаргументный
//! CMoveShape::OnBeenAttacked (slot `+0x15C`) его не вызывает. Смерть проходит
//! общий End/очистку состояний, а OnDied вызывает только
//! `CBaseAI::OnBeenKilled`; у постройки CBaseAI нет (конструктор CMoveShape
//! обнуляет `+0xE8`, AI создают CPlayer-ctor и CMonster::InitAI), поэтому
//! отсутствие AI не подменяется синхронным запуском OnDied/script.

pub(crate) use nebokrai_zone::regions::build::*;

use super::moveshape::{CMoveShape, SKILL_BASE_DEFENSE};
use super::shape::{ShapeIdentity, ShapeView};
use nebokrai_shared::values::CGuid;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CBuild {
    move_shape: CMoveShape,
    pub(crate) hp: u32,
    pub(crate) max_hp: u32,
    pub(crate) defence: u32,
    pub(crate) width_increment: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: u32,
    pub(crate) script: Vec<u8>,
}

impl CBuild {
    /// Создаёт локальное состояние уже успешно созданного factory type `0x44C`;
    /// скалярный пайплайн init принадлежит общей операции Zone build.
    pub(crate) fn from_created(init: BuildInit) -> Self {
        let direction = nebokrai_zone::regions::build::init_direction(init.direction);
        let pos_x = nebokrai_zone::regions::build::init_center_coordinate(init.tile_x);
        let pos_y = nebokrai_zone::regions::build::init_center_coordinate(init.tile_y);
        let mut move_shape = CMoveShape::default();
        nebokrai_zone::regions::build::register_base_skills(SKILL_BASE_DEFENSE, |skill_id, level| {
            move_shape.insert_new_skill(skill_id, level);
        });
        move_shape.shape_mut().set_identity(ShapeIdentity {
            object_type: BUILD_OBJECT_TYPE as i32,
            id: init.id,
            ex_id: CGuid::GUID_INVALID,
        });
        move_shape
            .shape_mut()
            .base_object_mut()
            .set_graphics_id(init.graphics_id);
        move_shape
            .shape_mut()
            .base_object_mut()
            .set_name(&init.name);
        move_shape.shape_mut().set_region_id(init.region_id);
        move_shape.shape_mut().set_pos_xy_base(pos_x, pos_y);
        move_shape.shape_mut().set_direction(direction);
        let mut build = Self {
            move_shape,
            hp: init.max_hp as u32,
            max_hp: init.max_hp as u32,
            defence: init.defence as u32,
            width_increment: init.width_increment,
            height_increment: init.height_increment,
            element_resistance: init.element_resistance as u32,
            script: Vec::new(),
        };
        if nebokrai_zone::regions::build::init_script_uses_file(&init.script) {
            build.set_script_file(init.script);
        }
        build
    }

    pub(crate) fn set_script_file(&mut self, script: Vec<u8>) {
        self.script = script;
    }

    /// При неизменившемся action оригинал не менял change-state и карту;
    /// решение и block-эффект принадлежат общей операции Zone build.
    pub(crate) fn set_action(&mut self, action: u16) -> Option<BuildBlockUpdate> {
        let update = nebokrai_zone::regions::build::decide_set_action_update(
            self.action(),
            action,
            self.region_id(),
            self.tile_x(),
            self.tile_y(),
            self.width_increment,
            self.height_increment,
        )?;
        self.move_shape.shape_mut().set_change_state(0);
        self.move_shape.shape_mut().set_action(action);
        Some(update)
    }

    pub(crate) fn set_hp(&mut self, hp: u32) {
        self.hp = hp;
    }

    /// Exact `CBuild::IsAttackAble`: локальные action/death guards принадлежат
    /// самой постройке, а virtual region-policy передаётся уже разрешённым
    /// результатом owning региона. Для country это `BuildIsAttackAble`, для
    /// остальных регионов — `SymbolIsAttackAble`.
    pub(crate) fn is_attackable_in_region(
        &self,
        region_allows: impl FnOnce() -> bool,
    ) -> bool {
        nebokrai_zone::regions::build::is_attackable_in_region(self.action(), self.hp, region_allows)
    }

    /// Стадия `ApplyFinalDamage` меняет только HP. Смертельные callbacks и
    /// packet выполняются раньше отдельного virtual `SetAction(6)`.
    pub(crate) fn apply_combat_damage(&mut self, damage: u32) {
        nebokrai_zone::regions::build::apply_combat_damage(&mut self.hp, damage);
    }

    pub(crate) fn refresh_hp(&mut self) {
        nebokrai_zone::regions::build::refresh_hp(&mut self.hp, self.max_hp);
    }

    pub(crate) const fn hp(&self) -> u32 {
        self.hp
    }

    pub(crate) const fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub(crate) fn defence(&self) -> u32 {
        self.defence
    }

    pub(crate) fn element_resistance(&self) -> u32 {
        self.element_resistance
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) const fn object_type(&self) -> u32 {
        self.move_shape.shape().identity().object_type as u32
    }

    pub(crate) const fn id(&self) -> i32 {
        self.move_shape.shape().identity().id
    }

    pub(crate) const fn region_id(&self) -> i32 {
        self.move_shape.shape().get_region_id()
    }

    pub(crate) const fn action(&self) -> u16 {
        self.move_shape.shape().get_action()
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.move_shape.shape().base_object().get_name()
    }

    // Клетка читается из единственного CShape, в том числе после ForceMove.
    // Некорректное float-преобразование сохраняет native integer-indefinite.
    pub(crate) fn tile_x(&self) -> i32 {
        self.move_shape.shape().get_tile_x().unwrap_or(i32::MIN)
    }

    pub(crate) fn tile_y(&self) -> i32 {
        self.move_shape.shape().get_tile_y().unwrap_or(i32::MIN)
    }

    pub(crate) fn shape_view(&self) -> ShapeView {
        let shape = self.move_shape.shape();
        ShapeView {
            identity: shape.identity(),
            tile_x: self.tile_x(),
            tile_y: self.tile_y(),
            pos_x_bits: shape.get_pos_x().to_bits(),
            pos_y_bits: shape.get_pos_y().to_bits(),
            figure: nebokrai_zone::regions::build::figure_from_increments(
                self.width_increment,
                self.height_increment,
            ),
        }
    }

    /// Exact `CBuild::GetBeAttackedPoint` (RVA `0x001DD350`): выбирает
    /// ближайшую к атакующему клетку прямоугольного footprint. При равной
    /// Chebyshev-дистанции диагональное направление уступает прямому.
    pub(crate) fn be_attacked_point(&self, attacker_x: i32, attacker_y: i32) -> (i32, i32) {
        CMoveShape::nearest_figure_attack_point(
            self.tile_x(),
            self.tile_y(),
            self.shape_view().figure,
            attacker_x,
            attacker_y,
        )
    }

    pub(crate) fn current_block_update(&self) -> BuildBlockUpdate {
        nebokrai_zone::regions::build::block_snapshot(
            self.action(),
            self.region_id(),
            self.tile_x(),
            self.tile_y(),
            self.width_increment,
            self.height_increment,
        )
    }

    /// Exact `CBuild::AddToByteArray`: общий current-state serializer
    /// `CMoveShape`, затем общий 0x18-byte + WORD property-хвост Zone build.
    pub(crate) fn encode_client_snapshot(
        &self,
        include_child: bool,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let mut payload = self.move_shape.encode_client_snapshot(
            include_child,
            self.hp == 0,
            timed_state_now_milliseconds,
        )?;
        nebokrai_zone::regions::build::encode_client_property(
            &mut payload,
            BuildProperties {
                hp: self.hp,
                max_hp: self.max_hp,
                defence: self.defence,
                width_increment: self.width_increment,
                height_increment: self.height_increment,
                element_resistance: self.element_resistance,
            },
            self.action(),
        );
        Some(payload)
    }

    /// Exact `CBuild::DecordFromByteArray`: base owner вызывает `CShape`, а
    /// не `CMoveShape` decoder, затем общий 0x18-byte property-хвост Zone
    /// build; запись колонок и commit cursor идут после coordinate validation.
    pub(crate) fn decode_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<(), BuildDecodeError> {
        self.move_shape
            .shape_mut()
            .decode_from_byte_array(source, cursor, include_child)
            .map_err(BuildDecodeError::Shape)?;
        let (properties, position) =
            nebokrai_zone::regions::build::decode_client_property(source, *cursor)
                .map_err(BuildDecodeError::Property)?;
        self
            .move_shape
            .shape()
            .get_tile_x()
            .map_err(BuildDecodeError::Coordinate)?;
        self
            .move_shape
            .shape()
            .get_tile_y()
            .map_err(BuildDecodeError::Coordinate)?;

        *cursor = position;
        self.hp = properties.hp;
        self.max_hp = properties.max_hp;
        self.defence = properties.defence;
        self.width_increment = properties.width_increment;
        self.height_increment = properties.height_increment;
        self.element_resistance = properties.element_resistance;
        Ok(())
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.h

// ============================================================================
// FUNCTION: CBuild::SetScriptFile
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001CBAC0
//
// Реализовано выше: byte-exact script assignment.
//

// ============================================================================
// FUNCTION: CBuild::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:32
// RVA: 0x001DD140
// ADDRESS: 005dd140
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Реализовано выше как `encode_client_snapshot`: canonical CMoveShape prefix,
// точный 0x18-byte property block и текущий WORD action.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:41
// RVA: 0x001DD180
// ADDRESS: 005dd180
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Реализовано выше как `decode_from_byte_array`: exact CShape wire, затем
// непрерывный 0x18-byte property block с safe-Rust проверкой границ.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::SetAction
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD1C0
//
// Реализовано выше: change-state/action и single-cell block effect.
//

// ============================================================================
// FUNCTION: CBuild::AI
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:67
// RVA: 0x001DD210
// ADDRESS: 005dd210
// PROTOTYPE: void __thiscall AI(void)
//
// Vtable `0x0065E704`: slots `+0x1A4/+0x1A8/+0x1AC` все указывают на
// `0x00601200`, общий `xor eax,eax; ret`. Поэтому достигнутый region scan
// корректно не создаёт для обычной постройки отдельного action-runtime.
//


// ============================================================================
// FUNCTION: CBuild::GetFigure
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:277
// RVA: 0x001DD280
// Реализовано выше в `shape_view`: directions `0/1` используют младший byte
// `height_increment`, `2/3` — `width_increment`, остальные дают ноль.

// ============================================================================
// FUNCTION: CBuild::SetTileXY
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD2B0
//
// Реализовано выше: x87-compatible coordinate conversion.
//

// ============================================================================
// FUNCTION: CBuild::GetAttackerDir
// STATUS: VERIFIED_DISASSEMBLY (тело); живые вызыватели — PARTIAL
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:297
// RVA: 0x001DD300
// ADDRESS: 005dd300
// PROTOTYPE: long __thiscall GetAttackerDir(long param_1, long param_2, long param_3, long param_4)
//
// Vtable `0x0065E704` slot `+0x0B0`; то же значение в slot `+0x0B0` vtable
// CCityGate (`0x0065E8C4`). Тело `0x50` байт до следующего pub
// `GetBeAttackedPoint` (`0x001DD350`), завершение `ret 0x10`.
// Машинный стержень (`0x005DD300`..`0x005DD34D`): `x = CShape::GetTileX(this)`
// (`0x0005B110`), `y = CShape::GetTileY(this)` (`0x0005B140`); virtual
// `GetBeAttackedPoint` (slot `+0xAC`) вызывается как
// `(param_2, param_2, &x, &y)` — второй stack-аргумент передан в обе
// координаты точки атаки, `param_1`/`param_3`/`param_4` телом не читаются;
// результат — `CMoveShape::GetDestDir(this, param_2, param_2, x, y)`
// (`0x000CCF60`). `GetDestDir` — не atan2: 8-way таблица знаков по
// `dx = a0 - a2`, `dy = a1 - a3`: dx>0 → dy>0:7, dy==0:6, dy<0:5;
// dx<0 → dy>0:1, dy==0:2, dy<0:3; dx==0 → dy>0:0, иначе 4 (совпавшая точка
// даёт direction 4). Базовая `CMoveShape::GetAttackerDir` (`0x0004A270`) —
// голый `jmp GetDestDir` со всеми четырьмя аргументами; переопределение
// CBuild сначала пересчитывает точку footprint своим `GetBeAttackedPoint`.
// Вызыватели: прямых `E8` на `0x001DD300` в `.text` нет; все разобранные
// сайты `call [reg+0xB0]` принадлежат другим иерархиям с тем же смещением
// (CServerRegion `+0xB0` = `GetOwnedCityFaction` `0x000485F0`, `+0xB4` =
// `GetOwnedCityUnion` `0x00048600`; скриптовые и менеджер-объекты);
// подтверждённого живого сайта с receiver CMoveShape не установлено —
// использование доказано только для двух vtable-слотов.
//
//

// ============================================================================
// FUNCTION: CBuild::GetBeAttackedPoint
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:309
// RVA: 0x001DD350
// ADDRESS: 005dd350
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Реализовано выше как `be_attacked_point`.
//

// ============================================================================
// FUNCTION: CBuild::IsAttackAble
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:254
// RVA: 0x001DD520
// ADDRESS: 005dd520
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Реализовано выше как `is_attackable_in_region`: action `6` и dead guard
// выполняются до результата virtual region-policy; concrete policy остаётся у
// `CServerRegion`/`CServerCountryRegion` и передаётся без erased указателей.
//
//

// ============================================================================
// FUNCTION: CBuild::CBuild
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD570
//
// Реализовано выше: type 0x44C и zeroed property/action state.
//

// ============================================================================
// FUNCTION: CBuild::GetHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD5F0
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::SetHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD600
//
// Реализовано выше: property mutation.
//

// ============================================================================
// FUNCTION: CBuild::GetMaxHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD610
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::GetDef
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD620
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::GetElementResistant
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD630
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::~CBuild
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD640
//
// Реализовано выше: owned Vec/Drop replacement.
//

// ============================================================================
// FUNCTION: CBuild::OnBeenAttacked
// STATUS: VERIFIED_DISASSEMBLY (тело и контракт пакетов); мёртвый virtual —
// ноль вызывателей в этой сборке
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:104
// RVA: 0x001DD6B0
// ADDRESS: 005dd6b0
// PROTOTYPE: void __thiscall OnBeenAttacked(tagAttackInformation * param_1)
//
// Vtable CBuild slot `+0x1B0` — новый virtual CBuild (vtable CMoveShape
// заканчивается раньше и такого slot не имеет); CCityGate slot `+0x1B0`
// содержит то же тело `0x001DD6B0` (no-op у ворот стоит только в OnDied slot
// `+0x178`; `+0x180` ворот — реальный `CCityGate::OnBeenHurted` `0x001DDD80`,
// сохраняющий атакующего в region `+0x2B8/+0x2BC`).
// Машинный стержень (`0x005DD6B0`..`0x005DD9CD`):
// 1. Guard `this->+0x40 (m_pRegion) != 0`, иначе мгновенный `ret 4`.
// 2. `m_pRegion->FindChildObject(att+8, att+0xC, CGUID::GUID_INVALID)` через
//    region slot `+0x1C` (`0xEF3D9C` = статический `CGUID::GUID_INVALID`,
//    pub `?GUID_INVALID@CGUID@@2V1@A`); результат приводится
//    `__RTDynamicCast` `CBaseObject*`→`CMoveShape*` (type_info `.?AVCBaseObject@@`
//    @`0x69E6B0` → `.?AVCMoveShape@@` @`0x69E64C`); далее обязателен
//    `this->IsAttackAble(attacker)` (slot `+0x134`).
// 3. `std::vector<tagDamage*> v; this->ApplyFinalDamage(att, &v)` (slot
//    `+0x160` = `CBuild::ApplyFinalDamage` `0x001DD270`).
// 4. Ветвление по `CMoveShape::IsDied()` (`0x000CCF20`, читает virtual GetHP
//    slot `+0xD0` и возвращает 1 при HP==0):
//    - жив и вектор не пуст: `CMessage(0xBF60A)` — Add(M)(att+8),
//      Add(M)(att+0xC), Add(M)(type `this+4`), Add(M)(id `this+8`),
//      Add(D)((char)count), на каждую запись вектора Add(D)((char)dmg+0) и
//      Add(M)(dmg+4), затем Add(M)(GetHP()), Add(D)(att+0x10),
//      Add(D)(att+0x11), Add(M)(att+0), Add(E)(att+4);
//      `SendToAround(this, 0)` (`0x00014970`); затем virtual
//      `OnBeenHurted(att+8, att+0xC)` (slot `+0x180`; у CBuild общий пустой
//      `ret 8` `0x000A8750`). Пустой вектор — без пакета и без OnBeenHurted.
//    - мёртв: сначала virtual `OnBeenMurdered(att+8, att+0xC)` (slot `+0x17C`,
//      тот же пустой thunk), затем `CMessage(0xBF60B)` — те же 4 dword,
//      условно один Add(M)(dmg+4) первой записи вектора с `dmg+0 == 0` (ни
//      одной подходящей → поле пропускается), Add(D)(1), Add(D)(att+0x10),
//      Add(D)(att+0x11), Add(M)(att+0), Add(E)(att+4);
//      `SendToAround(this, 0)`; `CMoveShape::SetKilledMeAttackInfo(att)`
//      (`0x000CCE50`: копии att+8→`+0x180`, att+0xC→`+0x184`, att+0x30→`+0x1A8`,
//      att+0x34→`+0x1AC`, att+0x38→`+0x1B0`, att+0x14/+0x18/+0x1C/+0x20 →
//      `+0x18C/+0x190/+0x194/+0x198`, байты att+0x10→`+0x188`,
//      att+0x28..+0x2B→`+0x1A0..+0x1A3`); virtual `SetAction(6)` (slot `+0x80`).
//      Записи tagDamage и буфер вектора освобождаются `operator delete`
//      (`0x0021770D`).
//    `Add(M)` = `CBaseMessage::Add(float)` (`0x00013240`) дописывает 4 сырых
//    little-endian байта аргумента; `Add(D)` (`0x00013110`) и `Add(E)`
//    (`0x00013170`) по 1 байту (quirk: при исчерпании буфера байт молча не
//    пишется, счётчик растёт); `CMessage(long)` (`0x000136D0`) кладёт ID в
//    header+4. Семантика полей `tagAttackInformation` по именам не
//    установлена (только смещения). OnDied отсюда не вызывается.
// Вызыватели: ни одного `call [reg+0x1B0]` в `.text` (для смещения `0x1B0`
// кодировка всегда disp32, скан исчерпывающий), ни одного прямого `E8` на
// `0x001DD6B0`; ссылки на тело — только два vtable slot (CBuild `+0x1B0`,
// CCityGate `+0x1B0`). Тело недостижимо в этой сборке; реальный урон
// постройкам идёт двухаргументным слотом `+0x15C`
// `CMoveShape::OnBeenAttacked` (`0x004D2890`).
//
//

// ============================================================================
// FUNCTION: CBuild::OnDied
// STATUS: VERIFIED_DISASSEMBLY (тело и цепочка вызовов); достижимость у
// постройки в этой сборке не подтверждается: единственный вызыватель требует
// CBaseAI, которого у CBuild нет
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:214
// RVA: 0x001DD9D0
// ADDRESS: 005dd9d0
// PROTOTYPE: void __thiscall OnDied(void)
//
// Vtable CBuild slot `+0x178`; CCityGate в том же slot держит общий пустой
// thunk (VA `0x00485540`) — см. `??_7CCityGate@@6B@` (`0x0065E8C4`).
// Машинный стержень (`0x005DD9D0`..`0x005DDAF9`):
// 1. Guards: `m_pRegion (this+0x40) != 0` и `this->+0x180 == 0x190` — тип
//    убийцы из kill-инфо (записывает `SetKilledMeAttackInfo`, см. блок
//    OnBeenAttacked); `0x190` = тип объекта CPlayer (ctor CPlayer пишет
//    `[this+4] = 0x190`, RVA `0x58F1F`).
// 2. `killer = m_pRegion->FindChildObject(0x190, this->+0x184,
//    CGUID::GUID_INVALID)` (region slot `+0x1C`).
// 3. `m_pRegion->OnSymbolDestroy(this->id (+8), this->+0x184, this->+0x1AC,
//    this->+0x1B0)` (slot `+0x68`) — в этой сборке тело пустое
//    (`ret 0x10`, `0x001CED50`) у всех проверенных region-классов
//    (CServerRegion, GodsBattle, Nation, Country, City, Village, War — по их
//    vtable); вызывается в любом случае до проверок script-запуска.
// 4. `if (!killer) return; if (m_ScriptFile пуст) return;` — MSVC-строка
//    постройки: buffer `+0x1F0`, size `+0x200`, capacity `+0x204`; тело
//    различает SSO/heap-вариант буфера по capacity ≥ `0x10`.
// 5. `stRunScript rs` (ctor `0x00024AB0`): rs+0 = region, rs+4 = killer,
//    rs+8 = 0, rs+0xC = CGUID, rs+0x20 = `m_ScriptFile`;
//    `data = CGame::GetScriptFileData(m_ScriptFile)` (`0x00028C00`);
//    `RunScript(&rs, data, 0)` (`0x00028840`; сторожа rs/[rs+4]/data и
//    ScriptIfExit). SetAction, HP и CountryWarSys тело не вызывает.
// Вызыватели: единственный сайт slot `+0x178` в `.text` —
// `CBaseAI::OnBeenKilled` (`0x000C9220`, сайт `0x000C9303`): тот вызывает
// OnDied owner-shape (поле `+0x68` AI) только при опустевшей AI-очереди
// событий; прямых `E8` на `0x001DD9D0` нет; для смещения `0x178` кодировка
// всегда disp32, скан исчерпывающий. Конструктор CMoveShape обнуляет
// `+0xE8 (m_pAI)`, AI создают CPlayer-ctor и `CMonster::InitAI`; CBuild AI не
// создаёт, поэтому script-ветка постройки в этой сборке недостижима.
// Country-war: `CountryWarSys::on_flag_destroy(long)` (`0x000EBE60`)
// вызывается только из `OnCountryMessage` (сайт `0x0009A2A8`) — отдельный
// message-driven вход; из тела OnDied путь в CountryWarSys не выходит, что
// машинно подтверждает «победу обрабатывает CountryWarSys отдельно».
//
//


// COMPONENT_VARIANT_END: GameServer
