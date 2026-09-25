//! Общий формат обмена `CMonsterList` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    MonsterDropRegistry, MonsterListDecodeError,
    MonsterProperties, MonsterRegistry, MonsterSkill,
    decode_monster_list, get_monster_property_by_origin_index, get_monster_property_by_origin_name,
    get_monster_property_by_origin_name_mut, get_monster_property_by_picture_id,
    load_drop_goods_list, load_monster_list, serialize_monster_list,
};
