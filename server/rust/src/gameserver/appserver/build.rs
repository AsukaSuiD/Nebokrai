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
