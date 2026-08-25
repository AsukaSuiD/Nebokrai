//! Достигнутая send-family проекция `CPlayer` исторического GameServer.
//!
//! PDB `GameServer/GameServer.pdb` подтверждает base `CMoveShape +0x0` и signed
//! `m_lTeamID +0xB20`, а также unsigned byte `m_btCountry +0xA5C`. Exact
//! `CMessage::SendToAround` RVA `0x00014420` и
//! `SendToRegionContryPlayer` RVA `0x00014760` читают inherited
//! `CBaseObject::m_lID +0x8` как numeric map/player identity, team ID и country
//! после RTTI `CShape/CMoveShape -> CPlayer`. Эти достигнутые поля имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; исходники
//! `server/gameserver/appserver/player.h/.cpp`.
//! Organizing identity `m_lFactionID/m_lFacMasterID` обновляется из полного
//! World `0x7FE06` wire; `IsFactionMaster` сохраняет exact positive-faction и
//! player-ID equality contract.
//! Полный World→Game `0x7F901` handoff теперь декодирует единый
//! `CPlayer::DecordFromByteArray(..., true)` layout: shape/base/combat,
//! skills/states, containers, variables, timers, companions, quests, country,
//! organization и session. Обратный `AddGameSaveToByteArray` использует те же
//! owned поля и live pet/carriage snapshot; container codec failure прекращает
//! wire строго на первом false, как исходная цепочка. После чтения base-wire
//! `bBFSummon` намеренно снова выводится из локального `m_dwWarSoulState`, а не
//! принимается как независимый persisted fact.
//! Initial-login tail обходит GoodsAI candidates в exact positional order:
//! equipment, packet, hand, auction и depot. Для equipment-state `2` нулевая
//! packed date прерывает только текущий container; просроченное состояние
//! становится `3` до old-client `0xBF928`, как в `OnLogMessage`.
//! `CMessage::Run` RVA `0x000149D0` дополнительно читает inherited father
//! `+0x40` как текущий `CServerRegion*`; удалённый raw pointer выражен
//! `Option<i32>` region identity в assembly-проекции.
//!
//! Материализована также подтверждённая setter-family: боевые scalar-ы
//! насыщаются до `INT_MAX`, contribution — до `±2_000_000_000`, а fetch power
//! сравнивается с unsigned-представлением setup limit. Это минимальный owned
//! player state для будущих equipment/battle-fairy side effects, но не замена
//! полного constructor-а, property recalc или runtime player lifecycle.
//! World kill confirmation `0x7F806` materializes `wPkCount`, `dwKillCount` и
//! murderer timestamp: PK насыщается до `0xFFFF`, kills wrapping-инкрементятся,
//! а clock читается только при первом ненулевом murderer state.
//! FourNation reward `0x7FE46` добавляет owned `dwExploit`: advertised client
//! value сохраняет wrapping addition, `SetExploit` отдельно применяет exact
//! unsigned CountryParam maximum, а virtual `UpdateProperty` остаётся
//! обязательным caller-runtime effect после мутации.
//! Reached script property catalog отделён от gameplay setter-ов: generic
//! `SetValue/ChangeValue` сохраняет narrowing/wrapping storage, включая
//! shipped `Experience` alias и достигнутые honor
//! `dwAppellationID/dwRankOfNobilityID`; bool fairy enable pair нормализует
//! ненулевой write в persisted player state. `GetValue` читает также credit,
//! SZL и contribution из canonical storage. Пересчёт и
//! `0xBF721` остаются у вызывающего `CGame`.
//! Total honor-rank startup материализует days/weeks/months counters и
//! nobility rank: reset меняет owned state, а пока RAW `PlayerRunScript`
//! выражен точным typed AdjustHonorRank script-effect-ом.
//! Silence-timeout, как и оригинал, проверяется лениво при query по
//! инъецируемому wrapping `timeGetTime`-значению; GM `0x7FC0B/0x7FC0E`
//! замыкают name lookup, mutation, двухпроходный ordered query и World
//! responses, поэтому отдельный scheduler не требуется.
//! Client allocation `0x8FA01` владеет sex/occupation, remaining point,
//! четырьмя base stat и base HP/MP maxima. Сохранены общий STR gate для всех
//! `Add*`, безусловный расход очка и отдельный 0x9c-byte `m_Property` wire:
//! reached recompute заменяет только подтверждённые поля, не обнуляя хвост.
//! PvP preferences `0x8FA05` хранят пять live permission flags, которые
//! downstream player/skill AI читает при выборе обычных, team, union,
//! criminal и country целей; unknown selector только потребляет вход.
//! Cross-Game progression `0x7FA08..0B` использует owned skill map и level/exp:
//! name-overload-ы делегируют factory ID lookup, а `SetLevel` возвращает
//! faction side effect caller-у до exact client progression packet.
//! World/country public talk timestamps принадлежат тому же player state:
//! wrapping cooldown обновляется до проверки и списания channel-cost.
//! Текущие HP/MP имеют собственные setter-и с clamp к текущим max-свойствам;
//! изменение самих max не выполняет этот clamp без конкретного caller-а.
//! `UseItem` материализует exact requirement-коды, owned skill learning,
//! packet consumption и четыре replaceable `tagExpendableEffect` combat
//! mutation. Mount и ChangeBody guards/state замкнуты на canonical player/game
//! owners; timed `CState`, recall и неподдержанные script VM selector-ы остаются
//! у caller runtime.
//! Как в связном `RefreshContainerOwners`, достигнутые equipment,
//! ordinary-fairy и battle-fairy containers принадлежат player type `400` с
//! его numeric ID. Ordinary fairy получает exact volume 14 и persisted
//! enable/vigour/experience; persisted battle-fairy enable декодируется из
//! World base-property offset `0x128`, а `CanMountEquip` использует оба enable
//! flag-а, headgear addon и live requirements без внешнего result snapshot.
//! Periodic hatcher caller замкнут через `CGame`;
//! Hotkey owner хранит exact 24 DWORD и связывает назначение с возвратом
//! consumable из hand в packet/hand/wallet/YuanBao; equipment destination
//! проходит исходный remove→failed add→hand rollback без потери ownership.
//! Enhancement/precious-box confirm хранит server-trusted container-script
//! path у игрока; отмена очищает только shadow selection без переноса goods.
//! Remote equipment inspection использует owned persisted head/face/mode и
//! тот же live equipment container, не отдельный display snapshot.
//! Depot-password vertical дополнительно материализует `m_eProgress`, оба
//! changing-guard-а, password byte-string и owned `CBank/CDepot`; numeric
//! значения внутреннего `eProgress` не выходят в wire и потому заменены typed
//! enum без выдуманного `repr`.
//! Exact `GetWarSoulGoods` читает headgear cell 10 и признаёт её боевой феей
//! только при addon `GAP_BF_BATTLE_FAIRY` value-id 1, равном единице.
//! `BatllteFairyCombine` соединяет container inputs, global BattleFairy gate,
//! fetch power, shared Game RNG/factory, `CMoveShape::AddSkill` и ordered
//! адресные object/skill/goods/audit effects. `BTreeMap` skill storage в
//! `CMoveShape` заменяет четыре pointer-vector-а только для общего confirmed
//! identity/level/type/name state; выполнение concrete skill owners не
//! перенесено сюда. Результат combine кладётся в обычную ячейку `Battle`, а
//! не в gear-ячейку, поэтому исходный owner доказательно не вызывает здесь
//! `BFPropertyAdd`, equipment mutation или `PropertiesChanged`. Account для
//! audit принадлежит player snapshot и пока заполняется отдельным caller-ом
//! при восстановлении player identity.
//! Script revive боевой феи восстанавливает HP/MP из maxima и атомарно меняет
//! recall/died/summon/WarSoul state; goods и properties wire публикует CGame.
//! Поэтому `from_send_state` остаётся явной assembly-границей уже
//! восстановленного runtime. Figure передаётся как доказанный derived virtual
//! fact; владение spatial state остаётся у `CMoveShape`.
//! `SummonBF` RVA `0x00101CB0` материализован единым Player→CGame→region
//! проходом: guards, summon/recall state, ordered around effects и area-map
//! action. Active pet пока count-derived fact; codec и goods-message decoder
//! остаются явной границей и report не подменяет исторические packet bytes.
//! `BFPropertyAdd` соединяет восемь gear-ячеек с headgear battle fairy,
//! `GlobeSetup` occupation coefficients и player combat state. Сохранены
//! ранний effect до результата Add, post-remove `-1`, clamp текущих HP/MP,
//! двойное применение MaxHP/Str/Int/Dex и двойной `0xBF918` в Remove.
//! Goods-message `0x8FC2A` материализован до ordered potential mutation:
//! aggregate guard остаётся в клиентских единицах, отдельные allocation
//! умножаются на `10000`, одинаковые property keys имеют `std::map` first-win,
//! а каждый вызов и outer caller публикуют собственный `0xBF918`.
//! Gear add/remove теперь через `CGame` действительно исполняет ordered
//! `0xBF721/0xBF918`; remove сохраняет две одинаково обязательные публикации
//! old-client payload после успешного `BFPropertyAdd(-1)`.
//! Upgrade `0x8FC28` замыкает validation, owned wallet goods, общий RNG,
//! factory level/growth mutation, target failure outcome, positional расход
//! gem-ов и ordered client/audit effects. Снимки target/gems/player сохраняют
//! World audit после необратимого удаления, а `CGame` публикует concrete
//! `0xC0101/0xC0102`, `0xBF918` и `0x60202/0x60203` в исходном порядке.
//! `ResetPotential` использует owned packet `CVolumeLimitGoodsContainer` 8×12:
//! первый `ZHQLS01` расходуется до addon/player mutation, семь tracked-вкладов
//! возвращаются в общий potential и публикуется один итоговый `0xBF918`.
//! `ResetSkill` соединяет equipment headgear, optional packet-reset item,
//! общий Game RNG, exact несовместимые пары, полный detach/attach девяти
//! war-soul skills и подтверждения `0xBF71D/0xBF918`.
//! Goods-message `0x8FC29` использует отдельный script reset: player owner
//! сохраняет native detach/attach девяти addon skills вокруг live script,
//! не подменяя его внутренней random-веткой `ResetSkill`.
//! `GetGoodsById` теперь сохраняет exact hand→packet→equipment→auction lookup;
//! hand и auction являются owned containers и участвуют в owner refresh.
//! Однослотовый `m_cEnhancementContainer` хранит shadow выбранного исходного
//! goods и даёт script ID 9351 тот же живой предмет без копии. Входящий
//! `0x90301` проверяет packet/equipment position, GUID, amount и stackability,
//! затем записывает shadow без смены ownership исходного goods и сохраняет
//! native last-operated source для последующих container-переходов.
//! Script `2249` использует тот же live owner; ripe replacement добавляется
//! напрямую в packet даже пока script progress занят.
//! Двусторонний auction-listing route использует те же owned packet/equipment
//! containers: обратный ход считает exact equipment+packet+hand burden,
//! сохраняет last-operated только после успешного destination add и оставляет
//! temporary auction/fairy/session goods вне весовой суммы, как исходный owner.
//! CiQing unlocked base-index set хранится ordered `BTreeSet`; его query не
//! создаёт постоянные goods, а только передаёт snapshot CGame factory owner-у;
//! make считает/удаляет packet stack-и в container order и сохраняет
//! new-object/stack ownership; `CGame` публикует concrete `0xC0101/02` и
//! World audit `0x60218`. Owned CiQing containers имеют exact volumes `8/3`;
//! compose slots удаляются по позиции.
//! Основной CiQing delete сохраняет partial-amount семантику `DeleteGoods`.
//! Hand mount читает exact addon `243/244`; hand consumption также сохраняет
//! partial amount и не выдаёт reached `CGoods` projection за полный Clone.
//! CiQing property owner хранит ordered обычные/TaoZhuang map-ы и set ID.
//! `UpdateCiQingProperty` сопоставляет равные по размеру ordered снимки и
//! насыщает отрицательную разницу нулём; merge для клиента сохраняет unsigned
//! wrapping addition. Other-person snapshot читает это состояние и те же
//! восемь owned CiQing slots без копий. Универсальные equipment/addon формулы
//! остаются обязательной runtime-границей до materialization всех combat scalar-ов.
//! `skillmessage 0x90001` сохраняет learned-skill authorization, contend
//! notice, безусловное обнуление emotion state, self/point/object target и
//! socket reject; `0x90005` добавляет feature/HP guards и странный fallback
//! `546/547`. Concrete `CPlayerAI`, region symbol rule и полный region object
//! registry передаются как explicit facts.
//! Item-skill `0x90004` использует тот же player route с client-provided level
//! и добавляет ID в owned ordered `CMoveShape` vector только перед AI dispatch.
//! Shape commands сохраняют owned direction и emotion index/timestamp:
//! ClearEmotion всегда обнуляет оба поля, PerformEmotion делает это до guards
//! и запоминает repeated ID/time только при разрешённом живом AI owner-е.
//! Client relocation использует общие movement facts и `CShape` owner через
//! `CServerRegion`; caller сохраняет исходный `BF603 -> SetTileXY -> GS0163`
//! порядок и contend/symbol predicate, поэтому замещённый RAW удалён.
//! Quest movement также использует concrete `OnCannotMove` wire с текущими
//! tile coordinates; player-AI caller очищает emotion перед постановкой шага.
//! Friend owner хранит исходный ordered список до 40 byte-exact имён и online
//! flag; message caller замыкает reciprocal mutation, World persistence и
//! addressed client result, поэтому `AddFriend/DelFriend` RAW удалён.
//! Public identity owner хранит headpiece/appellation/honor state; country job
//! вычисляется concrete `CCountry`, а change request записывает attempt ID до
//! вызова server-trusted script owner-а.
//! Client timing owner хранит quest countdown и heartbeat acknowledgement:
//! остаток сохраняет signed 32-bit arithmetic исходного `time_t`, а wall/local
//! clock остаются внешними runtime-фактами message caller-а.
//! Player quest lifecycle хранит persisted `ushort → complete byte`: accept,
//! complete и disband публикуют `0xBFF2C/2D/2E`, а `0xBFF2F` position остаётся
//! transient client hint и не создаёт второго авторитетного quest state.
//! LeiTing owner хранит пять scalar-полей и ordered `tagThing` list; codec
//! совпадает с WorldServer `Add/DecodeByteArrayLeiTing`, а reward-флаг
//! выставляется только после exact energy/count threshold. Script `2650/2651`
//! работает с тем же списком: успешное увеличение добавляет `point * delta`
//! к энергии, применяет суточный порог `60` и публикуется единым snapshot.
//! Goods-session `0x8FC25` использует полный typed `eProgress` owner и
//! сбрасывает его в `None`, одновременно снимая один nesting moveable-запрет;
//! полиморфные session End/plug Exit принадлежат caller runtime-у.
//! Nation-war player lifecycle связывает exact `SetContendState`,
//! `OnDied`/`OnRelive` и millisecond-tail `PeriodicalUpdate`: owned state
//! хранит три PDB-поля `+0xBA5/+0xBA8/+0xBAC`, а конкретные self/around
//! маршруты сообщений остаются у `CGame`, владеющего network/session runtime.
//! Periodic `ComputeWarSoulXY` сохраняет float follow-state, exact dead/snap
//! thresholds, общий area-map tail и последующий `0xBF605`; restored-state
//! concrete skill остаётся входным фактом. Non-finite повреждённый float-state
//! блокируется typed outcome до старого x87 integer conversion.
//! Periodic HP-death prefix `CPlayer::AI` повторно нормализует summon/state и
//! recall/died флаги нулевой по HP equipped fairy, затем вызывает
//! `PropertiesChanged`; `CGame` собирает exact `0xBF721` из owned combat/base
//! полей, оставляя runtime facts только для ещё не сведённых add-element-
//! attack, RP/max-vigour и exalt scalar-ов.
//! Оригинал в этой ветви не чистит stale area-map entry и не посылает status
//! broadcast; оба отсутствующих side effect сохранены.
//! `CEquipmentContainer::OnObjectRemoved` player-tail связывает снятие
//! headgear с exact `SetWarSoulStaus(0)`, девятью skill detach, пересчётом
//! свойств при уже отсутствующем slot-е, HP/MP clamp и `0xBF720`. Полный
//! virtual property owner остаётся injected callback-границей.
//! GodsBattle player snapshot теперь также хранит persisted faction/SZL;
//! faction membership появляется только в concrete region AddObject-tail и
//! удаляется его RemoveObject/DelObj-tail, не при восстановлении snapshot-а.
//! `UpdateSZL` проходит через `CGame`: player property/notice предшествуют
//! decrease-only appellation check и script-effect-у `RequestChangeAppellation`.
//! Симметричный `OnObjectAdded` сохраняет late-block partial mutations, после
//! commit добавляет девять war-soul skills, пересчитывает свойства, публикует
//! `0xBF720` с исключением owner-а и отражает даже zero-delta `PackExpand` log.

use super::area::WarSoulPoint;
use super::container::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCodecError, AmountLimitGoodsRemoved,
    AmountLimitGoodsTaken, CAmountLimitGoodsContainer,
};
use super::container::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::container::cbank::CBank;
use super::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck, BattleFairyCombineRemovedInput,
    BattleFairyContainerAddOutcome, BattleFairyDefaultGoodsUpdate, BattleFairyDefaultSkill,
    BattleFairyPropertyAddEffect, BattleFairyUpgradeConsumedGem, CBattleFairyContainer,
};
use super::container::ccontainer::ContainerListenerHandle;
use super::container::ccontainer::PreviousContainer;
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{
    CEquipmentContainer, EquipmentAddOutcome, EquipmentAddRuntimeFacts, EquipmentAroundUpdate,
    EquipmentColumn, EquipmentContainerCodecError, EquipmentOwnerPlayerFacts,
    EquipmentRemoveOutcome, EquipmentRemoveRuntimeFacts, EquipmentUnserializedEntry,
};
use super::container::cfairycontainer::{CFairyContainer, FairyContainerCodecError};
use super::container::cgoodscontainer::GoodsStackMergeOutcome;
use super::container::cgoodsshadowcontainer::{PlacedShadowGoods, ShadowRecordBlock};
use super::container::cjifen::CJiFen;
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome, VolumeGoodsCodecError,
    VolumeGoodsRemoveOutcome,
};
use super::container::cwallet::{
    CWallet, CurrencyCodecError, CurrencyDecreaseOutcome, CurrencyGoodsAddOutcome,
    CurrencyIncreaseOutcome,
};
use super::container::cyuanbao::CYuanBao;
use super::goods::cbattlefairyproperty::BattleFairyCompose;
use super::goods::cgoods::CGoods;
use super::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_AGILITY_CORRECTION, GAP_ARMOR_CORRECTION, GAP_ATTACK_AVOID,
    GAP_ATTACK_SPEED_CORRECTION, GAP_BF_ABRAVE_ADDON, GAP_BF_AGILITY, GAP_BF_AGILITY_ADDON,
    GAP_BF_AGILITY_POTENTIAL, GAP_BF_ALL_SKILL, GAP_BF_ATTACK, GAP_BF_ATTACK_ADDON,
    GAP_BF_ATTACK_POTENTIAL, GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST, GAP_BF_BLAST_ADDON,
    GAP_BF_BLAST_POTENTIAL, GAP_BF_BRAVE, GAP_BF_BRAVE_POTENTIAL, GAP_BF_CUT_HURT_ADDON,
    GAP_BF_CUT_HURT_SCALE, GAP_BF_EARTH, GAP_BF_EARTH_SKILL, GAP_BF_HP, GAP_BF_HUOXIESHU_SKILL,
    GAP_BF_LIFE_ADDON, GAP_BF_LINGZHISHU_SKILL, GAP_BF_MAN, GAP_BF_MAN_SKILL, GAP_BF_MAX_HP,
    GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_MP_ADDON, GAP_BF_POTENTIAL, GAP_BF_SKY, GAP_BF_SKY_SKILL,
    GAP_BF_SPRITE, GAP_BF_SPRITE_ADDON, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITUALISE_ADDON,
    GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_STRENGH, GAP_BF_STRENGH_ADDON,
    GAP_BF_STRENGH_POTENTIAL, GAP_BF_WEAPON_LEVEL, GAP_BLAST_ATTACK, GAP_BLAST_ELEMENT_ATTACK,
    GAP_BURDEN_UPPER_LIMIT_CORRECTION, GAP_CIQING_PROPERTY1, GAP_CIQING_PROPERTY2,
    GAP_CONSTITUTION_CORRECTION, GAP_DODGE_CORRECTION, GAP_ELEMENT_ATTACK_CORRECTION,
    GAP_ELEMENT_AVOID, GAP_ELEMENT_RESISTANCE_CORRECTION, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_FULL_MISS, GAP_FUMO_PROPERTY, GAP_GEM_LEVEL, GAP_GOODS_BIND, GAP_GOODS_PACKAGE_EXTENTION,
    GAP_HIT_RATE_CORRECTION, GAP_HP_RESTORE_SPEED_CORRECTION, GAP_HP_UPPER_LIMIT_CORRECTION,
    GAP_MAXIMUM_ATTACK_CORRECTION, GAP_MINIMUM_ATTACK_CORRECTION, GAP_MOUNT_LEVEL, GAP_MOUNT_TYPE,
    GAP_MP_RESTORE_SPEED_CORRECTION, GAP_MP_UPPER_LIMIT_CORRECTION, GAP_REQUIRE_GENDER,
    GAP_REQUIRE_OCCUPATION, GAP_ROLE_MINIMUM_AGILITY_LIMIT, GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
    GAP_ROLE_MINIMUM_LEVEL_LIMIT, GAP_ROLE_MINIMUM_STRENGTH_LIMIT, GAP_ROLE_MINIMUM_WAKAN_LIMIT,
    GAP_STIFFEN_PROBABILITY_CORRECTION, GAP_STRENGTH_CORRECTION, GAP_WAKAN_CORRECTION,
    GOODS_TYPE_CONSUMABLE,
};
use super::goods::cgoodsfactory::CGoodsFactory;
use super::moveshape::{
    CMoveShape, MoveShapeCommandBlock, MoveShapeCommandContext, MoveShapePositionFacts,
    MoveShapeSkill,
};
use super::script::variablelist::{
    CVariableList, GameVariableMutationOutcome, GameVariableSnapshotError,
};
use super::serverregion::CServerRegion;
use super::shape::{
    CShape, ShapeCoordinateBlock, ShapeDecodeError, ShapeFigure, ShapeIdentity, ShapeView,
};
use super::skills::skillfactory::{CSkillFactory, UNKNOWN_SKILL_ID};
use crate::nets::netserver::message::GameServerAroundRuntime;
use crate::public::auctionnode::CGoodsNode;
use crate::public::guid::CGuid;
use crate::setup::globesetup::GlobePlayerPropertyCoefficients;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const PLAYER_TYPE: i32 = 400;
const PLAYER_BASE_PROPERTY_WIRE_SIZE: usize = 0x194;
const BASE_LEVEL_OFFSET: usize = 0x04;
const BASE_EXPERIENCE_OFFSET: usize = 0x08;
const BASE_HEAD_PICTURE_OFFSET: usize = 0x0c;
const BASE_FACE_PICTURE_OFFSET: usize = 0x0d;
const BASE_OCCUPATION_OFFSET: usize = 0x0e;
const BASE_SEX_OFFSET: usize = 0x0f;
const BASE_PK_COUNT_OFFSET: usize = 0x1c;
const BASE_KILL_COUNT_OFFSET: usize = 0x20;
const BASE_CHARGED_OFFSET: usize = 0x38;
const BASE_REMAIN_POINT_OFFSET: usize = 0x3a;
const BASE_HOTKEY_OFFSET: usize = 0x3c;
const BASE_PK_NORMAL_OFFSET: usize = 0x9c;
const BASE_PK_TEAM_OFFSET: usize = 0x9d;
const BASE_PK_UNION_OFFSET: usize = 0x9e;
const BASE_PK_BADMAN_OFFSET: usize = 0x9f;
const BASE_PK_COUNTRY_OFFSET: usize = 0xa0;
const BASE_HEALTH_OFFSET: usize = 0xa4;
const BASE_MANA_OFFSET: usize = 0xa8;
const BASE_MAXIMUM_HP_OFFSET: usize = 0xb0;
const BASE_MAXIMUM_MP_OFFSET: usize = 0xb4;
const BASE_STRENGTH_OFFSET: usize = 0xbc;
const BASE_DEXTERITY_OFFSET: usize = 0xc0;
const BASE_CONSTITUTION_OFFSET: usize = 0xc4;
const BASE_INTELLIGENCE_OFFSET: usize = 0xc8;
const BASE_VIGOUR_OFFSET: usize = 0xec;
const BASE_CREDIT_OFFSET: usize = 0xfc;
const BASE_DISPLAY_HEAD_PIECE_OFFSET: usize = 0x104;
const BASE_QUEST_TIME_BEGIN_OFFSET: usize = 0x108;
const BASE_QUEST_TIME_LIMIT_OFFSET: usize = 0x10c;
const BASE_QUEST_ENABLED_OFFSET: usize = 0x110;
const BASE_EXPLOIT_OFFSET: usize = 0x114;
const BASE_FAIRY_CONTAINER_ENABLED_OFFSET: usize = 0x11c;
const BASE_BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x128;
const BASE_DAYS_HONOR_OFFSET: usize = 0x140;
const BASE_WEEKS_HONOR_OFFSET: usize = 0x144;
const BASE_MONTHS_HONOR_OFFSET: usize = 0x148;
const BASE_TOTAL_HONOR_OFFSET: usize = 0x14c;
const BASE_RANK_OF_NOBILITY_OFFSET: usize = 0x150;
const BASE_APPELLATION_OFFSET: usize = 0x154;
const BASE_MODE_OFFSET: usize = 0x158;
const BASE_FETCH_POWER_OFFSET: usize = 0x164;
const BASE_BATTLE_FAIRY_SUMMONED_OFFSET: usize = 0x16c;
const BASE_BATTLE_FAIRY_RECALL_OFFSET: usize = 0x16d;
const BASE_BATTLE_FAIRY_DIED_OFFSET: usize = 0x16e;
const BASE_FY_ENERGY_OFFSET: usize = 0x17c;
const BASE_FY_ENABLE_FLAGS_OFFSET: usize = 0x180;
const BASE_LT_UP_60_COUNT_OFFSET: usize = 0x184;
const BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET: usize = 0x186;
const BASE_LT_60_STAMP_OFFSET: usize = 0x188;
const BASE_SZL_OFFSET: usize = 0x18c;
const BASE_GODS_BATTLE_FACTION_OFFSET: usize = 0x190;
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;
const CONTRIBUTION_MINIMUM: i32 = -2_000_000_000;
const CONTRIBUTION_MAXIMUM: i32 = 2_000_000_000;
const BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE: u32 = 0x0b_f71d;
const BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE: u32 = 0x0b_f80c;
const BATTLE_FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0c;
const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;
const BATTLE_FAIRY_MOVE_MESSAGE_TYPE: u32 = 0x0b_f605;
const BATTLE_FAIRY_STATUS_MESSAGE_TYPE: u32 = 0x0b_f930;
const BATTLE_FAIRY_SUMMON_MESSAGE_TYPE: u32 = 0x0b_f92e;
const BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE: u32 = 0x0b_f71e;
const BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING: &str = "ZHGS0022";
const SKILL_EFFECT_MESSAGE_TYPE: u32 = 0x0b_fe01;
const SKILL_REJECT_REASON: u32 = 0;
const SKILL_REJECT_WAR_SOUL_REASON: u32 = 4;
const SKILL_REJECT_CODE: u8 = 0x0c;
const SKILL_POJIA: u32 = 530;
const SKILL_LEIMING: u32 = 543;
const SKILL_ID_MASK: u32 = i32::MAX as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyObjectMoveOperation {
    Delete,
    New,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyObjectMove {
    pub(crate) operation: BattleFairyObjectMoveOperation,
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) amount: u32,
    pub(crate) old_client_payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillAdded {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) skill_type: u32,
    pub(crate) skill_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAuditLog {
    pub(crate) string_id: &'static str,
    pub(crate) account: Vec<u8>,
    pub(crate) goods_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    FetchPowerChanged {
        message_type: u32,
        player_id: i32,
        subject_id: i32,
        property_name: &'static str,
        value: u32,
    },
    ObjectMove(BattleFairyObjectMove),
    SkillAdded(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    Audit(BattleFairyAuditLog),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineOutcome {
    FeatureDisabled,
    Rejected,
    InsufficientFetchPower,
    InputRemovalStopped,
    Failed,
    CreationFailed,
    CreationRejected,
    Created,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineDelivery {
    Player(i32),
    FetchPower(i32),
    ObjectMove(Vec<i32>),
    SkillAdded(i32),
    GoodsUpdated(i32),
    Audit,
}

#[must_use = "combine report содержит последовательность адресных packet/log effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyCombineOutcome,
    pub(crate) removed_inputs: Vec<BattleFairyCombineRemovedInput>,
    pub(crate) effects: Vec<BattleFairyCombineEffect>,
    pub(crate) deliveries: Vec<BattleFairyCombineDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyWarSoulAction {
    SetPosition {
        previous: WarSoulPoint,
        target: WarSoulPoint,
    },
    Delete {
        previous: WarSoulPoint,
        player_position: WarSoulPoint,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonOutcome {
    FeatureDisabled,
    AlreadySummoned,
    AlreadyRecalled,
    MissingHeadgear,
    InvalidHeadgear,
    NoHitPoints,
    ActivePet,
    MonsterTamingActive,
    CoordinateBlocked(ShapeCoordinateBlock),
    Summoned,
    Recalled,
    IgnoredMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    AroundMessage {
        message_type: u32,
        player_id: i32,
        values: Vec<i32>,
    },
    PropertiesChanged {
        player_id: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonDelivery {
    Player(i32),
    Around(Option<Result<i32, ShapeCoordinateBlock>>),
    Properties(i32),
}

#[must_use = "summon report хранит точный порядок адресных broadcast и property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySummonReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairySummonOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) spatial_applied: bool,
    pub(crate) effects: Vec<BattleFairySummonEffect>,
    pub(crate) deliveries: Vec<BattleFairySummonDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowOutcome {
    ActiveSkill,
    NotSummoned,
    CoordinateBlocked(ShapeCoordinateBlock),
    NonFiniteVisualState,
    InsideDeadZone,
    Moved,
    Snapped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowEffect {
    AroundMove {
        message_type: u32,
        player_id: i32,
        object_type: i32,
        x: u32,
        y: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowDelivery {
    Around(Option<Result<i32, ShapeCoordinateBlock>>),
}

#[must_use = "follow report содержит spatial tail и обязательный move broadcast"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyFollowReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyFollowOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) visual_x_bits: u32,
    pub(crate) visual_y_bits: u32,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) spatial_applied: bool,
    pub(crate) effects: Vec<BattleFairyFollowEffect>,
    pub(crate) deliveries: Vec<BattleFairyFollowDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyDeathOutcome {
    MissingHeadgear,
    NotBattleFairy,
    Alive,
    Died,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyDeathEffect {
    PropertiesChanged { player_id: i32 },
}

#[must_use = "death report сохраняет periodic state transition и property effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDeathReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyDeathOutcome,
    pub(crate) effects: Vec<BattleFairyDeathEffect>,
    pub(crate) property_delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) player_goods_package_extension: Option<u32>,
    pub(crate) active_war_soul_blocks_headgear: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentRemoveEffect {
    WarSoulStatusAround {
        message_type: u32,
        player_id: i32,
        values: [i32; 2],
    },
    WarSoulSkillDetached {
        skill_id: u32,
    },
    SkillRemoved(BattleFairySkillRemoved),
    PropertiesChangedWithoutRemovedSlot {
        column: EquipmentColumn,
        combat_properties: PlayerCombatProperties,
    },
    VitalsClamped {
        previous_health: u32,
        current_health: u32,
        previous_mana: u32,
        current_mana: u32,
    },
    AroundUpdate(EquipmentAroundUpdate),
}

#[must_use = "equipment remove report сохраняет container ownership и player/network tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentRemoveOutcome,
    pub(crate) effects: Vec<PlayerEquipmentRemoveEffect>,
    pub(crate) deliveries: Vec<PlayerEquipmentDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) now: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentAddEffect {
    WarSoulSkillAttached {
        skill_id: u32,
        level: i32,
    },
    SkillAdded(BattleFairySkillAdded),
    PropertiesChanged {
        combat_properties: PlayerCombatProperties,
    },
    AroundUpdate(EquipmentAroundUpdate),
    PackageExtensionLogged {
        category: &'static str,
        string_id: &'static str,
        expanded_package_num: u32,
    },
}

#[must_use = "equipment add report сохраняет partial mutations и player/network tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentAddOutcome,
    pub(crate) effects: Vec<PlayerEquipmentAddEffect>,
    pub(crate) deliveries: Vec<PlayerEquipmentDelivery>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentDelivery {
    SkillAdded(i32),
    SkillRemoved(i32),
    Runtime(Vec<i32>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationOutcome {
    Added(BattleFairyContainerAddOutcome),
    Removed(VolumeGoodsRemoveOutcome),
    MissingGoods,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationEffect {
    PropertiesChanged { player_id: i32 },
    BattleFairyUpdated(BattleFairyDefaultGoodsUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationDelivery {
    Properties(i32),
    GoodsUpdated(i32),
}

#[must_use = "equipment report сохраняет container ownership и ранние property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyEquipmentMutationReport {
    pub(crate) player_id: i32,
    pub(crate) cell: Option<BattleFairyCell>,
    pub(crate) delta: i32,
    pub(crate) property_applied: bool,
    pub(crate) outcome: BattleFairyEquipmentMutationOutcome,
    pub(crate) effects: Vec<BattleFairyEquipmentMutationEffect>,
    pub(crate) deliveries: Vec<BattleFairyEquipmentMutationDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationOutcome {
    MissingHeadgear,
    InvalidHeadgear,
    AggregateInsufficient,
    Processed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationDelivery {
    Player(i32),
    Properties(i32),
    GoodsUpdated(i32),
}

#[must_use = "allocation report сохраняет ordered player и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialAllocationReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialAllocationOutcome,
    pub(crate) aggregate_client_points: i32,
    pub(crate) processed_properties: Vec<i32>,
    pub(crate) effects: Vec<BattleFairyPotentialAllocationEffect>,
    pub(crate) deliveries: Vec<BattleFairyPotentialAllocationDelivery>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeLogGates {
    pub(crate) success: bool,
    pub(crate) failure: bool,
    pub(crate) lost_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeOutcome {
    MissingRegion,
    InsufficientMoney,
    InvalidEquipment,
    MissingBaseGem,
    GemLevelMismatch,
    MaximumLevel,
    Succeeded,
    FailedKept,
    FailedDowngraded,
    FailedReset,
    FailedDestroyed,
    ConsumptionStopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeGoodsSnapshot {
    pub(crate) identity: super::shape::ShapeIdentity,
    pub(crate) name: Vec<u8>,
    pub(crate) price: u32,
    pub(crate) amount: u32,
}

impl BattleFairyUpgradeGoodsSnapshot {
    fn capture(goods: &CGoods) -> Self {
        Self {
            identity: goods.identity(),
            name: goods.name().to_vec(),
            price: goods.price(),
            amount: goods.amount(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradePlayerSnapshot {
    pub(crate) pk_count: u16,
    pub(crate) money: u32,
    pub(crate) depot_money: u32,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) client_ip: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        format_value: Option<u32>,
    },
    MoneyChanged {
        player_id: i32,
        previous: u32,
        current: u32,
        outcome: CurrencyDecreaseOutcome,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    GemConsumed {
        player_id: i32,
        consumed: BattleFairyUpgradeConsumedGem,
    },
    TargetDeleted {
        player_id: i32,
        goods: BattleFairyUpgradeGoodsSnapshot,
        position: u32,
        removal: VolumeGoodsRemoveOutcome,
    },
    Audit {
        message_type: u32,
        event: u8,
        player_id: i32,
        player: BattleFairyUpgradePlayerSnapshot,
        target: BattleFairyUpgradeGoodsSnapshot,
        gems: [Option<BattleFairyUpgradeGoodsSnapshot>; 4],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeDelivery {
    Player(i32),
    Money(Vec<i32>),
    GoodsUpdated(i32),
    Container(Vec<i32>),
    Audit(Vec<i32>),
}

#[must_use = "upgrade report содержит wallet, ownership и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyUpgradeOutcome,
    pub(crate) price: u32,
    pub(crate) probability: u32,
    pub(crate) previous_level: Option<i32>,
    pub(crate) resulting_level: Option<i32>,
    pub(crate) consumed_gems: Vec<BattleFairyUpgradeConsumedGem>,
    pub(crate) effects: Vec<BattleFairyUpgradeEffect>,
    pub(crate) deliveries: Vec<BattleFairyUpgradeDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        position: Option<u32>,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetDelivery {
    Player(i32),
    PacketItem(Vec<i32>),
    Properties(i32),
    GoodsUpdated(i32),
}

#[must_use = "reset report содержит packet ownership и player/network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialResetReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialResetOutcome,
    pub(crate) recovered_potential: i32,
    pub(crate) effects: Vec<BattleFairyPotentialResetEffect>,
    pub(crate) deliveries: Vec<BattleFairyPotentialResetDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    InvalidPosition,
    SelectedSkillUnavailable,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRemoved {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    SkillRemoved(BattleFairySkillRemoved),
    SkillAdded(BattleFairySkillAdded),
    SelectedSkillLearned(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetDelivery {
    Player(i32),
    PacketItem(Vec<i32>),
    SkillRemoved(i32),
    SkillAdded(i32),
    SelectedSkillLearned(i32),
    GoodsUpdated(i32),
}

#[must_use = "skill reset report содержит packet, skill-state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillResetReport {
    pub(crate) player_id: i32,
    pub(crate) position: i32,
    pub(crate) outcome: BattleFairySkillResetOutcome,
    pub(crate) previous_skill: Option<u32>,
    pub(crate) selected_skill: Option<u32>,
    pub(crate) detached_skill_ids: Vec<u32>,
    pub(crate) attached_skill_ids: Vec<u32>,
    pub(crate) effects: Vec<BattleFairySkillResetEffect>,
    pub(crate) deliveries: Vec<BattleFairySkillResetDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSkillRequest {
    pub(crate) raw_skill_id: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
}

impl PlayerSkillRequest {
    pub(crate) const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Facts ещё сырых virtual owner-ов `CServerRegion::SymbolIsAttackAble`,
/// `CPlayer::GetAI` и полного player/monster region registry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerSkillRequestFacts {
    pub(crate) symbol_attackable: bool,
    pub(crate) player_ai_available: bool,
    pub(crate) object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillDispatch {
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
        target: super::shape::ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillRequestOutcome {
    Unauthorized,
    CoordinateBlocked(ShapeCoordinateBlock),
    AiUnavailable,
    MissingRegion,
    MissingTarget,
    Queued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillRequestEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        message_type: u32,
    },
    ClearEmotion,
    SocketReject {
        message_type: u32,
        reason: u32,
        code: u8,
    },
    AiDispatch(PlayerSkillDispatch),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillRequestDelivery {
    Player(i32),
    EmotionAround(Option<Result<i32, ShapeCoordinateBlock>>),
    SocketReject(i32),
    AiQueued,
}

#[must_use = "skill report содержит emotion, authorization, target и dispatch effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSkillRequestReport {
    pub(crate) player_id: i32,
    pub(crate) region_id: Option<i32>,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
    pub(crate) outcome: PlayerSkillRequestOutcome,
    pub(crate) effects: Vec<PlayerSkillRequestEffect>,
    pub(crate) deliveries: Vec<PlayerSkillRequestDelivery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequest {
    pub(crate) raw_skill_id: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) property_offset: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
}

impl BattleFairySkillRequest {
    pub(crate) const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Facts ещё сырых virtual owner-ов `CServerRegion::SymbolIsAttackAble`,
/// `CPlayer::GetAI` и полного player/monster region registry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequestFacts {
    pub(crate) symbol_attackable: bool,
    pub(crate) player_ai_available: bool,
    pub(crate) object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillDispatch {
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
        target: super::shape::ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillRequestOutcome {
    FeatureDisabled,
    MissingHeadgear,
    NoHitPoints,
    Unauthorized,
    CoordinateBlocked(ShapeCoordinateBlock),
    AiUnavailable,
    MissingRegion,
    MissingTarget,
    Queued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillRequestEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        message_type: u32,
    },
    SocketReject {
        message_type: u32,
        reason: u32,
        code: u8,
    },
    AiDispatch(BattleFairySkillDispatch),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillRequestDelivery {
    Player(i32),
    SocketReject(i32),
    AiQueued,
}

#[must_use = "war-soul skill report содержит authorization, target rewrite и dispatch"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequestReport {
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
    pub(crate) outcome: BattleFairySkillRequestOutcome,
    pub(crate) effects: Vec<BattleFairySkillRequestEffect>,
    pub(crate) deliveries: Vec<BattleFairySkillRequestDelivery>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BattleFairyGearAddons {
    attack: i32,
    sprite: i32,
    strength: i32,
    brave: i32,
    agility: i32,
    spiritualism: i32,
    blast: i32,
    cut_hurt: i32,
    life: i32,
    mana: i32,
}

impl BattleFairyGearAddons {
    fn read(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        let value = |property_type| goods.addon_property_value(factory, property_type, 1);
        Self {
            attack: value(GAP_BF_ATTACK_ADDON),
            sprite: value(GAP_BF_SPRITE_ADDON),
            strength: value(GAP_BF_STRENGH_ADDON),
            brave: value(GAP_BF_ABRAVE_ADDON),
            agility: value(GAP_BF_AGILITY_ADDON),
            spiritualism: value(GAP_BF_SPRITUALISE_ADDON),
            blast: value(GAP_BF_BLAST_ADDON),
            cut_hurt: value(GAP_BF_CUT_HURT_ADDON),
            life: value(GAP_BF_LIFE_ADDON),
            mana: value(GAP_BF_MP_ADDON),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) sex: u8,
    pub(crate) remain_point: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
    pub(crate) pk_normal: bool,
    pub(crate) pk_team: bool,
    pub(crate) pk_union: bool,
    pub(crate) pk_badman: bool,
    pub(crate) pk_country: bool,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) experience: u32,
    pub(crate) vigour: u32,
    pub(crate) credit: u32,
    pub(crate) charged: bool,
    pub(crate) fairy_container_enabled: bool,
    pub(crate) battle_fairy_enabled: bool,
    pub(crate) hotkeys: [u32; 24],
    pub(crate) mode: u32,
    pub(crate) display_head_piece: bool,
    pub(crate) quest_time_begin: i32,
    pub(crate) quest_time_limit: i32,
    pub(crate) quest_enabled: bool,
    pub(crate) fy_enable_flags: u32,
    pub(crate) fy_energy: u32,
    pub(crate) lt_60_stamp: u32,
    pub(crate) lt_up_60_count: u16,
    pub(crate) remain_jing_li_dan_count: u16,
    pub(crate) appellation_id: u32,
    pub(crate) head_picture: i32,
    pub(crate) face_picture: i32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
    pub(crate) fetch_power: u32,
    pub(crate) battle_fairy_recall: bool,
    pub(crate) battle_fairy_died: bool,
    pub(crate) days_honor_eliminate: u32,
    pub(crate) weeks_honor_eliminate: u32,
    pub(crate) months_honor_eliminate: u32,
    pub(crate) total_honor_eliminate: u32,
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) exploit: u32,
    pub(crate) gods_battle_faction: i32,
    pub(crate) szl: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorSnapshot {
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) appellation_id: u32,
    pub(crate) days_eliminate: u32,
    pub(crate) weeks_eliminate: u32,
    pub(crate) months_eliminate: u32,
    pub(crate) total_eliminate: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerFriend {
    pub(crate) name: Vec<u8>,
    pub(crate) online: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLeiTingThing {
    pub(crate) thing_id: u16,
    pub(crate) count: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLeiTingThingCountOutcome {
    Missing,
    Rejected {
        current: u16,
        requested: i32,
        maximum: u16,
    },
    Updated {
        previous_count: u16,
        current_count: u16,
        previous_energy: u32,
        current_energy: u32,
        daily_count_incremented: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerUncreatedPet {
    pub(crate) original_name: Vec<u8>,
    pub(crate) health: u32,
    pub(crate) level: u32,
    pub(crate) experience: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerUncreatedCarriage {
    pub(crate) original_name: Vec<u8>,
    pub(crate) script: Vec<u8>,
    pub(crate) health: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLeiTingDecodeBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerGameSaveCodecError {
    Shape(ShapeDecodeError),
    Goods(AmountLimitGoodsCodecError),
    Volume(VolumeGoodsCodecError),
    Equipment(EquipmentContainerCodecError),
    Fairy(FairyContainerCodecError),
    Currency(CurrencyCodecError),
    Variables(GameVariableSnapshotError),
    LeiTing(PlayerLeiTingDecodeBlock),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    NegativeCount {
        field: &'static str,
        count: i32,
    },
    StringTooLong {
        field: &'static str,
        length: usize,
        maximum: usize,
    },
    CollectionTooLarge {
        field: &'static str,
        length: usize,
    },
    WrongObjectType {
        object_type: i32,
    },
    EquipmentRejected {
        position: u32,
    },
    CodecReturnedFalse {
        field: &'static str,
    },
}

impl From<ShapeDecodeError> for PlayerGameSaveCodecError {
    fn from(value: ShapeDecodeError) -> Self {
        Self::Shape(value)
    }
}

impl From<AmountLimitGoodsCodecError> for PlayerGameSaveCodecError {
    fn from(value: AmountLimitGoodsCodecError) -> Self {
        Self::Goods(value)
    }
}

impl From<VolumeGoodsCodecError> for PlayerGameSaveCodecError {
    fn from(value: VolumeGoodsCodecError) -> Self {
        Self::Volume(value)
    }
}

impl From<CurrencyCodecError> for PlayerGameSaveCodecError {
    fn from(value: CurrencyCodecError) -> Self {
        Self::Currency(value)
    }
}

impl From<FairyContainerCodecError> for PlayerGameSaveCodecError {
    fn from(value: FairyContainerCodecError) -> Self {
        Self::Fairy(value)
    }
}

impl From<GameVariableSnapshotError> for PlayerGameSaveCodecError {
    fn from(value: GameVariableSnapshotError) -> Self {
        Self::Variables(value)
    }
}

impl From<PlayerLeiTingDecodeBlock> for PlayerGameSaveCodecError {
    fn from(value: PlayerLeiTingDecodeBlock) -> Self {
        Self::LeiTing(value)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerGameSaveDecodeReport {
    pub(crate) consumed_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoginGoodsLocation {
    Equipment,
    Packet,
    Hand,
    Auction,
    Depot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerFriendAddOutcome {
    Added,
    AlreadyPresent,
    LimitReached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerConfirmedKillReport {
    pub(crate) player_id: i32,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) murderer_timestamp_started: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteSkillMutation {
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteLevelMutation {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) previous_level: u8,
    pub(crate) level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyHandTransferOutcome {
    MissingHandGoods,
    NotConsumable,
    UnsupportedSource,
    Moved,
    RolledBack,
    GarbageCollected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandOwnershipEvent {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: Option<u32>,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "hand transfer report сохраняет ownership, fallback add и object-move исход"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandTransferReport {
    pub(crate) source_container_extend_id: u32,
    pub(crate) source_position: u32,
    pub(crate) goods: Option<ShapeIdentity>,
    pub(crate) hand_removal: Option<HotkeyHandOwnershipEvent>,
    pub(crate) packet_adds: Vec<VolumeGoodsAddOutcome>,
    pub(crate) currency_adds: Vec<CurrencyGoodsAddOutcome>,
    pub(crate) hand_rollback: Option<AmountLimitGoodsAdded>,
    pub(crate) outcome: HotkeyHandTransferOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorResetReport {
    pub(crate) player_id: i32,
    pub(crate) reset_mask: u32,
    pub(crate) previous_days: u32,
    pub(crate) previous_weeks: u32,
    pub(crate) previous_months: u32,
    pub(crate) adjust_honor_rank_script: Option<&'static [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorEliminateMutation {
    pub(crate) player_id: i32,
    pub(crate) previous: [u32; 4],
    pub(crate) current: [u32; 4],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExploitMutationReport {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) requested: u32,
    pub(crate) applied: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCountryMutationReport {
    pub(crate) player_id: i32,
    pub(crate) previous: u8,
    pub(crate) requested: i32,
    pub(crate) applied: u8,
    pub(crate) changed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurderCountersResetReport {
    pub(crate) player_id: i32,
    pub(crate) previous_pk_count: u16,
    pub(crate) previous_kill_count: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerCombatProperties {
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) constitution: u32,
    pub(crate) intelligence: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) attack_speed: u16,
    pub(crate) hit: u16,
    pub(crate) dodge: u16,
    pub(crate) cch: u16,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) hp_recovery: u16,
    pub(crate) mp_recovery: u16,
    pub(crate) burden: u16,
    pub(crate) reank: u16,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) full_miss: u16,
    pub(crate) element_modify: i32,
    pub(crate) blast_attack: u16,
    pub(crate) blast_element_attack: u16,
    pub(crate) blast_defense_scale_bits: u32,
    pub(crate) full_miss_scale_bits: u32,
    pub(crate) critical_rate_bits: u32,
}

/// Exact `GetPlayerAllProperties` diagnostic projection. Числа хранят raw
/// DWORD vararg bits: конкретный `%d`/`%u` шаблона определяет их signed view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAllPropertiesDiagnosticSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) summary_words: [u32; 15],
    pub(crate) base_combat_words: [u32; 15],
    pub(crate) current_combat_words: [u32; 20],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExpendableEffect {
    pub(crate) property_type: i32,
    pub(crate) value: i32,
    pub(crate) start_time_ms: u32,
    pub(crate) effect_time_ms: u32,
}

pub(crate) const PLAYER_COMBAT_PROPERTY_WIRE_SIZE: usize = 0x9c;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationState {
    pub(crate) sex: u8,
    pub(crate) occupation: u8,
    pub(crate) remain_point: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: u8,
    pub(crate) stat_changed: bool,
    pub(crate) previous: PlayerStatAllocationState,
    pub(crate) current: PlayerStatAllocationState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissions {
    pub(crate) player: bool,
    pub(crate) teammate: bool,
    pub(crate) guild_member: bool,
    pub(crate) criminal: bool,
    pub(crate) country: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissionMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: i8,
    pub(crate) requested: bool,
    pub(crate) recognized: bool,
    pub(crate) changed: bool,
    pub(crate) previous: PlayerPkPermissions,
    pub(crate) current: PlayerPkPermissions,
}

impl PlayerCombatProperties {
    pub(crate) const fn blast_defense_scale(self) -> f32 {
        f32::from_bits(self.blast_defense_scale_bits)
    }

    pub(crate) const fn full_miss_scale(self) -> f32 {
        f32::from_bits(self.full_miss_scale_bits)
    }

    pub(crate) const fn critical_rate(self) -> f32 {
        f32::from_bits(self.critical_rate_bits)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PlayerProgress {
    #[default]
    None,
    Banking,
    Trading,
    Shopping,
    OpenStall,
    Increment,
    Upgrade,
    Synthesis,
    Mailing,
    DaKong,
    Compose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsSessionPlayerRelease {
    pub(crate) previous_progress: PlayerProgress,
    pub(crate) previous_moveable_count: i32,
    pub(crate) resulting_moveable_count: i32,
    pub(crate) moveable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionSelfGoodsRefresh {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Requested {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
        goods_space: u32,
        wallet_space: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketAddition {
    pub(crate) player_id: i32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) position: Option<u32>,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[must_use = "изменение YuanBao содержит обязательный container/client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerYuanBaoChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: PlayerYuanBaoChangeOutcome,
}

#[must_use = "списание денег содержит wallet outcome для обязательного client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMoneyDecrease {
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: CurrencyDecreaseOutcome,
}

#[must_use = "изменение аукционных денег содержит wallet outcome для client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionMoneyChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: CurrencyIncreaseOutcome,
}

#[must_use = "возврат с аукциона содержит container, bind и ownership outcome"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionGoodsReturn {
    pub(crate) player_id: i32,
    pub(crate) position: u32,
    pub(crate) source: ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) resulting_goods: Option<ShapeIdentity>,
    pub(crate) resulting_amount: Option<u32>,
    pub(crate) bind_stored: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionBuyGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionListingGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerYuanBaoChangeOutcome {
    Unchanged,
    Increased(CurrencyIncreaseOutcome),
    Decreased(CurrencyDecreaseOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerAddition {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerConsumption {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[must_use = "уничтожение hand goods содержит ownership и listener-эффекты удаления"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementSelectionBlock {
    MissingGoods,
    UnsupportedSourceContainer,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    StackableGoods,
    MissingBaseProperties,
    Shadow(ShadowRecordBlock),
}

#[must_use = "selection report связывает source container и AddShadow effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementSelectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) shadow: AmountShadowAdded,
    pub(crate) previous_last_operated: (u32, u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementDeselectionBlock {
    MissingShadow,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    MissingSourceGoods,
}

#[must_use = "deselection report сохраняет original source и RemoveShadow effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementDeselectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) removed: super::container::cgoodsshadowcontainer::ShadowRemovedReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTalkChannel {
    Normal,
    Area,
    Country,
    World,
    Private,
    Union,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPlayer {
    move_shape: CMoveShape,
    figure: ShapeFigure,
    faction_id: i32,
    faction_master_id: i32,
    faction_name: Vec<u8>,
    union_id: i32,
    team_id: i32,
    team_captain: bool,
    country: u8,
    server_region_id: Option<i32>,
    in_changing_server: bool,
    in_changing_region: bool,
    state_before_server_region_change: u16,
    current_progress: PlayerProgress,
    personal_shop_session_id: i32,
    personal_shop_plug_id: i32,
    war_soul_state: u32,
    war_soul_point: WarSoulPoint,
    war_soul_visual_x_bits: u32,
    war_soul_visual_y_bits: u32,
    battle_fairy_summoned: bool,
    recreate_carriage: bool,
    create_faction_operator: bool,
    apply_join_faction_operator: bool,
    faction_declare_operator: bool,
    active_pet_count: u32,
    attempt_appellation_id: u32,
    realm_appellation_skill_id: u32,
    realm_appellation_skill_level: i32,
    heart_request_sent: i32,
    heart_received: bool,
    friends: Vec<PlayerFriend>,
    quest_states: BTreeMap<u16, u8>,
    lei_ting_things: VecDeque<PlayerLeiTingThing>,
    uncreated_pets: Vec<PlayerUncreatedPet>,
    uncreated_carriage: PlayerUncreatedCarriage,
    login: bool,
    session_id: Vec<u8>,
    title: Vec<u8>,
    base_property_wire: [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    jjc_data: [u8; 0x10],
    jjc_pk_state: bool,
    fight_state_count: i32,
    organizing_wire: Vec<u8>,
    base_properties: PlayerBaseProperties,
    combat_properties: PlayerCombatProperties,
    combat_property_wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    expendable_effects: BTreeMap<i32, PlayerExpendableEffect>,
    ci_qing_open: bool,
    ci_qing_list: BTreeSet<u32>,
    ci_qing_add_values: BTreeMap<u32, u32>,
    ci_qing_tao_zhuang_add_values: BTreeMap<u32, u32>,
    tao_zhuang_id: u32,
    contend_state: bool,
    emotion_index: i32,
    emotion_timestamp_ms: u32,
    city_war_died_state: bool,
    city_war_died_state_time_ms: i32,
    died_state_start_time_ms: u32,
    murderer_time_stamp_ms: u32,
    contribution: i32,
    silence_minutes: i32,
    silence_timestamp_minutes: u32,
    normal_talk_timestamp_ms: u32,
    area_talk_timestamp_ms: u32,
    world_talk_timestamp_ms: u32,
    country_talk_timestamp_ms: u32,
    private_talk_timestamp_ms: u32,
    union_talk_timestamp_ms: u32,
    money: u32,
    client_ip: u32,
    account: Vec<u8>,
    depot_password: Vec<u8>,
    last_container_script: Vec<u8>,
    variable_list: CVariableList,
    bank: CBank,
    depot: CDepot,
    hand: CAmountLimitGoodsContainer,
    enhancement: CAmountLimitGoodsShadowContainer,
    last_operated_container: u32,
    last_operated_goods_position: u32,
    packet: CVolumeLimitGoodsContainer,
    wallet: CWallet,
    yuan_bao: CYuanBao,
    ji_fen: CJiFen,
    equipment: CEquipmentContainer,
    auction_listing: CVolumeLimitGoodsContainer,
    auction_goods: CVolumeLimitGoodsContainer,
    auction_wallet: CWallet,
    auction_open: bool,
    auction_search_name: Vec<u8>,
    auction_search_lower_level: i32,
    auction_search_upper_level: i32,
    auction_search_use_self: i32,
    auction_search_money_type: i32,
    auction_search_weapon_type: i32,
    auction_current_page: i32,
    last_auction_limit_tick_ms: u32,
    last_auction_option_tick_ms: u32,
    current_auction_node: Option<CGoodsNode>,
    auction_listing_fee: u32,
    current_auction_buy_node: Option<CGoodsNode>,
    ci_qing: CVolumeLimitGoodsContainer,
    ci_qing_compose: CVolumeLimitGoodsContainer,
    fairy_container: CFairyContainer,
    battle_fairy_container: CBattleFairyContainer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveMutation {
    pub(crate) player_id: i32,
    pub(crate) previous_x: i32,
    pub(crate) previous_y: i32,
    pub(crate) direction: i32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
}

fn apply_ride_goods_properties(
    properties: &mut PlayerCombatProperties,
    goods: &CGoods,
    factory: &CGoodsFactory,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: usize,
) {
    fn add_u32(target: &mut u32, delta: i32) {
        *target = (i64::from(*target) + i64::from(delta)).clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn derived(value: i32, coefficient: f32) -> i32 {
        ((value as f32) * coefficient).round() as i32
    }

    let enabled = goods.enabled_addon_properties(factory);
    // Native `UpdateProperty` вызывает MountEquipRide(true), затем false:
    // первый pass принимает неотрицательные addon-ы, второй — отрицательные.
    for positive_pass in [true, false] {
        for &stored_type in &enabled {
            let fumo = stored_type == GAP_FUMO_PROPERTY;
            let (property_type, delta) = if fumo {
                (
                    goods.addon_property_value(factory, stored_type, 1),
                    goods.addon_property_value(factory, stored_type, 2),
                )
            } else {
                (
                    stored_type,
                    goods.addon_property_value(factory, stored_type, 1),
                )
            };
            // GAP_FUMO_PROPERTY native-ветка существует только в первом
            // `MountEquipRide(true)` pass и уже внутри принимает signed delta.
            if (fumo && !positive_pass) || (!fumo && (delta >= 0) != positive_pass) {
                continue;
            }
            match property_type {
                GAP_MINIMUM_ATTACK_CORRECTION => add_u32(&mut properties.minimum_attack, delta),
                GAP_MAXIMUM_ATTACK_CORRECTION => add_u32(&mut properties.maximum_attack, delta),
                GAP_ELEMENT_ATTACK_CORRECTION => {
                    let value = properties.element_modify.wrapping_add(delta);
                    properties.element_modify = if delta < 0 && value < 0 { 0 } else { value };
                }
                GAP_ARMOR_CORRECTION => add_u32(&mut properties.defense, delta),
                GAP_ATTACK_SPEED_CORRECTION => add_u16(&mut properties.attack_speed, delta),
                GAP_HIT_RATE_CORRECTION => add_u16(&mut properties.hit, delta),
                GAP_FATAL_BLOW_RATE_CORRECTION => add_u16(&mut properties.cch, delta),
                GAP_DODGE_CORRECTION => add_u16(&mut properties.dodge, delta),
                GAP_ELEMENT_RESISTANCE_CORRECTION => {
                    add_u32(&mut properties.element_resistance, delta)
                }
                GAP_HP_RESTORE_SPEED_CORRECTION => add_u16(&mut properties.hp_recovery, delta),
                GAP_MP_RESTORE_SPEED_CORRECTION => add_u16(&mut properties.mp_recovery, delta),
                GAP_STRENGTH_CORRECTION => {
                    add_u32(&mut properties.strength, delta);
                    add_u32(
                        &mut properties.maximum_attack,
                        derived(delta, coefficients.str_to_max_attack[occupation]),
                    );
                    add_u16(
                        &mut properties.burden,
                        derived(delta, coefficients.str_to_burden[occupation]),
                    );
                }
                GAP_AGILITY_CORRECTION => {
                    add_u32(&mut properties.dexterity, delta);
                    add_u32(
                        &mut properties.minimum_attack,
                        derived(delta, coefficients.dex_to_min_attack[occupation]),
                    );
                    add_u16(
                        &mut properties.reank,
                        derived(delta, coefficients.dex_to_stiff[occupation]),
                    );
                }
                GAP_CONSTITUTION_CORRECTION => {
                    add_u32(&mut properties.constitution, delta);
                    add_u32(
                        &mut properties.maximum_hp,
                        derived(delta, coefficients.con_to_max_hp[occupation]),
                    );
                    add_u32(
                        &mut properties.defense,
                        derived(delta, coefficients.con_to_defense[occupation]),
                    );
                }
                GAP_WAKAN_CORRECTION => {
                    add_u32(&mut properties.intelligence, delta);
                    properties.element_modify = properties
                        .element_modify
                        .wrapping_add(derived(delta, coefficients.int_to_element[occupation]));
                    if delta < 0 && properties.element_modify < 0 {
                        properties.element_modify = 0;
                    }
                    add_u32(
                        &mut properties.maximum_mp,
                        derived(delta, coefficients.int_to_max_mp[occupation]),
                    );
                    add_u32(
                        &mut properties.element_resistance,
                        derived(delta, coefficients.int_to_resistant[occupation]),
                    );
                }
                GAP_HP_UPPER_LIMIT_CORRECTION => add_u32(&mut properties.maximum_hp, delta),
                GAP_MP_UPPER_LIMIT_CORRECTION => add_u32(&mut properties.maximum_mp, delta),
                GAP_STIFFEN_PROBABILITY_CORRECTION => add_u16(&mut properties.reank, delta),
                GAP_BURDEN_UPPER_LIMIT_CORRECTION => add_u16(&mut properties.burden, delta),
                GAP_ATTACK_AVOID => add_u16(&mut properties.attack_avoid, delta),
                GAP_ELEMENT_AVOID => add_u16(&mut properties.element_avoid, delta),
                GAP_FULL_MISS => add_u16(&mut properties.full_miss, delta),
                GAP_BLAST_ATTACK => add_u16(&mut properties.blast_attack, delta),
                GAP_BLAST_ELEMENT_ATTACK => {
                    // Legacy case 96 берёт base из wBlastAttack, не из target.
                    let mut value = properties.blast_attack;
                    add_u16(&mut value, delta);
                    properties.blast_element_attack = value;
                }
                _ => {}
            }
        }
    }
}

impl CPlayer {
    /// Собирает только достигнутый send-family state уже созданного игрока;
    /// identity другого object type отвергается до регистрации.
    pub(crate) fn from_send_state(
        move_shape: CMoveShape,
        figure: ShapeFigure,
        team_id: i32,
        country: u8,
        server_region_id: Option<i32>,
    ) -> Option<Self> {
        if move_shape.shape().identity().object_type != PLAYER_TYPE {
            return None;
        }
        let owner_id = move_shape.shape().identity().id;
        let realm_appellation_bonus = move_shape
            .skills()
            .values()
            .find(|skill| {
                super::skills::realmappellation::is_bonus_skill(skill.id())
                    && (1..=4).contains(&skill.level())
            })
            .map(|skill| (skill.id(), skill.level()));
        let mut packet = CVolumeLimitGoodsContainer::new();
        let _empty_release = packet.set_container_dimensions(8, 12);
        let mut enhancement = CAmountLimitGoodsShadowContainer::new();
        enhancement.set_goods_amount_limit(1);
        enhancement.base_mut().set_container_extend_id(10);
        let mut ci_qing = CVolumeLimitGoodsContainer::new();
        let _empty_release = ci_qing.set_container_volume(8);
        let mut ci_qing_compose = CVolumeLimitGoodsContainer::new();
        let _empty_release = ci_qing_compose.set_container_volume(3);
        let mut fairy_container = CFairyContainer::new();
        let _empty_release = fairy_container.base_mut().set_container_volume(14);
        let mut auction_goods = CVolumeLimitGoodsContainer::new();
        let _empty_release = auction_goods.set_container_volume(0x12);
        let mut auction_listing = CVolumeLimitGoodsContainer::new();
        let _empty_release = auction_listing.set_container_volume(2);
        let mut player = Self {
            move_shape,
            figure,
            faction_id: 0,
            faction_master_id: 0,
            faction_name: Vec::new(),
            union_id: 0,
            team_id,
            team_captain: false,
            country,
            server_region_id,
            in_changing_server: false,
            in_changing_region: false,
            state_before_server_region_change: 0,
            current_progress: PlayerProgress::None,
            personal_shop_session_id: 0,
            personal_shop_plug_id: 0,
            war_soul_state: 0,
            war_soul_point: WarSoulPoint::default(),
            war_soul_visual_x_bits: 0.0f32.to_bits(),
            war_soul_visual_y_bits: 0.0f32.to_bits(),
            battle_fairy_summoned: false,
            recreate_carriage: false,
            create_faction_operator: false,
            apply_join_faction_operator: false,
            faction_declare_operator: false,
            active_pet_count: 0,
            attempt_appellation_id: 0,
            realm_appellation_skill_id: realm_appellation_bonus
                .map_or(UNKNOWN_SKILL_ID, |identity| identity.0),
            realm_appellation_skill_level: realm_appellation_bonus.map_or(0, |identity| identity.1),
            heart_request_sent: 0,
            heart_received: false,
            friends: Vec::new(),
            quest_states: BTreeMap::new(),
            lei_ting_things: VecDeque::new(),
            uncreated_pets: Vec::new(),
            uncreated_carriage: PlayerUncreatedCarriage::default(),
            login: false,
            session_id: Vec::new(),
            title: Vec::new(),
            base_property_wire: [0; PLAYER_BASE_PROPERTY_WIRE_SIZE],
            jjc_data: [0; 0x10],
            jjc_pk_state: false,
            fight_state_count: 0,
            organizing_wire: Vec::new(),
            base_properties: PlayerBaseProperties::default(),
            combat_properties: PlayerCombatProperties::default(),
            combat_property_wire: [0; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
            expendable_effects: BTreeMap::new(),
            ci_qing_open: false,
            ci_qing_list: BTreeSet::new(),
            ci_qing_add_values: BTreeMap::new(),
            ci_qing_tao_zhuang_add_values: BTreeMap::new(),
            tao_zhuang_id: 0,
            contend_state: false,
            emotion_index: 0,
            emotion_timestamp_ms: 0,
            city_war_died_state: false,
            city_war_died_state_time_ms: 0,
            died_state_start_time_ms: 0,
            murderer_time_stamp_ms: 0,
            contribution: 0,
            silence_minutes: 0,
            silence_timestamp_minutes: 0,
            normal_talk_timestamp_ms: 0,
            area_talk_timestamp_ms: 0,
            world_talk_timestamp_ms: 0,
            country_talk_timestamp_ms: 0,
            private_talk_timestamp_ms: 0,
            union_talk_timestamp_ms: 0,
            money: 0,
            client_ip: 0,
            account: Vec::new(),
            depot_password: Vec::new(),
            last_container_script: Vec::new(),
            variable_list: CVariableList::default(),
            bank: CBank::new(),
            depot: CDepot::new(),
            hand: CAmountLimitGoodsContainer::new(),
            enhancement,
            last_operated_container: 0,
            last_operated_goods_position: 0,
            packet,
            wallet: CWallet::new(),
            yuan_bao: CYuanBao::new(),
            ji_fen: CJiFen::new(),
            equipment: CEquipmentContainer::new(),
            auction_listing,
            auction_goods,
            auction_wallet: CWallet::new(),
            auction_open: false,
            auction_search_name: Vec::new(),
            auction_search_lower_level: 0,
            auction_search_upper_level: 0,
            auction_search_use_self: 0,
            auction_search_money_type: 0,
            auction_search_weapon_type: 0,
            auction_current_page: 0,
            last_auction_limit_tick_ms: 0,
            last_auction_option_tick_ms: 0,
            current_auction_node: None,
            auction_listing_fee: 0,
            current_auction_buy_node: None,
            ci_qing,
            ci_qing_compose,
            fairy_container,
            battle_fairy_container: CBattleFairyContainer::new(),
        };
        player.refresh_reached_container_owners(owner_id);
        Some(player)
    }

    /// Exact World→Game player handoff. Decoder восстанавливает единый
    /// `CPlayer::DecordFromByteArray(..., true)` блок, а не отдельные игровые
    /// фрагменты. Native GoodsAI tail намеренно отложен до успешной регистрации
    /// player-а в map/region и исполняется `CGame::complete_world_player_login`
    /// для достигнутых equipment и packet owners.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn decode_game_save(
        source: &[u8],
        cursor: &mut usize,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        variable_definitions: Option<&[u8]>,
        now_ms: u32,
        one_pk_count_time_ms: u32,
        ordinary_threshold: &mut dyn FnMut(u32, u32) -> u32,
        battle_threshold: &mut dyn FnMut(u32, u32) -> u32,
    ) -> Result<(Self, PlayerGameSaveDecodeReport), PlayerGameSaveCodecError> {
        let start = *cursor;
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .decode_from_byte_array(source, cursor, true)?;
        let object_type = move_shape.shape().identity().object_type;
        if object_type != PLAYER_TYPE {
            return Err(PlayerGameSaveCodecError::WrongObjectType { object_type });
        }
        let server_region_id = move_shape.shape().get_region_id();

        let mut player = Self::from_send_state(
            move_shape,
            ShapeFigure::default(),
            0,
            0,
            Some(server_region_id),
        )
        .expect("player object type проверен до создания CPlayer");
        player.base_property_wire =
            read_player_game_save_array(source, cursor, "m_BaseProperty[0x194]")?;
        player.apply_base_property_wire();
        player.battle_fairy_summoned = player.war_soul_state != 0;
        player.account = read_player_game_save_string(source, cursor, "strAccount", 0x100)?;
        player.title = read_player_game_save_string(source, cursor, "strTitle", 0x100)?;
        player.combat_property_wire =
            read_player_game_save_array(source, cursor, "m_Property[0x9c]")?;
        player.apply_combat_property_wire();
        player.team_id = read_player_game_save_i32(source, cursor, "m_lTeamID")?;

        player.ci_qing_list.clear();
        let ci_qing_count = read_player_game_save_count(source, cursor, "m_setCiQingList")?;
        for _ in 0..ci_qing_count {
            player.ci_qing_list.insert(read_player_game_save_u32(
                source,
                cursor,
                "m_setCiQingList entry",
            )?);
        }

        player.move_shape.clear_persisted_runtime_state();
        let skill_count = read_player_game_save_count(source, cursor, "skill count")?;
        for _ in 0..skill_count {
            let packed = read_player_game_save_u32(source, cursor, "tagSkillID")?;
            let skill_id = packed & 0xffff;
            let level = (packed >> 16) as i32;
            let loaded = player.move_shape.add_skill(skill_id, level, skill_factory);
            if loaded
                && super::skills::realmappellation::is_bonus_skill(skill_id)
                && (1..=4).contains(&level)
            {
                player.realm_appellation_skill_id = skill_id;
                player.realm_appellation_skill_level = level;
            }
        }
        let ex_state_length = read_player_game_save_count(source, cursor, "m_vExStates length")?;
        player.move_shape.replace_ex_states(
            read_player_game_save_slice(source, cursor, "m_vExStates", ex_state_length)?.to_vec(),
        );

        player.friends.clear();
        let friend_count = read_player_game_save_count(source, cursor, "m_listFriend")?;
        for _ in 0..friend_count {
            player.friends.push(PlayerFriend {
                name: read_player_game_save_string(source, cursor, "tagFriend.strName", 0x94)?,
                online: read_player_game_save_u8(source, cursor, "tagFriend.bOnline")? != 0,
            });
        }
        player.decode_lei_ting(source, cursor)?;

        let _cleared_hand = player.hand.clear_goods();
        player.hand.set_goods_amount_limit(1);
        player.hand.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;

        let base_properties = player.base_properties;
        let combat_properties = player.combat_properties;
        let equipment = player.equipment.unserialize_with(
            source,
            cursor,
            goods_factory,
            true,
            |source, cursor| {
                let mut goods = CGoods::default();
                goods.unserialize(
                    source,
                    cursor,
                    true,
                    goods_factory,
                    &mut *ordinary_threshold,
                    &mut *battle_threshold,
                )?;
                Ok(Some(goods))
            },
            |goods| EquipmentAddRuntimeFacts {
                owner_player: Some(EquipmentOwnerPlayerFacts {
                    can_mount_result: Self::can_mount_equip_from_properties(
                        base_properties,
                        combat_properties,
                        goods,
                        goods_factory,
                    ),
                }),
                pack_add_enabled: false,
                now: u64::from(now_ms),
            },
            &mut |_| {},
            &mut |_, _, _| {},
        );
        let equipment =
            equipment.map_err(|failure| PlayerGameSaveCodecError::Equipment(failure.error))?;
        if let Some(position) = equipment.entries.iter().find_map(|entry| match entry {
            EquipmentUnserializedEntry::Rejected { position, .. }
            | EquipmentUnserializedEntry::DecoderReturnedNull { position } => Some(*position),
            EquipmentUnserializedEntry::Added { .. } => None,
        }) {
            return Err(PlayerGameSaveCodecError::EquipmentRejected { position });
        }

        let _released = player.packet.set_container_dimensions(8, 12);
        player.packet.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player
            .packet
            .apply_player_expansion_limit(player.equipment.expanded_package_num());

        let _released = player.auction_goods.set_container_volume(0x12);
        player.auction_goods.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.auction_listing.set_container_volume(2);
        player.auction_listing.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.wallet.unserialize(
            source,
            cursor,
            "m_cWallet marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.auction_wallet.unserialize(
            source,
            cursor,
            "m_cAuctionWallet marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.yuan_bao.unserialize(
            source,
            cursor,
            "m_cYuanBao marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.ji_fen.unserialize(
            source,
            cursor,
            "m_cJiFen marker",
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        player.money = player.wallet.currency_amount();

        player.depot_password =
            read_player_game_save_string(source, cursor, "m_strDepotPassword", 0x6c)?;
        player.bank.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.depot.base_mut().set_container_volume(0xa1);
        player.depot.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.fairy_container.base_mut().set_container_volume(0x0e);
        player.fairy_container.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player
            .battle_fairy_container
            .base_mut()
            .set_container_volume(0x11);
        player.battle_fairy_container.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.ci_qing.set_container_volume(8);
        player.ci_qing.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;
        let _released = player.ci_qing_compose.set_container_volume(3);
        player.ci_qing_compose.unserialize(
            source,
            cursor,
            goods_factory,
            &mut *ordinary_threshold,
            &mut *battle_threshold,
        )?;

        player
            .variable_list
            .decode_world_snapshot(variable_definitions, source, cursor)?;
        player.silence_minutes = read_player_game_save_i32(source, cursor, "m_lSilenceTime")?;
        let murderer_state = read_player_game_save_u8(source, cursor, "murderer state")? != 0;
        let murderer_remain = read_player_game_save_u32(source, cursor, "murderer remain time")?;
        player.restore_murderer_timestamp(
            murderer_state,
            murderer_remain,
            now_ms,
            one_pk_count_time_ms,
        );
        player.fight_state_count = read_player_game_save_i32(source, cursor, "m_lFightStateCount")?;

        player.uncreated_pets.clear();
        let pet_count = read_player_game_save_count(source, cursor, "m_vUncreatedPets")?;
        for _ in 0..pet_count {
            player.uncreated_pets.push(PlayerUncreatedPet {
                original_name: read_player_game_save_string(
                    source,
                    cursor,
                    "tagPetInformation.strOriginalName",
                    0x94,
                )?,
                health: read_player_game_save_u32(source, cursor, "tagPetInformation.dwHp")?,
                level: read_player_game_save_u32(source, cursor, "tagPetInformation.dwLevel")?,
                experience: read_player_game_save_u32(
                    source,
                    cursor,
                    "tagPetInformation.dwExperience",
                )?,
            });
        }
        player.uncreated_carriage = PlayerUncreatedCarriage {
            original_name: read_player_game_save_string(
                source,
                cursor,
                "tagCarriageInfo.strOriginalName",
                0x94,
            )?,
            script: read_player_game_save_string(
                source,
                cursor,
                "tagCarriageInfo.strCarriageScript",
                0x94,
            )?,
            health: read_player_game_save_u32(source, cursor, "tagCarriageInfo.dwHp")?,
        };
        player.recreate_carriage =
            read_player_game_save_u8(source, cursor, "m_bReCreateCarriage")? != 0;
        player.login = read_player_game_save_u8(source, cursor, "m_bLogin")? != 0;
        player.city_war_died_state_time_ms =
            read_player_game_save_i32(source, cursor, "m_lCityWarDiedStateTime")?;
        player.died_state_start_time_ms =
            u32::from(player.city_war_died_state_time_ms > 0).wrapping_mul(now_ms);
        player.city_war_died_state =
            player.city_war_died_state_time_ms > 0 && player.base_properties.occupation != 6;

        player.quest_states.clear();
        let quest_count = read_player_game_save_count(source, cursor, "m_PlayerQuests")?;
        for _ in 0..quest_count {
            let quest_id = read_player_game_save_u16(source, cursor, "tagPlayerQuest.wQuestID")?;
            let state = read_player_game_save_u8(source, cursor, "tagPlayerQuest.byComplete")?;
            player.quest_states.insert(quest_id, state);
        }
        player.country = read_player_game_save_u8(source, cursor, "m_btCountry")?;
        player.contribution = read_player_game_save_i32(source, cursor, "m_lContribute")?;
        player.jjc_data = read_player_game_save_array(source, cursor, "m_jjcdata[0x10]")?;
        player.jjc_pk_state = read_player_game_save_u8(source, cursor, "bJJcPkState")? != 0;
        player.decode_organizing_snapshot(source, cursor)?;
        player.session_id = read_player_game_save_string(source, cursor, "m_strSessionID", 0x40)?;
        player.refresh_reached_container_owners(player.player_id());

        Ok((
            player,
            PlayerGameSaveDecodeReport {
                consumed_bytes: cursor.saturating_sub(start),
            },
        ))
    }

    /// `AddGameSaveToByteArray`: тот же persisted layout без organization
    /// snapshot (World обновляет его самостоятельно перед следующим handoff).
    pub(crate) fn encode_game_save(
        &self,
        destination: &mut Vec<u8>,
        goods_factory: &CGoodsFactory,
        now_ms: u32,
        one_pk_count_time_ms: u32,
        pets: &[PlayerUncreatedPet],
        carriage: &PlayerUncreatedCarriage,
        recreate_carriage: bool,
    ) -> Result<bool, PlayerGameSaveCodecError> {
        if !self.shape().encode_to_byte_array(destination, true) {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse { field: "CShape" });
        }
        destination.extend_from_slice(&self.synchronized_base_property_wire());
        append_player_game_save_string(destination, "strAccount", &self.account, 0x100)?;
        append_player_game_save_string(destination, "strTitle", &self.title, 0x100)?;
        destination.extend_from_slice(&self.combat_property_wire);
        destination.extend_from_slice(&self.team_id.to_le_bytes());

        append_player_game_save_count(destination, "m_setCiQingList", self.ci_qing_list.len())?;
        for base_index in &self.ci_qing_list {
            destination.extend_from_slice(&base_index.to_le_bytes());
        }
        let skills: Vec<_> = self
            .move_shape
            .skills()
            .values()
            .filter(|skill| !(skill.skill_type() == 1 && skill.id() == 10))
            .collect();
        append_player_game_save_count(destination, "skill count", skills.len())?;
        for skill in skills {
            let packed = (skill.id() & 0xffff) | ((skill.level() as u32 & 0xffff) << 16);
            destination.extend_from_slice(&packed.to_le_bytes());
        }
        let ex_states = self.move_shape.serialized_ex_states(now_ms);
        append_player_game_save_count(destination, "m_vExStates length", ex_states.len())?;
        destination.extend_from_slice(&ex_states);
        append_player_game_save_count(destination, "m_listFriend", self.friends.len())?;
        for friend in &self.friends {
            append_player_game_save_string(destination, "tagFriend.strName", &friend.name, 0x94)?;
            destination.push(u8::from(friend.online));
        }
        destination.extend_from_slice(&self.encode_lei_ting());

        destination.extend_from_slice(&0i32.to_le_bytes());
        let mut equipment_serialized = true;
        self.equipment.serialize_with(
            destination,
            goods_factory,
            true,
            |goods, include_child, destination| {
                equipment_serialized &= goods.serialize(destination, include_child);
            },
        );
        if !equipment_serialized {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse {
                field: "m_cEquipment",
            });
        }
        macro_rules! serialize_container {
            ($field:literal, $serialized:expr) => {
                if !$serialized {
                    return Err(PlayerGameSaveCodecError::CodecReturnedFalse { field: $field });
                }
            };
        }
        serialize_container!(
            "m_cPacket",
            self.packet.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cAuctionGoodsContainer",
            self.auction_goods.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cAuctionContainer",
            self.auction_listing.serialize(destination, goods_factory)
        );
        serialize_container!("m_cWallet", self.wallet.serialize(destination));
        serialize_container!(
            "m_cAuctionWallet",
            self.auction_wallet.serialize(destination)
        );
        serialize_container!("m_cYuanBao", self.yuan_bao.serialize(destination));
        serialize_container!("m_cJiFen", self.ji_fen.serialize(destination));
        append_player_game_save_string(
            destination,
            "m_strDepotPassword",
            &self.depot_password,
            0x6c,
        )?;
        serialize_container!("m_cBank", self.bank.serialize(destination));
        serialize_container!("m_cDepot", self.depot.serialize(destination, goods_factory));
        serialize_container!(
            "m_cFairy",
            self.fairy_container.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cBF",
            self.battle_fairy_container
                .serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cCiQing",
            self.ci_qing.serialize(destination, goods_factory)
        );
        serialize_container!(
            "m_cComposeCiQing",
            self.ci_qing_compose.serialize(destination, goods_factory)
        );
        if !self.variable_list.encode_world_snapshot(destination) {
            return Err(PlayerGameSaveCodecError::CodecReturnedFalse {
                field: "m_pVariableList",
            });
        }
        destination.extend_from_slice(&self.silence_minutes.to_le_bytes());
        let murderer_state = self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms != 0;
        destination.push(u8::from(murderer_state));
        let murderer_remain = if self.murderer_time_stamp_ms == 0 {
            0
        } else {
            self.murderer_time_stamp_ms
                .wrapping_add(one_pk_count_time_ms)
                .wrapping_sub(now_ms)
                .min(one_pk_count_time_ms)
        };
        destination.extend_from_slice(&murderer_remain.to_le_bytes());
        destination.extend_from_slice(&self.fight_state_count.to_le_bytes());
        append_player_game_save_count(destination, "m_vUncreatedPets", pets.len())?;
        for pet in pets {
            append_player_game_save_string(
                destination,
                "tagPetInformation.strOriginalName",
                &pet.original_name,
                0x94,
            )?;
            destination.extend_from_slice(&pet.health.to_le_bytes());
            destination.extend_from_slice(&pet.level.to_le_bytes());
            destination.extend_from_slice(&pet.experience.to_le_bytes());
        }
        append_player_game_save_string(
            destination,
            "tagCarriageInfo.strOriginalName",
            &carriage.original_name,
            0x94,
        )?;
        append_player_game_save_string(
            destination,
            "tagCarriageInfo.strCarriageScript",
            &carriage.script,
            0x94,
        )?;
        destination.extend_from_slice(&carriage.health.to_le_bytes());
        destination.push(u8::from(recreate_carriage));
        destination.push(u8::from(self.login));
        destination.extend_from_slice(&self.city_war_died_state_time_ms.to_le_bytes());
        append_player_game_save_count(destination, "m_PlayerQuests", self.quest_states.len())?;
        for (quest_id, state) in &self.quest_states {
            destination.extend_from_slice(&quest_id.to_le_bytes());
            destination.push(*state);
        }
        destination.push(self.country);
        destination.extend_from_slice(&self.contribution.to_le_bytes());
        destination.extend_from_slice(&self.jjc_data);
        destination.push(u8::from(self.jjc_pk_state));
        append_player_game_save_string(destination, "m_strSessionID", &self.session_id, 0x40)?;
        Ok(true)
    }

    fn synchronized_base_property_wire(&self) -> [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE] {
        let mut wire = self.base_property_wire;
        wire[BASE_LEVEL_OFFSET] = self.base_properties.level;
        write_player_wire_u32(
            &mut wire,
            BASE_EXPERIENCE_OFFSET,
            self.base_properties.experience,
        );
        wire[BASE_HEAD_PICTURE_OFFSET] = self.base_properties.head_picture as u8;
        wire[BASE_FACE_PICTURE_OFFSET] = self.base_properties.face_picture as u8;
        wire[BASE_OCCUPATION_OFFSET] = self.base_properties.occupation;
        wire[BASE_SEX_OFFSET] = self.base_properties.sex;
        write_player_wire_u16(
            &mut wire,
            BASE_PK_COUNT_OFFSET,
            self.base_properties.pk_count,
        );
        write_player_wire_u32(
            &mut wire,
            BASE_KILL_COUNT_OFFSET,
            self.base_properties.kill_count,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_REMAIN_POINT_OFFSET,
            self.base_properties.remain_point,
        );
        wire[BASE_CHARGED_OFFSET] = u8::from(self.base_properties.charged);
        for (index, hotkey) in self.base_properties.hotkeys.iter().copied().enumerate() {
            write_player_wire_u32(&mut wire, BASE_HOTKEY_OFFSET + index * 4, hotkey);
        }
        for (offset, value) in [
            (BASE_PK_NORMAL_OFFSET, self.base_properties.pk_normal),
            (BASE_PK_TEAM_OFFSET, self.base_properties.pk_team),
            (BASE_PK_UNION_OFFSET, self.base_properties.pk_union),
            (BASE_PK_BADMAN_OFFSET, self.base_properties.pk_badman),
            (BASE_PK_COUNTRY_OFFSET, self.base_properties.pk_country),
            (
                BASE_FAIRY_CONTAINER_ENABLED_OFFSET,
                self.base_properties.fairy_container_enabled,
            ),
            (
                BASE_BATTLE_FAIRY_ENABLED_OFFSET,
                self.base_properties.battle_fairy_enabled,
            ),
            (
                BASE_DISPLAY_HEAD_PIECE_OFFSET,
                self.base_properties.display_head_piece,
            ),
            (
                BASE_BATTLE_FAIRY_SUMMONED_OFFSET,
                self.battle_fairy_summoned,
            ),
            (
                BASE_BATTLE_FAIRY_RECALL_OFFSET,
                self.base_properties.battle_fairy_recall,
            ),
            (
                BASE_BATTLE_FAIRY_DIED_OFFSET,
                self.base_properties.battle_fairy_died,
            ),
            (
                BASE_QUEST_ENABLED_OFFSET,
                self.base_properties.quest_enabled,
            ),
        ] {
            wire[offset] = u8::from(value);
        }
        for (offset, value) in [
            (BASE_HEALTH_OFFSET, self.base_properties.health),
            (BASE_MANA_OFFSET, self.base_properties.mana),
            (BASE_MAXIMUM_HP_OFFSET, self.base_properties.base_maximum_hp),
            (BASE_MAXIMUM_MP_OFFSET, self.base_properties.base_maximum_mp),
            (BASE_STRENGTH_OFFSET, self.base_properties.base_strength),
            (BASE_DEXTERITY_OFFSET, self.base_properties.base_dexterity),
            (
                BASE_CONSTITUTION_OFFSET,
                self.base_properties.base_constitution,
            ),
            (
                BASE_INTELLIGENCE_OFFSET,
                self.base_properties.base_intelligence,
            ),
            (BASE_VIGOUR_OFFSET, self.base_properties.vigour),
            (BASE_CREDIT_OFFSET, self.base_properties.credit),
            (
                BASE_QUEST_TIME_BEGIN_OFFSET,
                self.base_properties.quest_time_begin as u32,
            ),
            (
                BASE_QUEST_TIME_LIMIT_OFFSET,
                self.base_properties.quest_time_limit as u32,
            ),
            (BASE_EXPLOIT_OFFSET, self.base_properties.exploit),
            (
                BASE_DAYS_HONOR_OFFSET,
                self.base_properties.days_honor_eliminate,
            ),
            (
                BASE_WEEKS_HONOR_OFFSET,
                self.base_properties.weeks_honor_eliminate,
            ),
            (
                BASE_MONTHS_HONOR_OFFSET,
                self.base_properties.months_honor_eliminate,
            ),
            (
                BASE_TOTAL_HONOR_OFFSET,
                self.base_properties.total_honor_eliminate,
            ),
            (
                BASE_RANK_OF_NOBILITY_OFFSET,
                self.base_properties.rank_of_nobility_id,
            ),
            (BASE_APPELLATION_OFFSET, self.base_properties.appellation_id),
            (BASE_MODE_OFFSET, self.base_properties.mode),
            (BASE_FETCH_POWER_OFFSET, self.base_properties.fetch_power),
            (BASE_FY_ENERGY_OFFSET, self.base_properties.fy_energy),
            (
                BASE_FY_ENABLE_FLAGS_OFFSET,
                self.base_properties.fy_enable_flags,
            ),
            (BASE_LT_60_STAMP_OFFSET, self.base_properties.lt_60_stamp),
            (BASE_SZL_OFFSET, self.base_properties.szl),
            (
                BASE_GODS_BATTLE_FACTION_OFFSET,
                self.base_properties.gods_battle_faction as u32,
            ),
        ] {
            write_player_wire_u32(&mut wire, offset, value);
        }
        write_player_wire_u16(
            &mut wire,
            BASE_LT_UP_60_COUNT_OFFSET,
            self.base_properties.lt_up_60_count,
        );
        write_player_wire_u16(
            &mut wire,
            BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET,
            self.base_properties.remain_jing_li_dan_count,
        );
        wire
    }

    fn restore_murderer_timestamp(
        &mut self,
        murderer_state: bool,
        murderer_remain: u32,
        now_ms: u32,
        one_pk_count_time_ms: u32,
    ) {
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
            return;
        }
        if murderer_state || self.murderer_time_stamp_ms == 0 {
            self.murderer_time_stamp_ms = now_ms;
        }
        if murderer_remain != 0 {
            let remain = murderer_remain.min(one_pk_count_time_ms);
            self.murderer_time_stamp_ms =
                now_ms.wrapping_sub(one_pk_count_time_ms.wrapping_sub(remain));
        }
    }

    fn apply_base_property_wire(&mut self) {
        let wire = &self.base_property_wire;
        self.base_properties.level = wire[BASE_LEVEL_OFFSET];
        self.base_properties.experience = read_player_wire_u32(wire, BASE_EXPERIENCE_OFFSET);
        self.base_properties.head_picture = i32::from(wire[BASE_HEAD_PICTURE_OFFSET]);
        self.base_properties.face_picture = i32::from(wire[BASE_FACE_PICTURE_OFFSET]);
        self.base_properties.occupation = wire[BASE_OCCUPATION_OFFSET];
        self.base_properties.sex = wire[BASE_SEX_OFFSET];
        self.base_properties.pk_count = read_player_wire_u16(wire, BASE_PK_COUNT_OFFSET);
        self.base_properties.kill_count = read_player_wire_u32(wire, BASE_KILL_COUNT_OFFSET);
        self.base_properties.charged = wire[BASE_CHARGED_OFFSET] != 0;
        self.base_properties.remain_point = read_player_wire_u16(wire, BASE_REMAIN_POINT_OFFSET);
        for (index, hotkey) in self.base_properties.hotkeys.iter_mut().enumerate() {
            *hotkey = read_player_wire_u32(wire, BASE_HOTKEY_OFFSET + index * 4);
        }
        self.base_properties.pk_normal = wire[BASE_PK_NORMAL_OFFSET] != 0;
        self.base_properties.pk_team = wire[BASE_PK_TEAM_OFFSET] != 0;
        self.base_properties.pk_union = wire[BASE_PK_UNION_OFFSET] != 0;
        self.base_properties.pk_badman = wire[BASE_PK_BADMAN_OFFSET] != 0;
        self.base_properties.pk_country = wire[BASE_PK_COUNTRY_OFFSET] != 0;
        self.base_properties.health = read_player_wire_u32(wire, BASE_HEALTH_OFFSET);
        self.base_properties.mana = read_player_wire_u32(wire, BASE_MANA_OFFSET);
        self.base_properties.base_maximum_hp = read_player_wire_u32(wire, BASE_MAXIMUM_HP_OFFSET);
        self.base_properties.base_maximum_mp = read_player_wire_u32(wire, BASE_MAXIMUM_MP_OFFSET);
        self.base_properties.base_strength = read_player_wire_u32(wire, BASE_STRENGTH_OFFSET);
        self.base_properties.base_dexterity = read_player_wire_u32(wire, BASE_DEXTERITY_OFFSET);
        self.base_properties.base_constitution =
            read_player_wire_u32(wire, BASE_CONSTITUTION_OFFSET);
        self.base_properties.base_intelligence =
            read_player_wire_u32(wire, BASE_INTELLIGENCE_OFFSET);
        self.base_properties.vigour = read_player_wire_u32(wire, BASE_VIGOUR_OFFSET);
        self.base_properties.credit = read_player_wire_u32(wire, BASE_CREDIT_OFFSET);
        self.base_properties.display_head_piece = wire[BASE_DISPLAY_HEAD_PIECE_OFFSET] != 0;
        self.base_properties.quest_time_begin =
            read_player_wire_u32(wire, BASE_QUEST_TIME_BEGIN_OFFSET) as i32;
        self.base_properties.quest_time_limit =
            read_player_wire_u32(wire, BASE_QUEST_TIME_LIMIT_OFFSET) as i32;
        self.base_properties.quest_enabled = wire[BASE_QUEST_ENABLED_OFFSET] != 0;
        self.base_properties.exploit = read_player_wire_u32(wire, BASE_EXPLOIT_OFFSET);
        self.base_properties.fairy_container_enabled =
            wire[BASE_FAIRY_CONTAINER_ENABLED_OFFSET] != 0;
        self.base_properties.battle_fairy_enabled = wire[BASE_BATTLE_FAIRY_ENABLED_OFFSET] != 0;
        self.base_properties.days_honor_eliminate =
            read_player_wire_u32(wire, BASE_DAYS_HONOR_OFFSET);
        self.base_properties.weeks_honor_eliminate =
            read_player_wire_u32(wire, BASE_WEEKS_HONOR_OFFSET);
        self.base_properties.months_honor_eliminate =
            read_player_wire_u32(wire, BASE_MONTHS_HONOR_OFFSET);
        self.base_properties.total_honor_eliminate =
            read_player_wire_u32(wire, BASE_TOTAL_HONOR_OFFSET);
        self.base_properties.rank_of_nobility_id =
            read_player_wire_u32(wire, BASE_RANK_OF_NOBILITY_OFFSET);
        self.base_properties.appellation_id = read_player_wire_u32(wire, BASE_APPELLATION_OFFSET);
        self.base_properties.mode = read_player_wire_u32(wire, BASE_MODE_OFFSET);
        self.base_properties.fetch_power = read_player_wire_u32(wire, BASE_FETCH_POWER_OFFSET);
        self.battle_fairy_summoned = wire[BASE_BATTLE_FAIRY_SUMMONED_OFFSET] != 0;
        self.base_properties.battle_fairy_recall = wire[BASE_BATTLE_FAIRY_RECALL_OFFSET] != 0;
        self.base_properties.battle_fairy_died = wire[BASE_BATTLE_FAIRY_DIED_OFFSET] != 0;
        self.base_properties.fy_energy = read_player_wire_u32(wire, BASE_FY_ENERGY_OFFSET);
        self.base_properties.fy_enable_flags =
            read_player_wire_u32(wire, BASE_FY_ENABLE_FLAGS_OFFSET);
        self.base_properties.lt_up_60_count =
            read_player_wire_u16(wire, BASE_LT_UP_60_COUNT_OFFSET);
        self.base_properties.remain_jing_li_dan_count =
            read_player_wire_u16(wire, BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET);
        self.base_properties.lt_60_stamp = read_player_wire_u32(wire, BASE_LT_60_STAMP_OFFSET);
        self.base_properties.szl = read_player_wire_u32(wire, BASE_SZL_OFFSET);
        self.base_properties.gods_battle_faction =
            read_player_wire_u32(wire, BASE_GODS_BATTLE_FACTION_OFFSET) as i32;
    }

    fn apply_combat_property_wire(&mut self) {
        let wire = &self.combat_property_wire;
        self.combat_properties = PlayerCombatProperties {
            maximum_hp: read_player_wire_u32(wire, 0x00),
            maximum_mp: read_player_wire_u32(wire, 0x04),
            strength: read_player_wire_u32(wire, 0x0c),
            dexterity: read_player_wire_u32(wire, 0x10),
            constitution: read_player_wire_u32(wire, 0x14),
            intelligence: read_player_wire_u32(wire, 0x18),
            minimum_attack: read_player_wire_u32(wire, 0x1c),
            maximum_attack: read_player_wire_u32(wire, 0x20),
            attack_speed: read_player_wire_u16(wire, 0x24),
            hit: read_player_wire_u16(wire, 0x2a),
            dodge: read_player_wire_u16(wire, 0x30),
            cch: read_player_wire_u16(wire, 0x28),
            burden: read_player_wire_u16(wire, 0x26),
            defense: read_player_wire_u32(wire, 0x2c),
            element_resistance: read_player_wire_u32(wire, 0x34),
            hp_recovery: read_player_wire_u16(wire, 0x38),
            mp_recovery: read_player_wire_u16(wire, 0x3a),
            element_modify: read_player_wire_u32(wire, 0x48) as i32,
            reank: read_player_wire_u16(wire, 0x4c),
            attack_avoid: read_player_wire_u16(wire, 0x4e),
            element_avoid: read_player_wire_u16(wire, 0x50),
            full_miss: read_player_wire_u16(wire, 0x52),
            blast_attack: read_player_wire_u16(wire, 0x54),
            blast_element_attack: read_player_wire_u16(wire, 0x56),
            blast_defense_scale_bits: read_player_wire_u32(wire, 0x5c),
            full_miss_scale_bits: read_player_wire_u32(wire, 0x68),
            critical_rate_bits: read_player_wire_u32(wire, 0x6c),
        };
    }

    fn decode_organizing_snapshot(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerGameSaveCodecError> {
        let start = *cursor;
        self.faction_id = read_player_game_save_i32(source, cursor, "m_lFactionID")?;
        if self.faction_id > 0 {
            let _logo = read_player_game_save_i32(source, cursor, "m_lFactionLogoID")?;
            let _level = read_player_game_save_u16(source, cursor, "m_wFactionLevel")?;
            let _experience = read_player_game_save_i32(source, cursor, "m_lFactionExperience")?;
            let _force = read_player_game_save_i32(source, cursor, "m_lForce")?;
            let _contribute = read_player_game_save_u32(source, cursor, "m_bFactionContribute")?;
            self.faction_name =
                read_player_game_save_string(source, cursor, "m_strFactionName", 0x100)?;
            let _title = read_player_game_save_string(source, cursor, "m_strFactionTitle", 0x100)?;
            self.faction_master_id =
                read_player_game_save_i32(source, cursor, "m_lFactionMasterID")?;
            self.union_id = read_player_game_save_i32(source, cursor, "m_lUnionID")?;
            let _union_master = read_player_game_save_i32(source, cursor, "m_lUnionMasterID")?;
            for field in ["m_EnemyFactions", "m_CityWarEnemyFactions"] {
                let count = read_player_game_save_count(source, cursor, field)?;
                let _ = read_player_game_save_slice(source, cursor, field, count * 4)?;
            }
            let count = read_player_game_save_count(source, cursor, "m_OwnedRegions")?;
            let _ = read_player_game_save_slice(source, cursor, "m_OwnedRegions", count * 8)?;
        } else {
            self.faction_name.clear();
            self.faction_master_id = 0;
            self.union_id = 0;
        }
        self.organizing_wire = source[start..*cursor].to_vec();
        Ok(())
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.move_shape.shape()
    }

    pub(crate) const fn restore_login_team(&mut self, captain: bool, team_id: i32) {
        self.team_captain = captain;
        self.team_id = team_id;
    }

    pub(crate) const fn mark_login_script_started(&mut self) -> bool {
        let first_login = !self.login;
        self.login = true;
        first_login
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.shape().identity().id
    }

    pub(crate) fn player_name(&self) -> &[u8] {
        self.shape().base_object().get_name()
    }

    pub(crate) fn friends(&self) -> &[PlayerFriend] {
        &self.friends
    }

    /// Exact `GetQuestState`: отсутствующий ushort ID имеет state `2`,
    /// существующий возвращает persisted byte без дополнительной проверки.
    pub(crate) fn quest_state(&self, quest_id: u16) -> i32 {
        self.quest_states
            .get(&quest_id)
            .copied()
            .map_or(2, i32::from)
    }

    pub(crate) fn set_quest_state_snapshot(&mut self, quest_id: u16, state: u8) {
        self.quest_states.insert(quest_id, state);
    }

    pub(crate) fn accept_script_quest(&mut self, quest_id: u16) -> bool {
        if self.quest_states.get(&quest_id).copied() == Some(0) {
            return false;
        }
        self.quest_states.insert(quest_id, 0);
        true
    }

    pub(crate) fn complete_script_quest(&mut self, quest_id: u16) -> bool {
        let Some(state) = self.quest_states.get_mut(&quest_id) else {
            return false;
        };
        *state = 1;
        true
    }

    pub(crate) fn remove_script_quest(&mut self, quest_id: u16) -> bool {
        self.quest_states.remove(&quest_id).is_some()
    }

    pub(crate) fn has_script_quest(&self, quest_id: u16) -> bool {
        self.quest_states.contains_key(&quest_id)
    }

    pub(crate) fn add_friend_state(&mut self, name: &[u8]) -> PlayerFriendAddOutcome {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        if self.friends.len() >= 0x28 {
            return PlayerFriendAddOutcome::LimitReached;
        }
        if self.friends.iter().any(|friend| friend.name == name) {
            return PlayerFriendAddOutcome::AlreadyPresent;
        }
        self.friends.push(PlayerFriend {
            name: name.to_vec(),
            online: true,
        });
        PlayerFriendAddOutcome::Added
    }

    pub(crate) fn delete_friend_state(&mut self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        let Some(index) = self.friends.iter().position(|friend| friend.name == name) else {
            return false;
        };
        self.friends.remove(index);
        true
    }

    pub(crate) fn has_friend(&self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.friends.iter().any(|friend| friend.name == name)
    }

    pub(crate) const fn team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) const fn is_charged(&self) -> bool {
        self.base_properties.charged
    }

    pub(crate) const fn set_charged(&mut self, charged: bool) {
        self.base_properties.charged = charged;
    }

    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    pub(crate) const fn is_faction_master(&self) -> bool {
        self.faction_id > 0 && self.faction_master_id == self.player_id()
    }

    pub(crate) fn faction_name(&self) -> &[u8] {
        &self.faction_name
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub(crate) const fn create_faction_operator(&self) -> bool {
        self.create_faction_operator
    }

    pub(crate) const fn set_create_faction_operator(&mut self, value: bool) {
        self.create_faction_operator = value;
    }

    pub(crate) const fn apply_join_faction_operator(&self) -> bool {
        self.apply_join_faction_operator
    }

    pub(crate) const fn set_apply_join_faction_operator(&mut self, value: bool) {
        self.apply_join_faction_operator = value;
    }

    pub(crate) const fn faction_declare_operator(&self) -> bool {
        self.faction_declare_operator
    }

    pub(crate) const fn set_faction_declare_operator(&mut self, value: bool) {
        self.faction_declare_operator = value;
    }

    pub(crate) const fn restore_faction_id(&mut self, faction_id: i32) {
        self.faction_id = faction_id;
    }

    pub(crate) fn restore_faction_identity(
        &mut self,
        faction_id: i32,
        faction_master_id: i32,
        faction_name: &[u8],
        union_id: i32,
    ) {
        self.faction_id = faction_id;
        self.faction_master_id = faction_master_id;
        self.faction_name.clear();
        self.faction_name.extend_from_slice(faction_name);
        self.union_id = union_id;
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    /// `CPlayer::SetScriptValue("btCountry")` сужает signed script value до
    /// исходного byte storage без country-range validation.
    pub(crate) const fn set_script_country(&mut self, requested: i32) -> i32 {
        self.country = requested as u8;
        self.country as i32
    }

    /// Ответ World `0x7FF01` меняет страну только для signed диапазона `1..4`;
    /// невалидное значение всё равно публикуется caller-ом в `0xC0301`.
    pub(crate) fn apply_world_country(&mut self, requested: i32) -> PlayerCountryMutationReport {
        let previous = self.country;
        if (1..5).contains(&requested) {
            self.country = requested as u8;
        }
        PlayerCountryMutationReport {
            player_id: self.player_id(),
            previous,
            requested,
            applied: self.country,
            changed: self.country != previous,
        }
    }

    /// Сохраняет player tail total-honor startup ветви: days сбрасывается
    /// всегда, weeks/months — по mask `2/4`, total не меняется, а ненулевой
    /// nobility rank требует точного `AdjustHonorRank` script-effect-а.
    pub(crate) fn reset_total_honor_eliminate(
        &mut self,
        reset_mask: u32,
    ) -> PlayerHonorResetReport {
        let report = PlayerHonorResetReport {
            player_id: self.player_id(),
            reset_mask,
            previous_days: self.base_properties.days_honor_eliminate,
            previous_weeks: self.base_properties.weeks_honor_eliminate,
            previous_months: self.base_properties.months_honor_eliminate,
            adjust_honor_rank_script: (self.base_properties.rank_of_nobility_id != 0)
                .then_some(b"scripts/circle/honorrank/adjusthonorrank.script"),
        };
        self.base_properties.days_honor_eliminate = 0;
        if reset_mask & 2 != 0 {
            self.base_properties.weeks_honor_eliminate = 0;
        }
        if reset_mask & 4 != 0 {
            self.base_properties.months_honor_eliminate = 0;
        }
        report
    }

    pub(crate) const fn server_region_id(&self) -> Option<i32> {
        self.server_region_id
    }

    pub(crate) const fn in_changing_server(&self) -> bool {
        self.in_changing_server
    }

    pub(crate) const fn in_changing_region(&self) -> bool {
        self.in_changing_region
    }

    pub(crate) const fn set_changing_state_snapshot(
        &mut self,
        in_changing_server: bool,
        in_changing_region: bool,
    ) {
        self.in_changing_server = in_changing_server;
        self.in_changing_region = in_changing_region;
    }

    /// Player-owned scalar tail локального `ChangeRegion`; фактическое
    /// membership перемещение остаётся deferred у `CServerRegion::AI`.
    pub(crate) fn stage_local_region_change(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        self.in_changing_server = false;
        self.in_changing_region = true;
        self.recreate_carriage = false;
        self.movement_shape_mut()
            .stage_region_change(region_id, tile_x, tile_y, direction);
    }

    pub(crate) fn begin_server_region_change(&mut self) {
        self.state_before_server_region_change = self.shape().get_state();
        self.movement_shape_mut().set_state(0);
        self.in_changing_server = true;
        self.in_changing_region = true;
        self.recreate_carriage = false;
    }

    pub(crate) fn cancel_server_region_change(&mut self) {
        if self.in_changing_server {
            let state = self.state_before_server_region_change;
            self.movement_shape_mut().set_state(state);
        }
        self.in_changing_server = false;
        self.in_changing_region = false;
    }

    pub(crate) fn apply_staged_local_region_change(&mut self) -> (i32, i32, i32, i32) {
        let destination = self.movement_shape_mut().apply_staged_region_change();
        self.server_region_id = Some(destination.0);
        destination
    }

    pub(crate) const fn current_progress(&self) -> PlayerProgress {
        self.current_progress
    }

    pub(crate) const fn set_current_progress_snapshot(&mut self, progress: PlayerProgress) {
        self.current_progress = progress;
    }

    pub(crate) const fn set_personal_shop_flag(&mut self, session_id: i32, plug_id: i32) {
        self.personal_shop_session_id = session_id;
        self.personal_shop_plug_id = plug_id;
    }

    pub(crate) const fn personal_shop_flag(&self) -> Option<(i32, i32)> {
        if self.personal_shop_session_id == 0 || self.personal_shop_plug_id == 0 {
            None
        } else {
            Some((self.personal_shop_session_id, self.personal_shop_plug_id))
        }
    }

    pub(crate) fn begin_equipment_session(
        &mut self,
        progress: PlayerProgress,
        lock_movement: bool,
    ) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = progress;
        if lock_movement {
            self.move_shape.set_moveable(false);
        }
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn attach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .add_listener(listener);
        let equipment = self.equipment.base_mut().base_mut().add_listener(listener);
        [packet, equipment]
    }

    pub(crate) fn detach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        let equipment = self
            .equipment
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        [packet, equipment]
    }

    pub(crate) const fn is_dead(&self) -> bool {
        CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) const fn set_god_mode(&mut self, enabled: bool) {
        self.move_shape.set_god(enabled);
    }

    pub(crate) const fn is_god_mode(&self) -> bool {
        self.move_shape.is_god()
    }

    pub(crate) fn release_goods_session_state(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::None;
        self.move_shape.set_moveable(true);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn begin_synthesis(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::Synthesis;
        self.move_shape.set_moveable(false);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn close_synthesis(&mut self) -> Option<GoodsSessionPlayerRelease> {
        (self.current_progress == PlayerProgress::Synthesis)
            .then(|| self.release_goods_session_state())
    }

    pub(crate) fn depot_password(&self) -> &[u8] {
        &self.depot_password
    }

    /// Exact `SetDepotPassword`: nullable C-string уже разрешена caller-ом;
    /// сохраняются только bytes до первого NUL.
    pub(crate) fn set_depot_password(&mut self, password: &[u8]) {
        let length = password
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(password.len());
        self.depot_password.clear();
        self.depot_password.extend_from_slice(&password[..length]);
    }

    pub(crate) fn unlock_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.unlock_if_authenticated(true);
        let _legacy_depot_result = self.depot.unlock_if_authenticated(true);
    }

    pub(crate) fn close_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.lock();
        let _legacy_depot_result = self.depot.lock();
        self.current_progress = PlayerProgress::None;
    }

    pub(crate) const fn bank_locked(&self) -> bool {
        self.bank.is_locked()
    }

    pub(crate) const fn depot_locked(&self) -> bool {
        self.depot.is_locked()
    }

    pub(crate) const fn base_properties(&self) -> PlayerBaseProperties {
        self.base_properties
    }

    pub(crate) const fn set_display_head_piece(&mut self, display: bool) {
        self.base_properties.display_head_piece = display;
    }

    pub(crate) const fn display_head_piece(&self) -> bool {
        self.base_properties.display_head_piece
    }

    pub(crate) const fn quest_enabled(&self) -> bool {
        self.base_properties.quest_enabled
    }

    pub(crate) const fn set_quest_enabled(&mut self, enabled: bool) {
        self.base_properties.quest_enabled = enabled;
    }

    pub(crate) const fn begin_quest_time(&mut self, now_seconds: i32, time_limit: i32) {
        self.base_properties.quest_time_begin = now_seconds;
        self.base_properties.quest_time_limit = time_limit;
    }

    pub(crate) const fn clear_quest_time(&mut self) {
        self.base_properties.quest_time_begin = 0;
        self.base_properties.quest_time_limit = 0;
    }

    pub(crate) const fn quest_time_remaining(&self, now_seconds: i32) -> i32 {
        if self.base_properties.quest_time_begin == 0 || self.base_properties.quest_time_limit == 0
        {
            return 0;
        }
        let remaining = self
            .base_properties
            .quest_time_begin
            .wrapping_add(self.base_properties.quest_time_limit)
            .wrapping_sub(now_seconds);
        if remaining < 0 { 0 } else { remaining }
    }

    pub(crate) const fn acknowledge_heartbeat(&mut self) {
        self.heart_request_sent = 0;
        self.heart_received = true;
    }

    pub(crate) fn encode_lei_ting(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(20 + self.lei_ting_things.len() * 8);
        payload.extend_from_slice(&self.base_properties.fy_enable_flags.to_le_bytes());
        payload.extend_from_slice(&self.base_properties.fy_energy.to_le_bytes());
        payload.extend_from_slice(&self.base_properties.lt_60_stamp.to_le_bytes());
        payload.extend_from_slice(&self.base_properties.lt_up_60_count.to_le_bytes());
        payload.extend_from_slice(&self.base_properties.remain_jing_li_dan_count.to_le_bytes());
        payload.extend_from_slice(&(self.lei_ting_things.len() as u32).to_le_bytes());
        for thing in &self.lei_ting_things {
            payload.extend_from_slice(&thing.thing_id.to_le_bytes());
            payload.extend_from_slice(&thing.count.to_le_bytes());
            payload.extend_from_slice(&thing.max_count.to_le_bytes());
            payload.extend_from_slice(&thing.point.to_le_bytes());
        }
        payload
    }

    pub(crate) fn decode_lei_ting(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerLeiTingDecodeBlock> {
        fn take<const N: usize>(
            source: &[u8],
            cursor: &mut usize,
            field: &'static str,
        ) -> Result<[u8; N], PlayerLeiTingDecodeBlock> {
            let offset = *cursor;
            let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
                return Err(PlayerLeiTingDecodeBlock {
                    field,
                    offset,
                    needed: N,
                    available: source.len().saturating_sub(offset),
                });
            };
            let value = bytes.try_into().expect("slice length проверена get range");
            *cursor += N;
            Ok(value)
        }
        fn read_u32(
            source: &[u8],
            cursor: &mut usize,
            field: &'static str,
        ) -> Result<u32, PlayerLeiTingDecodeBlock> {
            take::<4>(source, cursor, field).map(u32::from_le_bytes)
        }
        fn read_u16(
            source: &[u8],
            cursor: &mut usize,
            field: &'static str,
        ) -> Result<u16, PlayerLeiTingDecodeBlock> {
            take::<2>(source, cursor, field).map(u16::from_le_bytes)
        }

        self.base_properties.fy_enable_flags = read_u32(source, cursor, "dwfyenFlag")?;
        self.base_properties.fy_energy = read_u32(source, cursor, "dwfyEnergy")?;
        self.base_properties.lt_60_stamp = read_u32(source, cursor, "dwLT60Stamp")?;
        self.base_properties.lt_up_60_count = read_u16(source, cursor, "wLTUp60Cnt")?;
        self.base_properties.remain_jing_li_dan_count =
            read_u16(source, cursor, "wRemainJingLiDanCnt")?;
        self.lei_ting_things.clear();
        let count = read_u32(source, cursor, "m_listThing count")?;
        for _ in 0..count {
            self.lei_ting_things.push_back(PlayerLeiTingThing {
                thing_id: read_u16(source, cursor, "tagThing.wTID")?,
                count: read_u16(source, cursor, "tagThing.wCnt")?,
                max_count: read_u16(source, cursor, "tagThing.wMaxCnt")?,
                point: read_u16(source, cursor, "tagThing.wPoint")?,
            });
        }
        Ok(())
    }

    pub(crate) const fn change_fy_energy_flag(&mut self, index: u16) -> bool {
        let threshold_reached = match index {
            0 => self.base_properties.fy_energy >= 20,
            1 => self.base_properties.fy_energy >= 60,
            2 => self.base_properties.fy_energy >= 80,
            3 => self.base_properties.fy_energy >= 100,
            4 => self.base_properties.lt_up_60_count >= 4,
            5 => self.base_properties.lt_up_60_count >= 10,
            6 => self.base_properties.lt_up_60_count >= 16,
            7 => self.base_properties.lt_up_60_count >= 22,
            8 => self.base_properties.lt_up_60_count >= 28,
            _ => return false,
        };
        let mask = 1u32 << index;
        if !threshold_reached || self.base_properties.fy_enable_flags & mask != 0 {
            return false;
        }
        self.base_properties.fy_enable_flags |= mask;
        true
    }

    /// `GetOneThing((ushort)id)`: lookup намеренно сохраняет narrowing без
    /// последующей проверки исходного signed ID, как ветвь `GetThingCnt`.
    pub(crate) fn lei_ting_thing_count(&self, thing_id: i32) -> Option<u16> {
        let narrowed = thing_id as u16;
        self.lei_ting_things
            .iter()
            .find(|thing| thing.thing_id == narrowed)
            .map(|thing| thing.count)
    }

    /// Полный reached `AddThingCnt(id, count, false)`. В отличие от getter-а,
    /// setter после ushort lookup сравнивает сохранённый ID с исходным signed
    /// аргументом. Разрешено только строго положительное увеличение; энергия
    /// и суточные поля сохраняют wrapping-арифметику x86 owner-а. Closure —
    /// только CRT/local-time граница `AddLTUp60Cnt`; wire исполняет script
    /// caller после успешной mutation.
    pub(crate) fn set_lei_ting_thing_count(
        &mut self,
        thing_id: i32,
        requested_count: i32,
        next_daily_stamp_if_same_local_day: impl FnOnce(u32) -> Option<u32>,
    ) -> PlayerLeiTingThingCountOutcome {
        let narrowed = thing_id as u16;
        let Some(index) = self
            .lei_ting_things
            .iter()
            .position(|thing| thing.thing_id == narrowed)
        else {
            return PlayerLeiTingThingCountOutcome::Missing;
        };
        let current = self.lei_ting_things[index];
        if u32::from(current.thing_id) != thing_id as u32 {
            return PlayerLeiTingThingCountOutcome::Missing;
        }
        let difference = requested_count.wrapping_sub(i32::from(current.count));
        if difference < 1
            || difference > i32::from(current.max_count)
            || requested_count > i32::from(current.max_count)
        {
            return PlayerLeiTingThingCountOutcome::Rejected {
                current: current.count,
                requested: requested_count,
                maximum: current.max_count,
            };
        }

        self.lei_ting_things[index].count = requested_count as u16;
        let previous_energy = self.base_properties.fy_energy;
        self.base_properties.fy_energy =
            previous_energy.wrapping_add(u32::from(current.point).wrapping_mul(difference as u32));
        let daily_count_incremented = if self.base_properties.fy_energy >= 60 {
            if let Some(next_stamp) =
                next_daily_stamp_if_same_local_day(self.base_properties.lt_60_stamp)
            {
                self.base_properties.lt_up_60_count =
                    self.base_properties.lt_up_60_count.wrapping_add(1);
                self.base_properties.lt_60_stamp = next_stamp;
                true
            } else {
                false
            }
        } else {
            false
        };
        PlayerLeiTingThingCountOutcome::Updated {
            previous_count: current.count,
            current_count: requested_count as u16,
            previous_energy,
            current_energy: self.base_properties.fy_energy,
            daily_count_incremented,
        }
    }

    pub(crate) const fn honor_snapshot(&self) -> PlayerHonorSnapshot {
        PlayerHonorSnapshot {
            rank_of_nobility_id: self.base_properties.rank_of_nobility_id,
            appellation_id: self.base_properties.appellation_id,
            days_eliminate: self.base_properties.days_honor_eliminate,
            weeks_eliminate: self.base_properties.weeks_honor_eliminate,
            months_eliminate: self.base_properties.months_honor_eliminate,
            total_eliminate: self.base_properties.total_honor_eliminate,
        }
    }

    /// World acknowledgement `0x7FA16` подтверждает уже принятую honor-пару:
    /// все четыре счётчика увеличиваются независимо с DWORD wrapping.
    pub(crate) const fn acknowledge_honor_eliminate(&mut self) -> PlayerHonorEliminateMutation {
        let previous = [
            self.base_properties.days_honor_eliminate,
            self.base_properties.weeks_honor_eliminate,
            self.base_properties.months_honor_eliminate,
            self.base_properties.total_honor_eliminate,
        ];
        self.base_properties.days_honor_eliminate = previous[0].wrapping_add(1);
        self.base_properties.weeks_honor_eliminate = previous[1].wrapping_add(1);
        self.base_properties.months_honor_eliminate = previous[2].wrapping_add(1);
        self.base_properties.total_honor_eliminate = previous[3].wrapping_add(1);
        PlayerHonorEliminateMutation {
            player_id: self.player_id(),
            previous,
            current: [
                self.base_properties.days_honor_eliminate,
                self.base_properties.weeks_honor_eliminate,
                self.base_properties.months_honor_eliminate,
                self.base_properties.total_honor_eliminate,
            ],
        }
    }

    pub(crate) const fn request_change_appellation_state(&mut self, appellation_id: u32) {
        self.attempt_appellation_id = appellation_id;
    }

    pub(crate) fn add_appellation_state<Now>(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: Now,
    ) -> super::moveshape::UndeadStateMutation
    where
        Now: FnOnce() -> u32,
    {
        self.move_shape.add_undead_state(state_id, factory, now_ms)
    }

    pub(crate) fn delete_appellation_state(
        &mut self,
        state_id: u32,
    ) -> super::moveshape::UndeadStateMutation {
        self.move_shape.delete_undead_state(state_id)
    }

    pub(crate) fn get_appellation_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_undead_state(state_id)
    }

    pub(crate) fn activate_loaded_appellation_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<super::moveshape::UndeadState> {
        self.move_shape.activate_loaded_undead_states(now_ms)
    }

    pub(crate) fn appellation_state_tick(
        &mut self,
        now_ms: u32,
    ) -> (Vec<u32>, Vec<(u32, u32, u32)>) {
        let dead = self.is_dead();
        self.move_shape.undead_state_tick(now_ms, dead)
    }

    pub(crate) fn change_body_check(&self) -> bool {
        self.base_properties.mode == 0
            && !matches!(
                self.current_progress,
                PlayerProgress::Trading | PlayerProgress::OpenStall
            )
            && self.team_id == 0
            && !self.is_rider()
            && !self.has_pet()
            && self.uncreated_carriage.original_name.is_empty()
    }

    pub(crate) fn has_change_body_state(&self) -> bool {
        self.move_shape.active_change_body_state().is_some()
    }

    pub(crate) fn add_change_body_state(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
    ) -> super::chbystate::ChangeBodyMutation {
        let old_hotkeys = std::array::from_fn(|index| self.base_properties.hotkeys[index + 12]);
        let mutation =
            self.move_shape
                .add_change_body_state(state_id, factory, now_ms, old_hotkeys);
        let Some(state) = mutation.added.as_ref() else {
            return mutation;
        };
        self.clear_emotion_state();
        self.base_properties.mode = state.mode;
        for slot in 12..24 {
            self.base_properties.hotkeys[slot] = 0;
        }
        for (index, (skill_id, level)) in state.skills.iter().copied().enumerate() {
            if skill_id != 0 {
                let _ = self
                    .move_shape
                    .add_skill(u32::from(skill_id), i32::from(level), factory);
                self.base_properties.hotkeys[index + 12] = u32::from(skill_id) | 0x8000_0000;
            }
        }
        self.base_properties.hotkeys[17] = 0x8000_031f;
        mutation
    }

    pub(crate) fn delete_change_body_state(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
    ) -> super::chbystate::ChangeBodyMutation {
        let mutation = self.move_shape.delete_change_body_state(state_id);
        if let Some(state) = mutation.removed.as_ref() {
            self.base_properties.mode = 0;
            for (index, hotkey) in state.old_hotkeys.iter().copied().enumerate() {
                self.base_properties.hotkeys[index + 12] = hotkey;
            }
            for (skill_id, _) in state.skills {
                if skill_id != 0 {
                    let _ = self.move_shape.delete_skill(u32::from(skill_id), factory);
                }
            }
        }
        mutation
    }

    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_change_body_state(state_id)
    }

    pub(crate) fn activate_loaded_change_body_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<super::chbystate::ChangeBodyState> {
        self.move_shape.activate_loaded_change_body_states(now_ms)
    }

    pub(crate) fn expired_change_body_state_ids(&self, now_ms: u32) -> Vec<u32> {
        self.move_shape.expired_change_body_state_ids(now_ms)
    }

    pub(crate) fn change_body_region_transition_end_ids(&mut self) -> Vec<u32> {
        self.move_shape.change_body_region_transition_end_ids()
    }

    pub(crate) fn change_body_player_lost_end_ids(&mut self) -> Vec<u32> {
        self.move_shape.change_body_player_lost_end_ids()
    }

    pub(crate) fn change_body_death_end_ids(&self) -> Vec<u32> {
        self.move_shape.change_body_death_end_ids()
    }

    pub(crate) const fn is_rider(&self) -> bool {
        self.move_shape.has_ride_state()
    }

    pub(crate) const fn fight_state_count(&self) -> i32 {
        self.fight_state_count
    }

    pub(crate) fn begin_ride_state(
        &mut self,
        mount_type: u32,
        level: u32,
        role_limit: u32,
        goods_name: &[u8],
    ) -> Option<super::ridestate::RideState> {
        self.move_shape
            .begin_ride_state(super::ridestate::RideState::new(
                mount_type, level, role_limit, goods_name,
            ))
    }

    pub(crate) fn end_ride_state(&mut self) -> Option<super::ridestate::RideState> {
        self.move_shape.end_ride_state()
    }

    pub(crate) fn activate_loaded_ride_state(&mut self) -> Option<super::ridestate::RideState> {
        self.move_shape.activate_loaded_ride_state()
    }

    pub(crate) fn ride_goods_check_due(&self, now_ms: u32) -> bool {
        self.move_shape
            .ride_state()
            .is_some_and(|state| state.goods_check_due(now_ms))
    }

    /// Exact `CRideState::AI` packet scan: имя здесь не участвует, только
    /// addon mount type/level. Успех обновляет безопасный GUID cache вместо
    /// старого сырого `CGoods*`, но timestamp намеренно не меняется.
    pub(crate) fn refresh_ride_goods_cache(&mut self, factory: &CGoodsFactory) -> bool {
        let Some(state) = self.move_shape.ride_state() else {
            return false;
        };
        let (mount_type, level, cached_id) =
            (state.mount_type(), state.level(), state.cached_goods_id());
        let matches = |goods: &CGoods| {
            goods.addon_property_value(factory, GAP_MOUNT_TYPE, 1) == mount_type as i32
                && goods.addon_property_value(factory, GAP_MOUNT_LEVEL, 1) == level as i32
        };
        let found = self
            .packet
            .base()
            .find(cached_id)
            .filter(|goods| matches(goods))
            .or_else(|| {
                self.packet
                    .base()
                    .traversing_goods()
                    .find(|goods| matches(goods))
            })
            .map(|goods| goods.identity().ex_id);
        if let Some(state) = self.move_shape.ride_state_mut() {
            if let Some(goods_id) = found {
                state.set_cached_goods_id(goods_id);
            } else {
                state.clear_cached_goods_id();
            }
        }
        found.is_some()
    }

    fn ride_goods(&mut self, factory: &CGoodsFactory) -> Option<CGoods> {
        let state = self.move_shape.ride_state()?.clone();
        let base_index = factory.query_goods_id_by_original_name(Some(state.goods_name()));
        if base_index == 0 {
            return None;
        }
        let found = self
            .packet
            .base()
            .find(state.cached_goods_id())
            .filter(|goods| goods.base_properties_index() == base_index)
            .or_else(|| {
                self.packet
                    .base()
                    .traversing_goods()
                    .find(|goods| goods.base_properties_index() == base_index)
            })
            .cloned();
        if let Some(state) = self.move_shape.ride_state_mut() {
            if let Some(goods) = &found {
                state.set_cached_goods_id(goods.identity().ex_id);
            } else {
                state.clear_cached_goods_id();
            }
        }
        found
    }

    pub(crate) fn apply_change_body_properties(
        &mut self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        goods_factory: &CGoodsFactory,
    ) {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let signed = |target: &mut u32, value: i64| {
            *target = ((*target as i64) + value).clamp(1, i32::MAX as i64) as u32;
        };
        let percent = |target: &mut u32, value: i64| {
            let delta = ((*target as f64) * value as f64 * 0.01).round() as i64;
            *target = ((*target as i64) + delta).clamp(1, i32::MAX as i64) as u32;
        };
        for state in self.move_shape.undead_states() {
            let scalar = |current: u32, value: i64| -> i64 {
                if state.percentage {
                    ((current as f64) * value as f64 * 0.01).round() as i64
                } else {
                    value
                }
            };
            if state.percentage {
                percent(&mut properties.maximum_hp, i64::from(state.maximum_hp));
                percent(&mut properties.maximum_mp, i64::from(state.maximum_mp));
                percent(&mut properties.defense, i64::from(state.defense));
                percent(
                    &mut properties.element_resistance,
                    i64::from(state.element_resistance),
                );
            } else {
                signed(&mut properties.maximum_hp, i64::from(state.maximum_hp));
                signed(&mut properties.maximum_mp, i64::from(state.maximum_mp));
                signed(&mut properties.defense, i64::from(state.defense));
                signed(
                    &mut properties.element_resistance,
                    i64::from(state.element_resistance),
                );
            }
            let strength_delta = scalar(properties.strength, i64::from(state.strength));
            let dexterity_delta = scalar(properties.dexterity, i64::from(state.dexterity));
            let constitution_delta = scalar(properties.constitution, i64::from(state.constitution));
            let intelligence_delta = scalar(properties.intelligence, i64::from(state.intelligence));
            signed(&mut properties.strength, strength_delta);
            signed(&mut properties.dexterity, dexterity_delta);
            signed(&mut properties.constitution, constitution_delta);
            signed(&mut properties.intelligence, intelligence_delta);
            signed(
                &mut properties.maximum_attack,
                (strength_delta as f64 * f64::from(coefficients.str_to_max_attack[occupation]))
                    .round() as i64,
            );
            properties.burden = ((i64::from(properties.burden)
                + (strength_delta as f64 * f64::from(coefficients.str_to_burden[occupation]))
                    .round() as i64)
                .clamp(1, i64::from(u16::MAX))) as u16;
            signed(
                &mut properties.minimum_attack,
                (dexterity_delta as f64 * f64::from(coefficients.dex_to_min_attack[occupation]))
                    .round() as i64,
            );
            properties.reank = ((i64::from(properties.reank)
                + (dexterity_delta as f64 * f64::from(coefficients.dex_to_stiff[occupation]))
                    .round() as i64)
                .clamp(1, i64::from(u16::MAX))) as u16;
            signed(
                &mut properties.maximum_hp,
                (constitution_delta as f64 * f64::from(coefficients.con_to_max_hp[occupation]))
                    .round() as i64,
            );
            signed(
                &mut properties.defense,
                (constitution_delta as f64 * f64::from(coefficients.con_to_defense[occupation]))
                    .round() as i64,
            );
            properties.element_modify = properties.element_modify.wrapping_add(
                (intelligence_delta as f64 * f64::from(coefficients.int_to_element[occupation]))
                    .round() as i32,
            );
            signed(
                &mut properties.maximum_mp,
                (intelligence_delta as f64 * f64::from(coefficients.int_to_max_mp[occupation]))
                    .round() as i64,
            );
            signed(
                &mut properties.element_resistance,
                (intelligence_delta as f64 * f64::from(coefficients.int_to_resistant[occupation]))
                    .round() as i64,
            );
            signed(
                &mut properties.minimum_attack,
                i64::from(state.minimum_attack),
            );
            signed(
                &mut properties.maximum_attack,
                i64::from(state.maximum_attack),
            );
            properties.element_modify = properties
                .element_modify
                .wrapping_add(i32::from(state.element_modify));
            properties.blast_attack = properties
                .blast_attack
                .wrapping_add(state.blast_attack as u16);
            properties.blast_element_attack = properties
                .blast_element_attack
                .wrapping_add(state.blast_element_attack as u16);
            properties.cch = properties.cch.wrapping_add(state.cch as u16);
            properties.full_miss = properties.full_miss.wrapping_add(state.full_miss as u16);
            properties.attack_avoid = properties
                .attack_avoid
                .wrapping_add(state.attack_avoid as u16);
            properties.element_avoid = properties
                .element_avoid
                .wrapping_add(state.element_avoid as u16);
            properties.attack_speed = properties.attack_speed.wrapping_add(state.hit as u16);
            properties.dodge = properties.dodge.wrapping_add(state.dodge as u16);
        }
        for state in self.move_shape.extended_states() {
            let add = |target: &mut u32, value: u16| {
                *target = (*target)
                    .saturating_add(u32::from(value))
                    .min(i32::MAX as u32);
            };
            add(&mut properties.maximum_hp, state.maximum_hp);
            add(&mut properties.maximum_mp, state.maximum_mp);
            add(&mut properties.minimum_attack, state.minimum_attack);
            add(&mut properties.maximum_attack, state.maximum_attack);
            add(&mut properties.defense, state.defense);
            add(&mut properties.element_resistance, state.element_resistance);
            properties.element_modify = properties
                .element_modify
                .wrapping_add(i32::from(state.element_modify));
            properties.cch = properties.cch.wrapping_add(state.cch);
            properties.full_miss = properties.full_miss.wrapping_add(state.full_miss);
            properties.attack_avoid = properties.attack_avoid.wrapping_add(state.attack_avoid);
            properties.element_avoid = properties.element_avoid.wrapping_add(state.element_avoid);
            properties.attack_speed = properties.attack_speed.wrapping_add(state.hit);
            properties.dodge = properties.dodge.wrapping_add(state.dodge);
        }
        if let Some(state) = self.move_shape.active_change_body_state() {
            let add = |target: &mut u32, value: u32| {
                *target = u32::min((*target).saturating_add(value), i32::MAX as u32);
            };
            add(&mut properties.maximum_hp, state.maximum_hp);
            add(&mut properties.maximum_mp, state.maximum_mp);
            add(&mut properties.minimum_attack, state.minimum_attack);
            add(&mut properties.maximum_attack, state.maximum_attack);
            add(&mut properties.defense, state.defense);
            add(&mut properties.element_resistance, state.element_resistance);
            properties.cch = properties.cch.wrapping_add(state.cch);
            properties.blast_attack = properties.blast_attack.wrapping_add(state.blast_attack);
            properties.blast_element_attack = properties
                .blast_element_attack
                .wrapping_add(state.blast_element_attack);
        }
        if let Some(goods) = self.ride_goods(goods_factory) {
            apply_ride_goods_properties(
                &mut properties,
                &goods,
                goods_factory,
                coefficients,
                usize::from(self.base_properties.occupation).min(2),
            );
        }
        self.apply_recomputed_combat_properties(properties);
    }

    pub(crate) fn add_extended_state(
        &mut self,
        kind: super::exstate::ExtendedStateKind,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
    ) -> super::exstate::ExtendedStateMutation {
        self.move_shape
            .add_extended_state(kind, state_id, factory, now_ms)
    }

    pub(crate) fn delete_extended_state(
        &mut self,
        kind: super::exstate::ExtendedStateKind,
        state_id: u32,
    ) -> super::exstate::ExtendedStateMutation {
        self.move_shape.delete_extended_state(kind, state_id)
    }

    pub(crate) fn delete_extended_state_by_type(
        &mut self,
        state_type: u16,
    ) -> super::exstate::ExtendedStateMutation {
        self.move_shape.delete_extended_state_by_type(state_type)
    }

    pub(crate) const fn remain_jing_li_dan_count(&self) -> u16 {
        self.base_properties.remain_jing_li_dan_count
    }

    pub(crate) const fn set_remain_jing_li_dan_count(&mut self, count: u16) {
        self.base_properties.remain_jing_li_dan_count = count;
    }

    pub(crate) fn get_extended_state(
        &self,
        kind: super::exstate::ExtendedStateKind,
        state_id: u32,
    ) -> u32 {
        self.move_shape.get_extended_state(kind, state_id)
    }

    pub(crate) fn extended_state_tick(
        &mut self,
        now_ms: u32,
    ) -> (
        Vec<(super::exstate::ExtendedStateKind, u32)>,
        Vec<(super::exstate::ExtendedStateKind, u32, u32, u32)>,
    ) {
        self.move_shape.extended_state_tick(now_ms)
    }

    pub(crate) fn activate_loaded_extended_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<super::exstate::ExtendedState> {
        self.move_shape.activate_loaded_extended_states(now_ms)
    }

    pub(crate) fn realm_appellation_bonus_identity(
        &self,
    ) -> Option<super::skills::realmappellation::RealmBonusIdentity> {
        (self.realm_appellation_skill_id != UNKNOWN_SKILL_ID
            && (1..=4).contains(&self.realm_appellation_skill_level))
        .then_some(super::skills::realmappellation::RealmBonusIdentity {
            skill_id: self.realm_appellation_skill_id,
            level: self.realm_appellation_skill_level,
        })
    }

    pub(crate) const fn set_realm_appellation_bonus_identity(&mut self, skill_id: u32, level: i32) {
        self.realm_appellation_skill_id = skill_id;
        self.realm_appellation_skill_level = level;
    }

    pub(crate) fn realm_appellation_entitled(&self, appellation_id: u32) -> bool {
        super::skills::realmappellation::is_title(appellation_id)
            && self
                .move_shape
                .skill(appellation_id)
                .is_some_and(|skill| skill.level() > 0)
    }

    /// Достигнутая часть единого `m_mapNameValue/GetScriptValue` catalog.
    /// DWORD возвращаются теми же битами в signed script integer; неизвестное
    /// имя остаётся `None`, а сам GetMe преобразует его в legacy zero.
    pub(crate) fn script_value(&self, property: &[u8]) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"lRegionID") {
            Some(self.server_region_id().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lID") {
            Some(self.player_id())
        } else if property.eq_ignore_ascii_case(b"lTileX") {
            Some(self.shape().get_tile_x().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lTileY") {
            Some(self.shape().get_tile_y().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lDir") {
            Some(self.shape().get_direction())
        } else if property.eq_ignore_ascii_case(b"wState") {
            Some(i32::from(self.shape().get_state()))
        } else if property.eq_ignore_ascii_case(b"wAction") {
            Some(i32::from(self.shape().get_action()))
        } else if property.eq_ignore_ascii_case(b"btCountry") {
            Some(i32::from(self.country()))
        } else if property.eq_ignore_ascii_case(b"lPos") {
            Some(self.shape().get_position())
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.vigour() as i32)
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            Some(i32::from(self.level()))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.experience() as i32)
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            Some(i32::from(self.occupation()))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            Some(self.base_properties.appellation_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            Some(self.base_properties.rank_of_nobility_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            Some(self.base_properties.credit as i32)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            Some(self.base_properties.szl as i32)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            Some(self.contribution)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            Some(i32::from(self.base_properties.fairy_container_enabled))
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            Some(i32::from(self.base_properties.battle_fairy_enabled))
        } else {
            None
        }
    }

    /// Достигнутая writable-часть того же `m_mapNameValue/SetValue` catalog.
    /// Узкие поля сохраняют исходное integer narrowing, DWORD — все биты.
    pub(crate) fn set_script_value(&mut self, property: &[u8], value: i32) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"btCountry") {
            Some(self.set_script_country(value))
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.set_script_vigour(value))
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            self.base_properties.level = value as u8;
            Some(i32::from(self.base_properties.level))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            self.base_properties.sex = value as u8;
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.set_script_experience(value))
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            self.base_properties.occupation = value as u8;
            Some(i32::from(self.base_properties.occupation))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            self.base_properties.appellation_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            self.base_properties.rank_of_nobility_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            self.base_properties.credit = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            self.base_properties.szl = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            self.contribution = value;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            self.base_properties.fairy_container_enabled = value != 0;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            self.base_properties.battle_fairy_enabled = value != 0;
            Some(value)
        } else {
            None
        }
    }

    /// `ChangeValue` применяет wrapping arithmetic ширины фактического поля.
    /// Shipped GM-script использует исторический алиас `Experience` для
    /// canonical `dwExp`.
    pub(crate) fn change_script_value(&mut self, property: &[u8], delta: i32) -> Option<i32> {
        let canonical = if property.eq_ignore_ascii_case(b"Experience") {
            b"dwExp".as_slice()
        } else {
            property
        };
        let current = self.script_value(canonical)?;
        if canonical.eq_ignore_ascii_case(b"bFairyContainerEnabled")
            || canonical.eq_ignore_ascii_case(b"bBattleFairyEnabled")
        {
            let changed = i32::from(current.wrapping_add(delta) != 0);
            let _ = self.set_script_value(canonical, changed)?;
            return Some(changed);
        }
        self.set_script_value(canonical, current.wrapping_add(delta))
    }

    pub(crate) fn delete_realm_appellation_skill(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.delete_skill(skill_id, factory)
    }

    pub(crate) fn add_realm_appellation_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.add_skill(skill_id, level, factory)
    }

    pub(crate) const fn gods_battle_faction(&self) -> i32 {
        self.base_properties.gods_battle_faction
    }

    pub(crate) const fn level(&self) -> u8 {
        self.base_properties.level
    }

    pub(crate) const fn occupation(&self) -> u8 {
        self.base_properties.occupation
    }

    /// Cross-Game level relay сохраняет `SetLevel` mutation и отдельный
    /// caller-side faction publication; experience сбрасывается после неё.
    pub(crate) fn apply_remote_level(&mut self, level: u8) -> PlayerRemoteLevelMutation {
        let mutation = PlayerRemoteLevelMutation {
            player_id: self.player_id(),
            faction_id: self.faction_id,
            previous_level: self.base_properties.level,
            level,
        };
        self.base_properties.level = level;
        self.base_properties.experience = 0;
        mutation
    }

    pub(crate) fn add_remote_skill(
        &mut self,
        name: &[u8],
        level: u16,
        factory: &CSkillFactory,
    ) -> Option<PlayerRemoteSkillMutation> {
        let skill_id = factory.query_skill_id(Some(name));
        if skill_id == UNKNOWN_SKILL_ID {
            return None;
        }
        let legacy_result = self
            .move_shape
            .add_skill(skill_id, i32::from(level), factory);
        let skill = self.move_shape.skill(skill_id)?;
        Some(PlayerRemoteSkillMutation {
            skill_id,
            skill_level: skill.level(),
            legacy_result,
        })
    }

    pub(crate) fn set_script_skill_level(
        &mut self,
        name: &[u8],
        level: i32,
        factory: &CSkillFactory,
    ) -> Option<PlayerRemoteSkillMutation> {
        let skill_id = factory.query_skill_id(Some(name));
        if skill_id == UNKNOWN_SKILL_ID {
            return None;
        }
        let legacy_result = self.move_shape.add_skill(skill_id, level, factory);
        let skill = self.move_shape.skill(skill_id)?;
        Some(PlayerRemoteSkillMutation {
            skill_id,
            skill_level: skill.level(),
            legacy_result,
        })
    }

    pub(crate) fn delete_remote_skill(
        &mut self,
        name: &[u8],
        factory: &CSkillFactory,
    ) -> PlayerRemoteSkillMutation {
        let skill_id = factory.query_skill_id(Some(name));
        let legacy_result = self.move_shape.delete_skill(skill_id, factory);
        PlayerRemoteSkillMutation {
            skill_id,
            skill_level: 0,
            legacy_result,
        }
    }

    pub(crate) const fn szl(&self) -> u32 {
        self.base_properties.szl
    }

    pub(crate) const fn set_szl(&mut self, value: u32) {
        self.base_properties.szl = value;
    }

    pub(crate) const fn attempt_appellation_id(&self) -> u32 {
        self.attempt_appellation_id
    }

    pub(crate) const fn clear_attempt_appellation(&mut self) {
        self.attempt_appellation_id = 0;
    }

    pub(crate) const fn set_gods_battle_faction(&mut self, faction: i32) {
        self.base_properties.gods_battle_faction = faction;
    }

    /// Assembly/load boundary для persisted player tail; faction membership
    /// сам region восстанавливает только после фактического `AddObject`.
    pub(crate) const fn restore_gods_battle_state(&mut self, faction: i32, szl: u32) {
        self.base_properties.gods_battle_faction = faction;
        self.base_properties.szl = szl;
    }

    pub(crate) const fn restore_level_and_attempt_appellation(
        &mut self,
        level: u8,
        attempt_appellation_id: u32,
    ) {
        self.base_properties.level = level;
        self.attempt_appellation_id = attempt_appellation_id;
    }

    /// Exact `SetExploit`: signed CountryParam storage сравнивается как
    /// `unsigned long`, затем значение зажимается только сверху.
    pub(crate) const fn exploit(&self) -> u32 {
        self.base_properties.exploit
    }

    pub(crate) fn set_exploit(
        &mut self,
        requested: u32,
        maximum: i32,
    ) -> PlayerExploitMutationReport {
        let previous = self.base_properties.exploit;
        let applied = requested.min(maximum as u32);
        self.base_properties.exploit = applied;
        PlayerExploitMutationReport {
            player_id: self.player_id(),
            previous,
            requested,
            applied,
        }
    }

    /// Exact `SetValue("dwExploit", value)` из region reward path:
    /// generic property map пишет `DWORD` напрямую и не вызывает `SetExploit` clamp.
    pub(crate) fn set_exploit_property_value(
        &mut self,
        requested: u32,
    ) -> PlayerExploitMutationReport {
        let previous = self.base_properties.exploit;
        self.base_properties.exploit = requested;
        PlayerExploitMutationReport {
            player_id: self.player_id(),
            previous,
            requested,
            applied: requested,
        }
    }

    /// `OnPlayerTimgingStart` отсеивает action `ACT_DIED == 6`
    /// отдельно от health-based `CMoveShape::IsDied`.
    pub(crate) fn can_start_nation_war_timing(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Тот же double guard использует `ServerNationRegion::OnMonsterDamage`.
    pub(crate) fn can_attack_nation_monster(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) fn can_enter_gods_battle_contend(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Focused same-region branch `ChangeRegion`, которую вызывает
    /// `ServerNationRegion::KickOutAllPlayerToReturnPoint`.
    pub(crate) fn prepare_nation_relive(&mut self) {
        self.current_progress = PlayerProgress::None;
        self.recreate_carriage = false;
    }

    pub(crate) const fn movement_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: self.base_properties.health,
            figure: self.figure,
            current_area: None,
            area_width,
            area_height,
        }
    }

    pub(crate) const fn movement_shape_mut(&mut self) -> &mut CShape {
        self.move_shape.shape_mut()
    }

    pub(crate) fn force_move<Context: MoveShapeCommandContext>(
        &mut self,
        server_region: &mut CServerRegion,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
        context: &mut Context,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let facts = self.movement_position_facts(area_width, area_height);
        self.move_shape.force_move(
            Some(server_region),
            destination_x,
            destination_y,
            duration_ms,
            facts,
            around,
            context,
        )
    }

    pub(crate) const fn figure(&self) -> ShapeFigure {
        self.figure
    }

    pub(crate) const fn nation_relive_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        self.movement_position_facts(area_width, area_height)
    }

    pub(crate) const fn nation_relive_shape_mut(&mut self) -> &mut CShape {
        self.movement_shape_mut()
    }

    pub(crate) const fn combat_properties(&self) -> PlayerCombatProperties {
        self.combat_properties
    }

    /// Exact scalar checks `CanUseItem`; catalog/instance addon fallback
    /// остаётся у `CGoods`, а result-коды являются частью `0xBF709` wire.
    pub(crate) fn can_use_item(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_use_item_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    /// Exact `CPlayer::CanMountEquip`: для headgear сначала согласует persisted
    /// ordinary/battle-fairy enable flags с `GAP_BF_BATTLE_FAIRY`, затем
    /// возвращает те же requirement-коды `1..7` или magic success `9`.
    pub(crate) fn can_mount_equip(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_mount_equip_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    fn can_mount_equip_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        if factory
            .query_goods_base_properties(goods.base_properties_index())
            .is_some_and(|properties| properties.equip_place() == EQUIP_PLACE_HEADGEAR)
        {
            let battle_fairy_headgear =
                goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1;
            if (!base_properties.fairy_container_enabled
                && (!base_properties.battle_fairy_enabled || !battle_fairy_headgear))
                || (base_properties.fairy_container_enabled
                    && !base_properties.battle_fairy_enabled
                    && battle_fairy_headgear)
            {
                return 0;
            }
        }
        Self::can_use_item_from_properties(base_properties, combat_properties, goods, factory)
    }

    fn can_use_item_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        let required = |property| goods.addon_property_value(factory, property, 1) as u32;
        let level = required(GAP_ROLE_MINIMUM_LEVEL_LIMIT);
        if level != 0 && u32::from(base_properties.level) < level {
            return 1;
        }
        for (property, actual, result) in [
            (
                GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
                combat_properties.strength,
                2,
            ),
            (
                GAP_ROLE_MINIMUM_AGILITY_LIMIT,
                combat_properties.dexterity,
                3,
            ),
            (
                GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
                combat_properties.constitution,
                4,
            ),
            (
                GAP_ROLE_MINIMUM_WAKAN_LIMIT,
                combat_properties.intelligence,
                5,
            ),
        ] {
            let minimum = required(property);
            if minimum != 0 && actual < minimum {
                return result;
            }
        }
        let occupation = required(GAP_REQUIRE_OCCUPATION);
        if occupation != 0 && occupation != u32::from(base_properties.occupation) + 1 {
            return 6;
        }
        let gender = required(GAP_REQUIRE_GENDER);
        if gender != 0 && gender != u32::from(base_properties.sex) {
            return 7;
        }
        9
    }

    pub(crate) fn item_skill_level(&self, skill_id: u32) -> i32 {
        self.move_shape
            .skill(skill_id)
            .map_or(0, MoveShapeSkill::level)
    }

    pub(crate) fn learn_item_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.add_skill(skill_id, level, factory)
    }

    /// `ReUseSkillItem` success tail всегда пересоздаёт concrete skill перед
    /// `CItemSkill_2::SetItemPos`, даже когда уровень совпадает с текущим.
    pub(crate) fn replace_item_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
    ) -> bool {
        let _deleted = self.move_shape.delete_skill(skill_id, factory);
        self.move_shape.add_skill(skill_id, level, factory)
    }

    /// Owned tail четырёх `tagExpendableEffect` case-ов. Повторное применение
    /// сначала снимает прежнее значение, затем заменяет timer/value entry.
    pub(crate) fn apply_expendable_item_effect(
        &mut self,
        property_type: i32,
        value: i32,
        start_time_ms: u32,
        effect_time_ms: u32,
    ) -> i32 {
        let previous = self
            .expendable_effects
            .get(&property_type)
            .map_or(0, |effect| effect.value);
        match property_type {
            0x4a => {
                self.combat_properties.maximum_attack = self
                    .combat_properties
                    .maximum_attack
                    .wrapping_sub(previous as u32)
                    .wrapping_add(value as u32);
            }
            0x4b => {
                self.combat_properties.attack_speed = self
                    .combat_properties
                    .attack_speed
                    .wrapping_sub(previous as u16)
                    .wrapping_add(value as u16);
            }
            0x4c => {
                self.combat_properties.defense = self
                    .combat_properties
                    .defense
                    .wrapping_sub(previous as u32)
                    .wrapping_add(value as u32);
            }
            0x4d => {
                self.combat_properties.element_modify = self
                    .combat_properties
                    .element_modify
                    .wrapping_sub(previous)
                    .wrapping_add(value);
            }
            _ => return 0,
        }
        self.expendable_effects.insert(
            property_type,
            PlayerExpendableEffect {
                property_type,
                value,
                start_time_ms,
                effect_time_ms,
            },
        );
        match property_type {
            0x4a => self.combat_property_wire[0x20..0x24]
                .copy_from_slice(&self.combat_properties.maximum_attack.to_le_bytes()),
            0x4b => self.combat_property_wire[0x24..0x26]
                .copy_from_slice(&self.combat_properties.attack_speed.to_le_bytes()),
            0x4c => self.combat_property_wire[0x2c..0x30]
                .copy_from_slice(&self.combat_properties.defense.to_le_bytes()),
            0x4d => self.combat_property_wire[0x48..0x4c]
                .copy_from_slice(&self.combat_properties.element_modify.to_le_bytes()),
            _ => {}
        }
        match property_type {
            0x4a => self.combat_properties.maximum_attack as i32,
            0x4b => i32::from(self.combat_properties.attack_speed),
            0x4c => self.combat_properties.defense as i32,
            0x4d => self.combat_properties.element_modify,
            _ => 0,
        }
    }

    pub(crate) const fn combat_property_wire(&self) -> &[u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE] {
        &self.combat_property_wire
    }

    /// Selector `3009` читает не производную Rust-модель, а те же concrete
    /// `tagBaseProperty`/`tagProperty` slots, которые использует EXE. Поэтому
    /// неизвестные, но загруженные и пересчитанные поля сохраняются в выводе.
    pub(crate) fn all_properties_diagnostic_snapshot(
        &self,
    ) -> PlayerAllPropertiesDiagnosticSnapshot {
        let base = self.synchronized_base_property_wire();
        let current = &self.combat_property_wire;
        let base_u16 = |offset| u32::from(read_player_wire_u16(&base, offset));
        let base_u32 = |offset| read_player_wire_u32(&base, offset);
        let current_u16 = |offset| u32::from(read_player_wire_u16(current, offset));
        let current_u32 = |offset| read_player_wire_u32(current, offset);
        PlayerAllPropertiesDiagnosticSnapshot {
            name: self.shape().base_object().get_name().to_vec(),
            summary_words: [
                u32::from(self.occupation()),
                u32::from(self.level()),
                self.experience(),
                base_u32(BASE_HEALTH_OFFSET),
                current_u32(0x00),
                base_u32(BASE_MANA_OFFSET),
                current_u32(0x04),
                base_u16(0xac),
                current_u16(0x0a),
                base_u32(BASE_MAXIMUM_HP_OFFSET),
                base_u32(BASE_MAXIMUM_MP_OFFSET),
                base_u16(0xba),
                base_u16(BASE_PK_COUNT_OFFSET),
                base_u32(BASE_KILL_COUNT_OFFSET),
                self.money(),
            ],
            base_combat_words: [
                base_u32(BASE_STRENGTH_OFFSET),
                base_u32(BASE_DEXTERITY_OFFSET),
                base_u32(BASE_CONSTITUTION_OFFSET),
                base_u32(BASE_INTELLIGENCE_OFFSET),
                base_u32(0xcc),
                base_u32(0xd0),
                base_u16(0xd4),
                base_u16(0xd6),
                base_u16(0xd8),
                base_u32(0xdc),
                base_u16(0xe0),
                base_u16(0xe2),
                base_u32(0xe4),
                base_u16(0xe8),
                base_u16(0xea),
            ],
            current_combat_words: [
                current_u32(0x0c),
                current_u32(0x10),
                current_u32(0x14),
                current_u32(0x18),
                current_u32(0x1c),
                current_u32(0x20),
                current_u16(0x24),
                current_u16(0x26),
                current_u16(0x28),
                current_u32(0x2c),
                current_u16(0x30),
                i32::from(read_player_wire_u16(current, 0x32) as i16) as u32,
                current_u32(0x34),
                current_u16(0x38),
                current_u16(0x3a),
                current_u16(0x3c),
                current_u32(0x40),
                current_u16(0x44),
                current_u32(0x48),
                current_u16(0x4c),
            ],
        }
    }

    /// Граница восстановления exact `m_Property` из persisted player state.
    /// Последующие reached-пересчёты заменяют только известные поля layout.
    pub(crate) const fn restore_combat_property_wire(
        &mut self,
        wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    ) {
        self.combat_property_wire = wire;
    }

    /// Применяет результат виртуального `UpdateProperty` и синхронизирует
    /// подтверждённые поля 0x9c-byte `tagProperty`, сохраняя неизвестные байты.
    pub(crate) fn apply_recomputed_combat_properties(
        &mut self,
        properties: PlayerCombatProperties,
    ) {
        self.combat_properties = properties;
        let write_u16 = |wire: &mut [u8], offset: usize, value: u16| {
            wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let write_u32 = |wire: &mut [u8], offset: usize, value: u32| {
            wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        };
        write_u32(&mut self.combat_property_wire, 0x00, properties.maximum_hp);
        write_u32(&mut self.combat_property_wire, 0x04, properties.maximum_mp);
        write_u32(&mut self.combat_property_wire, 0x0c, properties.strength);
        write_u32(&mut self.combat_property_wire, 0x10, properties.dexterity);
        write_u32(
            &mut self.combat_property_wire,
            0x14,
            properties.constitution,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x18,
            properties.intelligence,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x1c,
            properties.minimum_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x20,
            properties.maximum_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x24,
            properties.attack_speed,
        );
        write_u16(&mut self.combat_property_wire, 0x2a, properties.hit);
        write_u16(&mut self.combat_property_wire, 0x30, properties.dodge);
        write_u16(&mut self.combat_property_wire, 0x28, properties.cch);
        write_u16(&mut self.combat_property_wire, 0x26, properties.burden);
        write_u32(&mut self.combat_property_wire, 0x2c, properties.defense);
        write_u32(
            &mut self.combat_property_wire,
            0x34,
            properties.element_resistance,
        );
        write_u16(&mut self.combat_property_wire, 0x38, properties.hp_recovery);
        write_u16(&mut self.combat_property_wire, 0x3a, properties.mp_recovery);
        write_u32(
            &mut self.combat_property_wire,
            0x48,
            properties.element_modify as u32,
        );
        write_u16(&mut self.combat_property_wire, 0x4c, properties.reank);
        write_u16(
            &mut self.combat_property_wire,
            0x4e,
            properties.attack_avoid,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x50,
            properties.element_avoid,
        );
        write_u16(&mut self.combat_property_wire, 0x52, properties.full_miss);
        write_u16(
            &mut self.combat_property_wire,
            0x54,
            properties.blast_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x56,
            properties.blast_element_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x5c,
            properties.blast_defense_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x68,
            properties.full_miss_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x6c,
            properties.critical_rate_bits,
        );
    }

    pub(crate) const fn stat_allocation_state(&self) -> PlayerStatAllocationState {
        PlayerStatAllocationState {
            sex: self.base_properties.sex,
            occupation: self.base_properties.occupation,
            remain_point: self.base_properties.remain_point,
            base_maximum_hp: self.base_properties.base_maximum_hp,
            base_maximum_mp: self.base_properties.base_maximum_mp,
            base_strength: self.base_properties.base_strength,
            base_dexterity: self.base_properties.base_dexterity,
            base_constitution: self.base_properties.base_constitution,
            base_intelligence: self.base_properties.base_intelligence,
        }
    }

    /// Восстанавливает owned поля `m_BaseProperty`, участвующие в client
    /// allocation `0x8FA01`; полный decoder игрока остаётся отдельным owner-ом.
    pub(crate) const fn restore_stat_allocation_state(&mut self, state: PlayerStatAllocationState) {
        self.base_properties.sex = state.sex;
        self.base_properties.occupation = state.occupation;
        self.base_properties.remain_point = state.remain_point;
        self.base_properties.base_maximum_hp = state.base_maximum_hp;
        self.base_properties.base_maximum_mp = state.base_maximum_mp;
        self.base_properties.base_strength = state.base_strength;
        self.base_properties.base_dexterity = state.base_dexterity;
        self.base_properties.base_constitution = state.base_constitution;
        self.base_properties.base_intelligence = state.base_intelligence;
    }

    /// Exact mutation-tail `0x8FA01`: DEX/CON/INT используют legacy STR gate;
    /// неизвестный selector всё равно расходует одно очко и ведёт к recompute.
    pub(crate) fn allocate_stat_point(
        &mut self,
        selector: u8,
        constitution_hp: u16,
        intelligence_mp: u16,
    ) -> Option<PlayerStatAllocationMutation> {
        if self.base_properties.remain_point == 0 {
            return None;
        }
        let previous = self.stat_allocation_state();
        let strength_gate = self.base_properties.base_strength < i32::MAX as u32;
        let stat_changed = match selector {
            0 if strength_gate => {
                self.base_properties.base_strength =
                    self.base_properties.base_strength.wrapping_add(1);
                true
            }
            1 if strength_gate => {
                self.base_properties.base_dexterity =
                    self.base_properties.base_dexterity.wrapping_add(1);
                true
            }
            2 => {
                if strength_gate {
                    self.base_properties.base_constitution =
                        self.base_properties.base_constitution.wrapping_add(1);
                }
                self.base_properties.base_maximum_hp = self
                    .base_properties
                    .base_maximum_hp
                    .wrapping_add(u32::from(constitution_hp));
                strength_gate
            }
            3 => {
                if strength_gate {
                    self.base_properties.base_intelligence =
                        self.base_properties.base_intelligence.wrapping_add(1);
                }
                self.base_properties.base_maximum_mp = self
                    .base_properties
                    .base_maximum_mp
                    .wrapping_add(u32::from(intelligence_mp));
                strength_gate
            }
            _ => false,
        };
        self.base_properties.remain_point = self.base_properties.remain_point.wrapping_sub(1);
        Some(PlayerStatAllocationMutation {
            player_id: self.player_id(),
            selector,
            stat_changed,
            previous,
            current: self.stat_allocation_state(),
        })
    }

    pub(crate) const fn pk_permissions(&self) -> PlayerPkPermissions {
        PlayerPkPermissions {
            player: self.base_properties.pk_normal,
            teammate: self.base_properties.pk_team,
            guild_member: self.base_properties.pk_union,
            criminal: self.base_properties.pk_badman,
            country: self.base_properties.pk_country,
        }
    }

    /// Граница восстановления пяти persisted `bPk_*` перед skill/AI use.
    pub(crate) const fn restore_pk_permissions(&mut self, permissions: PlayerPkPermissions) {
        self.base_properties.pk_normal = permissions.player;
        self.base_properties.pk_team = permissions.teammate;
        self.base_properties.pk_union = permissions.guild_member;
        self.base_properties.pk_badman = permissions.criminal;
        self.base_properties.pk_country = permissions.country;
    }

    /// Exact selector `0x8FA05`; неизвестное signed-char значение не меняет
    /// state, но caller уже прочитал оба входных байта.
    pub(crate) fn set_pk_permission(
        &mut self,
        selector: i8,
        requested: bool,
    ) -> PlayerPkPermissionMutation {
        let previous = self.pk_permissions();
        let recognized = match selector {
            0 => {
                self.base_properties.pk_normal = requested;
                true
            }
            1 => {
                self.base_properties.pk_team = requested;
                true
            }
            2 => {
                self.base_properties.pk_union = requested;
                true
            }
            3 => {
                self.base_properties.pk_badman = requested;
                true
            }
            4 => {
                self.base_properties.pk_country = requested;
                true
            }
            _ => false,
        };
        let current = self.pk_permissions();
        PlayerPkPermissionMutation {
            player_id: self.player_id(),
            selector,
            requested,
            recognized,
            changed: previous != current,
            previous,
            current,
        }
    }

    /// Exact `GetCurBurden`: только equipment, packet и hand, в исходном
    /// wrapping-порядке. Временные auction/fairy/session containers не входят.
    pub(crate) fn current_burden(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .contents_weight(factory)
            .wrapping_add(self.packet.base().contents_weight(factory))
            .wrapping_add(self.hand.contents_weight(factory))
    }

    pub(crate) const fn ci_qing_open(&self) -> bool {
        self.ci_qing_open
    }

    pub(crate) fn ci_qing_list(&self) -> impl ExactSizeIterator<Item = u32> + '_ {
        self.ci_qing_list.iter().copied()
    }

    pub(crate) fn restore_ci_qing_entry(&mut self, base_index: u32) -> bool {
        self.ci_qing_list.insert(base_index)
    }

    pub(crate) fn ci_qing_property_snapshot(
        &self,
    ) -> (&BTreeMap<u32, u32>, &BTreeMap<u32, u32>, u32) {
        (
            &self.ci_qing_add_values,
            &self.ci_qing_tao_zhuang_add_values,
            self.tao_zhuang_id,
        )
    }

    pub(crate) fn apply_ci_qing_property_snapshot(
        &mut self,
        add_values: BTreeMap<u32, u32>,
        tao_zhuang_add_values: BTreeMap<u32, u32>,
        tao_zhuang_id: u32,
    ) {
        self.ci_qing_add_values = add_values;
        self.ci_qing_tao_zhuang_add_values = tao_zhuang_add_values;
        self.tao_zhuang_id = tao_zhuang_id;
    }

    pub(crate) fn ci_qing_property_result(&self) -> BTreeMap<u32, u32> {
        let mut result = self.ci_qing_add_values.clone();
        for (&property, &value) in &self.ci_qing_tao_zhuang_add_values {
            let current = result.entry(property).or_default();
            *current = current.wrapping_add(value);
        }
        result
    }

    pub(crate) fn update_ci_qing_property_difference(
        previous: &BTreeMap<u32, u32>,
        current: &BTreeMap<u32, u32>,
    ) -> Option<BTreeMap<u32, u32>> {
        if previous.len() != current.len() {
            return None;
        }
        let mut destination = BTreeMap::new();
        for ((_, &previous), (&property, &current)) in previous.iter().zip(current) {
            destination.insert(property, current.saturating_sub(previous));
        }
        Some(destination)
    }

    pub(crate) fn check_item_in_packet(&self, base_index: u32) -> u32 {
        if base_index == 0 {
            return 0;
        }
        self.packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .fold(0u32, |total, goods| total.wrapping_add(goods.amount()))
    }

    /// Exact insertion-order `remove_item_in_packet`: каждый stack проходит
    /// через семантику `DeleteGoods(PEI_PACKET, ..., remaining, false)`.
    pub(crate) fn remove_item_in_packet(
        &mut self,
        base_index: u32,
        requested: u32,
    ) -> Vec<CiQingPacketConsumption> {
        if base_index == 0 || requested == 0 {
            return Vec::new();
        }
        let candidates: Vec<_> = self
            .packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .map(|goods| (goods.identity(), goods.amount()))
            .collect();
        let player_id = self.player_id();
        let mut removed_amount = 0u32;
        let mut consumptions = Vec::new();
        for (identity, previous_amount) in candidates {
            let remaining_request = requested.wrapping_sub(removed_amount);
            if remaining_request == 0 {
                break;
            }
            let position = self
                .packet
                .query_goods_position(identity.ex_id)
                .unwrap_or_default();
            let consumed = previous_amount.min(remaining_request);
            if consumed == 0 {
                continue;
            }
            let remaining_amount = previous_amount.wrapping_sub(consumed);
            let removal = if remaining_amount == 0 {
                self.packet.remove_goods(identity.ex_id)
            } else {
                let position = self.packet.query_goods_position(identity.ex_id);
                if let Some(goods) =
                    position.and_then(|position| self.packet.get_goods_mut(position))
                {
                    goods.set_amount(remaining_amount);
                }
                None
            };
            removed_amount = removed_amount.wrapping_add(consumed);
            consumptions.push(CiQingPacketConsumption {
                player_id,
                goods: identity,
                position,
                previous_amount,
                remaining_amount,
                removal,
            });
        }
        consumptions
    }

    pub(crate) fn remove_packet_goods_by_id(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<CiQingPacketConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.packet.base().find(goods_id)?;
        let identity = goods.identity();
        let position = self.packet.query_goods_position(goods_id)?;
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.packet.remove_goods(goods_id)
        } else {
            let position = self.packet.query_goods_position(goods_id)?;
            self.packet
                .get_goods_mut(position)?
                .set_amount(remaining_amount);
            None
        };
        Some(CiQingPacketConsumption {
            player_id: self.player_id(),
            goods: identity,
            position,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    /// Player-side `AddGoodsToPacket`: успешный add забирает ownership из
    /// входного vector, rejected/несовместимый stack остаётся у caller-а.
    pub(crate) fn add_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        self.add_goods_to_packet_with_progress(
            goods,
            factory,
            encode_old_client,
            owner_progress_allows,
        )
    }

    /// Billing Increment response приходит при `PROGRESS_INCREMENT`, но
    /// исходный `BillOfIncShop` добавляет batch напрямую в packet и не
    /// применяет progress-lock к stack merge.
    pub(crate) fn add_increment_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(goods, factory, encode_old_client, true)
    }

    /// Script `2249` выполняется при занятом script progress, но native owner
    /// после созревания добавляет replacement напрямую в packet.
    pub(crate) fn add_script_fairy_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(goods, factory, encode_old_client, true)
    }

    /// `GetPreciousItem` исполняется внутри script progress, но native owner
    /// также добавляет награду напрямую и не применяет ordinary progress-lock.
    pub(crate) fn add_precious_box_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(goods, factory, encode_old_client, true)
    }

    /// `CTrader::Trade` добавляет contrary goods при
    /// `PROGRESS_TRADING`; этот owner намеренно обходит общий progress-lock,
    /// как прямой packet `Add` исходной функции.
    pub(crate) fn add_traded_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(goods, factory, encode_old_client, true)
    }

    /// NPC shop добавляет batch напрямую при `PROGRESS_SHOPPING`.
    pub(crate) fn add_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(goods, factory, encode_old_client, true)
    }

    /// Обратная половина `CTrader::RollBack`: отменяет уже выполненный
    /// contrary packet add, включая direct stack merge, и возвращает client
    /// consumption fact. Сам исходный goods caller хранит отдельно до commit.
    pub(crate) fn rollback_traded_packet_addition(
        &mut self,
        addition: &CiQingPacketAddition,
        original_amount: u32,
    ) -> Option<CiQingPacketConsumption> {
        let position = addition.position?;
        match &addition.outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let removed = self.packet.remove_goods(added.identity.ex_id)?;
                let taken = match removed {
                    VolumeGoodsRemoveOutcome::Removed(taken)
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                };
                let removed = match taken {
                    AmountLimitGoodsTaken::Removed(removed) => removed,
                    AmountLimitGoodsTaken::Split(_) => return None,
                };
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: removed.goods.identity(),
                    position,
                    previous_amount: removed.amount,
                    remaining_amount: 0,
                    removal: None,
                })
            }
            VolumeGoodsAddOutcome::Stack(
                super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                    target,
                    amount,
                },
            ) if *amount == original_amount => {
                let goods = self.packet.get_goods_mut(position)?;
                if goods.identity() != *target || goods.amount() < original_amount {
                    return None;
                }
                let previous_amount = goods.amount();
                let remaining_amount = previous_amount.wrapping_sub(original_amount);
                goods.set_amount(remaining_amount);
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: *target,
                    position,
                    previous_amount,
                    remaining_amount,
                    removal: None,
                })
            }
            _ => None,
        }
    }

    fn add_goods_to_packet_with_progress(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        owner_progress_allows: bool,
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let player_id = self.player_id();
        let mut additions = Vec::new();
        let mut remaining = Vec::new();
        for goods in goods {
            let source = goods.identity();
            let mut incoming = Some(goods);
            let outcome = self
                .packet
                .add_goods(&mut incoming, factory, owner_progress_allows);
            let (old_client_payload, resulting_amount) = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => {
                    let stored = self
                        .packet
                        .base()
                        .find(added.identity.ex_id)
                        .expect("успешный packet add сохранил новый goods");
                    (Some(encode_old_client(stored)), Some(stored.amount()))
                }
                VolumeGoodsAddOutcome::Stack(stack) => {
                    let target = match stack {
                        super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                            target,
                            ..
                        } => self.packet.base().find(target.ex_id),
                        _ => None,
                    };
                    (None, target.map(CGoods::amount))
                }
                VolumeGoodsAddOutcome::Rejected(_) => (None, None),
            };
            let position = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => added.position,
                VolumeGoodsAddOutcome::Stack(
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    },
                ) => self.packet.query_goods_position(target.ex_id),
                _ => None,
            };
            additions.push(CiQingPacketAddition {
                player_id,
                source,
                position,
                outcome,
                old_client_payload,
                resulting_amount,
            });
            if let Some(goods) = incoming {
                remaining.push(goods);
            }
        }
        (additions, remaining)
    }

    pub(crate) fn ci_qing_compose_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing_compose.get_goods(position)
    }

    pub(crate) fn ci_qing_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing.get_goods(position)
    }

    pub(crate) fn ci_qing_goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.ci_qing.goods_amount(factory)
    }

    pub(crate) fn add_goods_to_ci_qing(
        &mut self,
        goods: CGoods,
        position: u32,
        compose_container: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (CiQingContainerAddition, Option<CGoods>) {
        let player_id = self.player_id();
        let source = goods.identity();
        let container_extend_id = if compose_container { 17 } else { 16 };
        let container = if compose_container {
            &mut self.ci_qing_compose
        } else {
            &mut self.ci_qing
        };
        let mut incoming = Some(goods);
        let outcome = container.add_goods_at(
            position,
            &mut incoming,
            factory,
            self.current_progress == PlayerProgress::None,
        );
        let (old_client_payload, resulting_amount) = match &outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let stored = container
                    .base()
                    .find(added.identity.ex_id)
                    .expect("успешный CiQing add сохранил goods");
                (Some(encode_old_client(stored)), Some(stored.amount()))
            }
            VolumeGoodsAddOutcome::Stack(stack) => {
                let target = match stack {
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    } => container.base().find(target.ex_id),
                    _ => None,
                };
                (None, target.map(CGoods::amount))
            }
            VolumeGoodsAddOutcome::Rejected(_) => (None, None),
        };
        (
            CiQingContainerAddition {
                player_id,
                container_extend_id,
                position,
                source,
                outcome,
                old_client_payload,
                resulting_amount,
            },
            incoming,
        )
    }

    pub(crate) fn remove_ci_qing_compose_goods(
        &mut self,
        position: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing_compose.get_goods(position)?;
        let identity = goods.identity();
        let amount = goods.amount();
        let removal = self.ci_qing_compose.remove_goods(identity.ex_id)?;
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 17,
            position,
            goods: identity,
            previous_amount: amount,
            remaining_amount: 0,
            removal: Some(removal),
        })
    }

    pub(crate) fn remove_ci_qing_goods(
        &mut self,
        position: u32,
        requested: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing.get_goods(position)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        if consumed == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.ci_qing.remove_goods(identity.ex_id)
        } else {
            self.ci_qing
                .get_goods_mut(position)
                .map(|goods| goods.set_amount(remaining_amount));
            None
        };
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 16,
            position,
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_hand_goods(&self) -> Option<&CGoods> {
        self.hand.get_goods(0)
    }

    pub(crate) fn hotkey(&self, slot: u8) -> Option<u32> {
        self.base_properties.hotkeys.get(usize::from(slot)).copied()
    }

    pub(crate) fn set_hotkey(&mut self, slot: u8, value: u32) -> bool {
        let Some(hotkey) = self.base_properties.hotkeys.get_mut(usize::from(slot)) else {
            return false;
        };
        *hotkey = value;
        true
    }

    pub(crate) const fn last_operated_goods(&self) -> (u32, u32) {
        (
            self.last_operated_container,
            self.last_operated_goods_position,
        )
    }

    pub(crate) fn record_last_operated_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
    ) -> (u32, u32) {
        let previous = self.last_operated_goods();
        self.last_operated_container = source_extend_id as u32;
        self.last_operated_goods_position = source_position;
        previous
    }

    /// Storage core назначения hotkey из hand. Packet/hand/wallet/YuanBao
    /// замкнуты на owned containers; equipment для consumable доказательно
    /// отвергается до mutation.
    pub(crate) fn return_hotkey_hand_goods(
        &mut self,
        factory: &CGoodsFactory,
    ) -> HotkeyHandTransferReport {
        let (source_container_extend_id, source_position) = self.last_operated_goods();
        let mut report = HotkeyHandTransferReport {
            source_container_extend_id,
            source_position,
            goods: None,
            hand_removal: None,
            packet_adds: Vec::new(),
            currency_adds: Vec::new(),
            hand_rollback: None,
            outcome: HotkeyHandTransferOutcome::MissingHandGoods,
        };
        let Some(hand_goods) = self.hand.get_goods(0) else {
            return report;
        };
        report.goods = Some(hand_goods.identity());
        let Some(properties) =
            factory.query_goods_base_properties(hand_goods.base_properties_index())
        else {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        };
        if properties.goods_type() != GOODS_TYPE_CONSUMABLE {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        }
        if !(1..=5).contains(&source_container_extend_id) {
            report.outcome = HotkeyHandTransferOutcome::UnsupportedSource;
            return report;
        }

        let removed = self
            .hand
            .remove_goods(hand_goods.identity().ex_id)
            .expect("unlocked hand goods проверен перед synchronous remove");
        report.hand_removal = Some(HotkeyHandOwnershipEvent {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position,
            amount: removed.amount,
            listeners: removed.listeners.clone(),
        });
        let mut incoming = Some(removed.goods);
        if source_container_extend_id == 1 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.packet_adds.push(self.packet.add_goods_at(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.packet_adds.push(self.packet.add_goods(
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        } else if source_container_extend_id == 3 {
            let goods = incoming.take().expect("removed hand goods остаётся owned");
            match self.hand.add_goods(goods, factory) {
                Ok(added) => report.hand_rollback = Some(added),
                Err(goods) => incoming = Some(goods),
            }
        } else if source_container_extend_id == 4 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.wallet.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.wallet.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        } else if source_container_extend_id == 5 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.yuan_bao.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.yuan_bao.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        }

        if incoming.is_none() {
            if source_container_extend_id == 4 {
                self.money = self.wallet.currency_amount();
            }
            report.outcome = HotkeyHandTransferOutcome::Moved;
            return report;
        }
        let goods = incoming
            .take()
            .expect("failed destination сохраняет incoming");
        match self.hand.add_goods(goods, factory) {
            Ok(added) => {
                report.hand_rollback = Some(added);
                report.outcome = HotkeyHandTransferOutcome::RolledBack;
            }
            Err(goods) => {
                report.goods = Some(goods.identity());
                drop(goods);
                report.outcome = HotkeyHandTransferOutcome::GarbageCollected;
            }
        }
        report
    }

    pub(crate) fn destroy_hand_goods(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<GoodsDestroyHandConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.hand.find(goods_id)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let removed_amount = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(removed_amount);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(goods_id)
        } else {
            self.hand.find_mut(goods_id)?.set_amount(remaining_amount);
            None
        };
        Some(GoodsDestroyHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            removed_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn remove_ci_qing_hand_goods(&mut self) -> Option<CiQingHandConsumption> {
        let goods = self.hand.get_goods(0)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        if previous_amount == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(1);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(identity.ex_id)
        } else {
            self.hand.find_mut(identity.ex_id).map(|goods| {
                goods.set_amount(remaining_amount);
            });
            None
        };
        Some(CiQingHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_mount_facts(&self, factory: &CGoodsFactory) -> Option<(u32, u32, u32)> {
        let goods = self.ci_qing_hand_goods()?;
        if goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
            > i32::from(self.level())
        {
            return None;
        }
        Some((
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 1) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 2) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY2, 1) as u32,
        ))
    }

    pub(crate) const fn contend_state(&self) -> bool {
        self.contend_state
    }

    pub(crate) fn apply_client_direction(&mut self, direction: u8) -> i32 {
        self.move_shape
            .shape_mut()
            .set_direction(i32::from(direction));
        self.move_shape.shape().get_direction()
    }

    pub(crate) fn clear_emotion_state(&mut self) {
        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
    }

    /// Exact reached `PerformEmotion` state: state очищается до guards;
    /// repeated emotion запоминается, но around publication выполняется для
    /// любого разрешённого AI/жизни вызова.
    pub(crate) fn perform_emotion_state(
        &mut self,
        emotion_id: i32,
        repeated: bool,
        now_ms: u32,
        ai_available: bool,
        ai_has_target: bool,
    ) -> bool {
        self.clear_emotion_state();
        if self.is_dead() || !ai_available || ai_has_target {
            return false;
        }
        if repeated {
            self.emotion_index = emotion_id;
            self.emotion_timestamp_ms = now_ms;
        }
        true
    }

    /// State-часть exact `SetContendState`: unchanged setter не публикуется.
    /// `0xBFF28` собирает и маршрутизирует caller после успешной мутации.
    pub(crate) const fn set_contend_state(&mut self, contend_state: bool) -> bool {
        if self.contend_state == contend_state {
            return false;
        }
        self.contend_state = contend_state;
        true
    }

    pub(crate) const fn city_war_died_state_time_ms(&self) -> i32 {
        self.city_war_died_state_time_ms
    }

    /// Script `9313` читает этот byte напрямую без вычисления аргументов.
    pub(crate) const fn is_nation_war_player_weak(&self) -> bool {
        self.city_war_died_state
    }

    pub(crate) const fn died_state_start_time_ms(&self) -> u32 {
        self.died_state_start_time_ms
    }

    /// Direct assignment из `OnDied`: original не вызывает setter и поэтому
    /// не посылает `0xBFF2B`; clock стартует только для positive duration.
    pub(crate) const fn begin_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
        }
    }

    /// Достигнутый decode-tail восстановления player: persisted duration
    /// запускает новый local clock, а action `ACT_DIED == 6` оставляет state
    /// выключенным до `OnRelive`.
    pub(crate) fn restore_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
            if self.shape().get_action() != 6 {
                self.city_war_died_state = true;
            }
        }
    }

    /// State-часть exact `SetCityWarDiedStateTime`; caller использует return
    /// как gate `0xBFF2B`, который допустим лишь при active died state.
    pub(crate) const fn set_city_war_died_state_time_ms(&mut self, time_ms: i32) -> bool {
        if self.city_war_died_state_time_ms == time_ms {
            return false;
        }
        self.city_war_died_state_time_ms = time_ms;
        self.city_war_died_state
    }

    /// Exact `SetCityWarDiedState` всегда пишет state и всегда публикует обе
    /// копии `0xBFF2A`, даже если значение не изменилось.
    pub(crate) const fn set_city_war_died_state(&mut self, died_state: bool) {
        self.city_war_died_state = died_state;
    }

    pub(crate) const fn restart_died_state_clock(&mut self, now_ms: u32) {
        self.died_state_start_time_ms = now_ms;
    }

    pub(crate) const fn contribution(&self) -> i32 {
        self.contribution
    }

    pub(crate) const fn money(&self) -> u32 {
        self.money
    }

    pub(crate) fn decrease_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
    ) -> PlayerMoneyDecrease {
        let previous = self.wallet.currency_amount();
        let outcome = self.wallet.decrease_currency(requested, factory);
        self.money = self.wallet.currency_amount();
        PlayerMoneyDecrease {
            previous,
            current: self.money,
            outcome,
        }
    }

    pub(crate) fn increase_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> super::container::cwallet::CurrencyIncreaseOutcome {
        let mut created_currency = Some(created_currency);
        let outcome = self
            .wallet
            .increase_currency(requested, factory, move |_, _| {
                created_currency.take().unwrap_or_default()
            });
        self.money = self.wallet.currency_amount();
        outcome
    }

    pub(crate) fn yuan_bao(&self) -> u32 {
        self.yuan_bao.currency_amount()
    }

    /// State-owner exact `SetYuanBao`: однослотовый currency container
    /// сохраняет create/increase/decrease/delete outcome для сетевого caller-а.
    pub(crate) fn set_yuan_bao(
        &mut self,
        current: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerYuanBaoChange {
        let previous = self.yuan_bao.currency_amount();
        let outcome = if previous < current {
            let mut created_currency = Some(created_currency);
            PlayerYuanBaoChangeOutcome::Increased(self.yuan_bao.increase_currency(
                current.wrapping_sub(previous),
                factory,
                move |_, _| created_currency.take().unwrap_or_default(),
            ))
        } else if current < previous {
            PlayerYuanBaoChangeOutcome::Decreased(
                self.yuan_bao
                    .decrease_currency(previous.wrapping_sub(current), factory),
            )
        } else {
            PlayerYuanBaoChangeOutcome::Unchanged
        };
        PlayerYuanBaoChange {
            player_id: self.player_id(),
            previous,
            current: self.yuan_bao.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub(crate) const fn set_client_ip_snapshot(&mut self, client_ip: u32) {
        self.client_ip = client_ip;
    }

    pub(crate) fn depot_money(&self) -> u32 {
        self.bank.gold_coins_amount()
    }

    pub(crate) const fn pk_count(&self) -> u16 {
        self.base_properties.pk_count
    }

    pub(crate) fn reset_murder_counters(&mut self) -> PlayerMurderCountersResetReport {
        let report = PlayerMurderCountersResetReport {
            player_id: self.player_id(),
            previous_pk_count: self.base_properties.pk_count,
            previous_kill_count: self.base_properties.kill_count,
        };
        self.base_properties.pk_count = 0;
        self.base_properties.kill_count = 0;
        report
    }

    /// World kill confirmation tail: unsigned saturation, wrapping kill count
    /// и `OnUpdateMurdererSign` с единственным clock sample при старте timer-а.
    pub(crate) fn apply_confirmed_kill(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> PlayerConfirmedKillReport {
        self.base_properties.pk_count = u32::from(self.base_properties.pk_count)
            .saturating_add(pk_count_per_kill)
            .min(u32::from(u16::MAX)) as u16;
        self.base_properties.kill_count = self.base_properties.kill_count.wrapping_add(1);
        let murderer_timestamp_started =
            self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms == 0;
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
        } else if murderer_timestamp_started {
            self.murderer_time_stamp_ms = now_ms();
        }
        PlayerConfirmedKillReport {
            player_id: self.player_id(),
            pk_count: self.base_properties.pk_count,
            kill_count: self.base_properties.kill_count,
            murderer_timestamp_started,
        }
    }

    pub(crate) const fn set_money_snapshot(&mut self, money: u32) {
        self.money = money;
    }

    pub(crate) const fn silence_minutes(&self) -> i32 {
        self.silence_minutes
    }

    /// Exact chat cooldown: unsigned wrapping elapsed сравнивается до
    /// mutation. Caller сохраняет исходный порядок последующих проверок.
    pub(crate) fn begin_talk(
        &mut self,
        channel: PlayerTalkChannel,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        let timestamp = match channel {
            PlayerTalkChannel::Normal => &mut self.normal_talk_timestamp_ms,
            PlayerTalkChannel::Area => &mut self.area_talk_timestamp_ms,
            PlayerTalkChannel::Country => &mut self.country_talk_timestamp_ms,
            PlayerTalkChannel::World => &mut self.world_talk_timestamp_ms,
            PlayerTalkChannel::Private => &mut self.private_talk_timestamp_ms,
            PlayerTalkChannel::Union => &mut self.union_talk_timestamp_ms,
        };
        if now_ms.wrapping_sub(*timestamp) < interval_ms {
            return false;
        }
        *timestamp = now_ms;
        true
    }

    pub(crate) const fn equipment(&self) -> &CEquipmentContainer {
        &self.equipment
    }

    /// Точный обход GoodsAI при первом входе: позиционные equipment/packet,
    /// одиночный hand, позиционные auction и depot. Возврат `false` повторяет
    /// исходный `break` только внутри текущего контейнера; следующий владелец
    /// всё равно обрабатывается. Закрытый depot читается через собственное
    /// базовое хранилище без временной смены lock-флага — безопасная замена
    /// исходного `Unlock(saved password) → traversal → Lock`, не меняющая
    /// наблюдаемое итоговое состояние блокировки.
    pub(crate) fn visit_login_goods_mut(
        &mut self,
        mut visit: impl FnMut(PlayerLoginGoodsLocation, &mut CGoods) -> bool,
    ) {
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Equipment, goods)
            {
                break;
            }
        }
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Packet, goods)
            {
                break;
            }
        }
        let hand_id = self
            .hand
            .traversing_goods()
            .next()
            .map(|goods| goods.identity().ex_id);
        if let Some(goods_id) = hand_id
            && let Some(goods) = self.hand.find_mut(goods_id)
        {
            let _ = visit(PlayerLoginGoodsLocation::Hand, goods);
        }
        for position in 0..self.auction_listing.size() {
            if let Some(goods) = self.auction_listing.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Auction, goods)
            {
                break;
            }
        }
        for position in 0..self.depot.base().size() {
            if let Some(goods) = self.depot.base_mut().get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Depot, goods)
            {
                break;
            }
        }
    }

    pub(crate) const fn packet(&self) -> &CVolumeLimitGoodsContainer {
        &self.packet
    }

    pub(crate) const fn depot(&self) -> &CDepot {
        &self.depot
    }

    pub(crate) fn trade_source_goods(
        &self,
        extend_id: i32,
        position: u32,
        goods_id: CGuid,
    ) -> Option<&CGoods> {
        let goods = match extend_id {
            1 => self.packet.get_goods(position),
            2 => self.equipment.get_goods(position),
            4 => self.wallet.get_goods(position),
            5 => self.yuan_bao.get_goods(position),
            _ => None,
        }?;
        (goods.identity().ex_id == goods_id).then_some(goods)
    }

    pub(crate) fn enhancement_selected_goods_id(&self) -> Option<CGuid> {
        self.enhancement.base().goods_id_at(0)
    }

    /// Enhancement container хранит только shadow metadata; script 9409/9411
    /// каждый раз разрешает выбранный товар обратно в его live owner.
    pub(crate) fn enhancement_selected_goods_mut(&mut self) -> Option<&mut CGoods> {
        let goods_id = self.enhancement_selected_goods_id()?;
        self.get_goods_by_id_mut(goods_id)
    }

    /// Script-function owner пишет server-trusted path; client `0x8FC11/12`
    /// никогда не передаёт имя исполняемого файла.
    pub(crate) fn set_last_container_script(&mut self, script: impl AsRef<[u8]>) {
        self.last_container_script.clear();
        self.last_container_script
            .extend_from_slice(script.as_ref());
    }

    pub(crate) fn last_container_script(&self) -> &[u8] {
        &self.last_container_script
    }

    pub(crate) const fn variable_list(&self) -> &CVariableList {
        &self.variable_list
    }

    pub(crate) fn initialize_variable_list(&mut self, definitions: Option<&[u8]>) {
        if self.variable_list.variables().is_empty() {
            self.variable_list = CVariableList::from_definitions(definitions);
        }
    }

    pub(crate) fn set_string_variable(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_string(name, value)
    }

    pub(crate) fn set_integer_variable(
        &mut self,
        name: &[u8],
        element_index: usize,
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_integer(name, element_index, value)
    }

    pub(crate) fn clear_all_enhancement_selection(&mut self) -> usize {
        self.enhancement.clear()
    }

    pub(crate) fn record_enhancement_selection(
        &mut self,
        goods_id: CGuid,
        previous: PreviousContainer,
        placed_position: u32,
    ) -> Result<AmountShadowAdded, EnhancementSelectionBlock> {
        let goods = self
            .get_goods_by_id(goods_id)
            .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        let placed = PlacedShadowGoods {
            identity: goods_id,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        self.enhancement
            .record_placed_goods(previous, placed)
            .map_err(EnhancementSelectionBlock::Shadow)
    }

    /// Exact player→enhancement часть `CC2SContainerObjectMove`: shadow не
    /// владеет goods, поэтому успешный native remove→source add безопасно
    /// свёрнут в проверку live source и атомарную запись metadata.
    pub(crate) fn select_enhancement_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        factory: &CGoodsFactory,
    ) -> Result<EnhancementSelectionReport, EnhancementSelectionBlock> {
        let goods = match source_extend_id {
            1 => self.packet.get_goods(source_position),
            2 => self.equipment.get_goods(source_position),
            _ => return Err(EnhancementSelectionBlock::UnsupportedSourceContainer),
        }
        .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        if goods.identity().ex_id != goods_id {
            return Err(EnhancementSelectionBlock::GoodsIdentityMismatch);
        }
        if goods.amount() != amount {
            return Err(EnhancementSelectionBlock::GoodsAmountMismatch);
        }
        match goods.can_stack(factory) {
            Ok(true) => return Err(EnhancementSelectionBlock::StackableGoods),
            Ok(false) => {}
            Err(_) => return Err(EnhancementSelectionBlock::MissingBaseProperties),
        }

        let goods = goods.identity();
        let source = PreviousContainer {
            container_type: PLAYER_TYPE,
            container_id: self.player_id(),
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        let shadow = self.record_enhancement_selection(goods_id, source, source_position)?;
        let previous_last_operated =
            self.record_last_operated_goods(source_extend_id, source_position);
        Ok(EnhancementSelectionReport {
            goods,
            source,
            shadow,
            previous_last_operated,
        })
    }

    pub(crate) fn enhancement_original_container(
        &self,
        shadow_position: u32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        (self.enhancement.base().goods_id_at(shadow_position) == Some(goods_id))
            .then(|| {
                self.enhancement
                    .base()
                    .original_container_information(goods_id)
            })
            .flatten()
    }

    pub(crate) fn enhancement_remove_shadow(
        &mut self,
        goods_id: CGuid,
    ) -> Option<super::container::cgoodsshadowcontainer::ShadowRemovedReport> {
        self.enhancement.base_mut().remove_shadow(goods_id)
    }

    /// Same-original-slot ветвь native shadow Remove: underlying goods после
    /// remove→add остаётся у прежнего owner-а, а здесь удаляется только shadow
    /// metadata и формируется обязательный `OT_DELETE_OBJECT` report.
    pub(crate) fn clear_enhancement_selection(
        &mut self,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EnhancementDeselectionReport, EnhancementDeselectionBlock> {
        let actual_id = self
            .enhancement
            .base()
            .goods_id_at(shadow_position)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        if actual_id != goods_id {
            return Err(EnhancementDeselectionBlock::GoodsIdentityMismatch);
        }
        let source = self
            .enhancement
            .base()
            .original_container_information(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        let goods = match source.container_extend_id {
            1 => self.packet.get_goods(source.goods_position),
            2 => self.equipment.get_goods(source.goods_position),
            _ => None,
        }
        .filter(|goods| goods.identity().ex_id == goods_id)
        .ok_or(EnhancementDeselectionBlock::MissingSourceGoods)?;
        if goods.amount() != amount {
            return Err(EnhancementDeselectionBlock::GoodsAmountMismatch);
        }
        let goods = goods.identity();
        let removed = self
            .enhancement
            .base_mut()
            .remove_shadow(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        Ok(EnhancementDeselectionReport {
            goods,
            source,
            removed,
        })
    }

    pub(crate) const fn packet_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.packet
    }

    pub(crate) const fn equipment_mut(&mut self) -> &mut CEquipmentContainer {
        &mut self.equipment
    }

    pub(crate) const fn battle_fairy_container(&self) -> &CBattleFairyContainer {
        &self.battle_fairy_container
    }

    pub(crate) const fn fairy_container(&self) -> &CFairyContainer {
        &self.fairy_container
    }

    pub(crate) const fn fairy_container_mut(&mut self) -> &mut CFairyContainer {
        &mut self.fairy_container
    }

    pub(crate) const fn battle_fairy_container_mut(&mut self) -> &mut CBattleFairyContainer {
        &mut self.battle_fairy_container
    }

    /// Account принадлежит player snapshot и используется exact audit-log
    /// combine; отсутствие ещё не загруженного account остаётся пустой строкой.
    pub(crate) fn set_account(&mut self, account: impl AsRef<[u8]>) {
        self.account.clear();
        self.account.extend_from_slice(account.as_ref());
    }

    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    pub(crate) fn billing_session_id(&self) -> &[u8] {
        &self.session_id
    }

    /// Exact `GetWarSoulGoods`: боевой дух — только headgear в позиции 10,
    /// чьё первое значение `GAP_BF_BATTLE_FAIRY` равно единице.
    pub(crate) fn war_soul_goods(&self, factory: &CGoodsFactory) -> Option<&CGoods> {
        self.equipment
            .get_goods(10)
            .filter(|goods| goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1)
    }

    /// Exact `GetGoodsById` lookup order для goods-message `0x8FC2E`.
    /// Locked hand/packet/auction goods скрываются container `find`, equipment
    /// использует собственный positional storage.
    pub(crate) fn get_goods_by_id(&self, goods_id: CGuid) -> Option<&CGoods> {
        self.hand
            .find(goods_id)
            .or_else(|| self.packet.base().find(goods_id))
            .or_else(|| self.equipment.find(goods_id))
            .or_else(|| self.auction_listing.base().find(goods_id))
            .or_else(|| self.auction_goods.base().find(goods_id))
    }

    pub(crate) fn get_goods_by_id_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        if self.hand.find(goods_id).is_some() {
            return self.hand.find_mut(goods_id);
        }
        if self.packet.base().find(goods_id).is_some() {
            return self.packet.base_mut().find_mut(goods_id);
        }
        if self.equipment.find(goods_id).is_some() {
            return self.equipment.find_mut(goods_id);
        }
        if self.auction_listing.base().find(goods_id).is_some() {
            return self.auction_listing.base_mut().find_mut(goods_id);
        }
        self.auction_goods.base_mut().find_mut(goods_id)
    }

    pub(crate) const fn hand_mut(&mut self) -> &mut CAmountLimitGoodsContainer {
        &mut self.hand
    }

    pub(crate) const fn hand(&self) -> &CAmountLimitGoodsContainer {
        &self.hand
    }

    pub(crate) const fn auction_goods_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_goods
    }

    pub(crate) const fn auction_goods(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_goods
    }

    pub(crate) const fn auction_listing(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_listing
    }

    pub(crate) const fn auction_listing_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_listing
    }

    pub(crate) fn auction_listing_extension_bonus(&self, factory: &CGoodsFactory) -> i32 {
        let Some(goods) = self.auction_listing.get_goods(1) else {
            return 0;
        };
        if goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) != 3 {
            return 0;
        }
        goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2)
    }

    pub(crate) fn take_auction_listing_goods(&mut self) -> Option<CGoods> {
        let goods_id = self.auction_listing.get_goods(0)?.identity().ex_id;
        let outcome = self.auction_listing.remove_goods(goods_id)?;
        let taken = match outcome {
            VolumeGoodsRemoveOutcome::Removed(taken)
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
        };
        Some(match taken {
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Removed(removed) => removed.goods,
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Split(split) => split.goods,
        })
    }

    /// Exact state-owner возврата `0x80404`: позиция выбирается до Add,
    /// stack merge использует обычный player-progress gate, а bind value-id 2
    /// записывается уже в итоговый stored goods.
    pub(crate) fn return_auction_goods(
        &mut self,
        goods: CGoods,
        bind_type: i32,
        factory: &CGoodsFactory,
    ) -> Option<PlayerAuctionGoodsReturn> {
        let position = self
            .auction_goods
            .find_position_for_goods(&goods, factory)?;
        let source = goods.identity();
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        let mut incoming = Some(goods);
        let outcome = self.auction_goods.add_goods_at(
            position,
            &mut incoming,
            factory,
            owner_progress_allows,
        );
        let successful = matches!(
            &outcome,
            VolumeGoodsAddOutcome::Added(_)
                | VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { .. })
        );
        let (resulting_goods, resulting_amount, bind_stored) = if successful {
            let stored = self
                .auction_goods
                .get_goods_mut(position)
                .expect("успешный auction Add обязан оставить stored goods");
            let bind_stored = stored.set_addon_property_value_core(GAP_GOODS_BIND, 2, bind_type);
            (Some(stored.identity()), Some(stored.amount()), bind_stored)
        } else {
            (None, None, false)
        };
        Some(PlayerAuctionGoodsReturn {
            player_id: self.player_id(),
            position,
            source,
            outcome,
            resulting_goods,
            resulting_amount,
            bind_stored,
        })
    }

    pub(crate) fn auction_money(&self) -> u32 {
        self.auction_wallet.currency_amount()
    }

    pub(crate) fn auction_money_goods(&self) -> Option<&CGoods> {
        self.auction_wallet.get_goods(0)
    }

    /// State-часть exact `SetAuctionMoney`; caller создаёт недостающий MONEY
    /// через общий factory и публикует extend-id 15.
    pub(crate) fn increase_auction_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerAuctionMoneyChange {
        let previous = self.auction_wallet.currency_amount();
        let mut created_currency = Some(created_currency);
        let outcome = self
            .auction_wallet
            .increase_currency(requested, factory, move |_, _| {
                created_currency.take().unwrap_or_default()
            });
        PlayerAuctionMoneyChange {
            player_id: self.player_id(),
            previous,
            current: self.auction_wallet.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn set_auction_open(&mut self, open: bool) {
        self.auction_open = open;
    }

    /// State/container часть exact `TellClientScale`; закрытый аукцион не
    /// создаёт client-effect, открытый сохраняет container traversal order.
    pub(crate) fn auction_scale_goods_ids(&self) -> Option<Vec<CGuid>> {
        self.auction_open.then(|| {
            self.auction_goods
                .base()
                .traversing_goods()
                .map(|goods| goods.identity().ex_id)
                .collect()
        })
    }

    pub(crate) fn auction_goods_identity_at(&self, position: u32) -> Option<ShapeIdentity> {
        self.auction_goods.get_goods(position).map(CGoods::identity)
    }

    /// Exact `BuyItemFromAauction` clock gate: strict wrapping threshold и
    /// отдельный второй sample записываются до GUID decode/query.
    pub(crate) fn begin_auction_buy(&mut self, mut tick_ms: impl FnMut() -> u32) -> AuctionBuyGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionBuyGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        AuctionBuyGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// Exact `MakeCurAucNode` 5-second gate с отдельным вторым tick sample.
    pub(crate) fn begin_auction_listing(
        &mut self,
        mut tick_ms: impl FnMut() -> u32,
    ) -> AuctionListingGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionListingGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        AuctionListingGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// Exact `IsAollowAuction` 1-second gate: timestamp обновляется до limit
    /// queries, а failed limit также поглощает текущую попытку.
    pub(crate) fn begin_auction_limit_check(
        &mut self,
        tick_ms: u32,
        owner_goods_count: usize,
        global_goods_count: usize,
        player_maximum: f32,
        global_maximum: f32,
        extension_bonus: i32,
    ) -> bool {
        if tick_ms.wrapping_sub(self.last_auction_limit_tick_ms) <= 1_000 {
            return false;
        }
        self.last_auction_limit_tick_ms = tick_ms;
        (owner_goods_count as f32) < extension_bonus as f32 + player_maximum
            && (global_goods_count as f32) < global_maximum
    }

    pub(crate) fn current_auction_node(&self) -> Option<&CGoodsNode> {
        self.current_auction_node.as_ref()
    }

    pub(crate) fn set_current_auction_node(&mut self, node: CGoodsNode) -> bool {
        if self.current_auction_node.is_some() {
            return false;
        }
        self.current_auction_node = Some(node);
        true
    }

    pub(crate) fn take_current_auction_node(&mut self) -> Option<CGoodsNode> {
        self.current_auction_node.take()
    }

    pub(crate) const fn auction_listing_fee(&self) -> u32 {
        self.auction_listing_fee
    }

    pub(crate) const fn set_auction_listing_fee(&mut self, fee: u32) {
        self.auction_listing_fee = fee;
    }

    pub(crate) fn current_auction_buy_node(&self) -> Option<&CGoodsNode> {
        self.current_auction_buy_node.as_ref()
    }

    pub(crate) fn set_current_auction_buy_node(&mut self, node: CGoodsNode) -> bool {
        if self.current_auction_buy_node.is_some() {
            return false;
        }
        self.current_auction_buy_node = Some(node);
        true
    }

    pub(crate) fn take_current_auction_buy_node(&mut self) -> Option<CGoodsNode> {
        self.current_auction_buy_node.take()
    }

    pub(crate) fn client_ip_text(&self) -> Vec<u8> {
        let ip = self.client_ip;
        format!(
            "{}.{}.{}.{}",
            ip & 0xff,
            (ip >> 8) & 0xff,
            (ip >> 16) & 0xff,
            ip >> 24
        )
        .into_bytes()
    }

    pub(crate) fn begin_auction_search(
        &mut self,
        name: &[u8],
        lower_level: i32,
        upper_level: i32,
        use_self: i32,
        money_type: i32,
        weapon_type: i32,
    ) {
        self.auction_search_name.clear();
        self.auction_search_name.extend_from_slice(name);
        self.auction_search_lower_level = lower_level;
        self.auction_search_upper_level = upper_level;
        self.auction_search_use_self = use_self;
        self.auction_search_money_type = money_type;
        self.auction_search_weapon_type = weapon_type;
        self.auction_current_page = 0;
    }

    /// Exact `ReFlushSelfGoods`: strict wrapping `last + 5000 < first sample`,
    /// затем отдельный второй `timeGetTime` sample записывается до World send.
    pub(crate) fn refresh_auction_self_goods(
        &mut self,
        factory: &CGoodsFactory,
        mut tick_ms: impl FnMut() -> u32,
    ) -> AuctionSelfGoodsRefresh {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_auction_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionSelfGoodsRefresh::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_auction_option_tick_ms = recorded_tick_ms;
        let wallet_amount = self.auction_wallet.currency_amount();
        let wallet_maximum = self.auction_wallet.max_stack_number(factory);
        AuctionSelfGoodsRefresh::Requested {
            sampled_tick_ms,
            recorded_tick_ms,
            goods_space: self.auction_goods.space(),
            wallet_space: wallet_maximum.wrapping_sub(wallet_amount),
        }
    }

    /// Exact derived `bHasPet`: отдельный pet owner materializes list later;
    /// этому caller-у нужен только подтверждённый факт её непустоты.
    pub(crate) const fn set_active_pet_count(&mut self, count: u32) {
        self.active_pet_count = count;
    }

    pub(crate) const fn has_pet(&self) -> bool {
        self.active_pet_count != 0
    }

    /// Snapshot/skill caller передаёт только current ID, достаточный для
    /// `SummonBF` запрета `SKILL_MONSTER_TAMING`; concrete skill execution не
    /// становится частью player owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.move_shape.set_current_skill_id(skill_id);
    }

    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.move_shape.current_skill_id()
    }

    pub(crate) const fn war_soul_state(&self) -> u32 {
        self.war_soul_state
    }

    pub(crate) const fn war_soul_point(&self) -> WarSoulPoint {
        self.war_soul_point
    }

    pub(crate) const fn battle_fairy_summoned(&self) -> bool {
        self.battle_fairy_summoned
    }

    /// Exact `SetWarSoulStaus`: around status публикуется по прежнему state,
    /// затем любое значение кроме единицы нормализуется к нулю.
    pub(crate) const fn set_war_soul_status(&mut self, value: u32) -> bool {
        let broadcast_previous = self.war_soul_state == 1;
        if value == 1 {
            self.battle_fairy_summoned = true;
            self.war_soul_state = 1;
        } else {
            self.battle_fairy_summoned = false;
            self.war_soul_state = 0;
        }
        broadcast_previous
    }

    /// Исполняет player-часть `CBattleFairyContainer::SummonBF`. Spatial map
    /// принадлежит `CServerRegion`, поэтому действие возвращается явным
    /// tail-ом для `CGame`; ordered notify/broadcast/property effects там
    /// сериализуются concrete wire после spatial mutation.
    pub(crate) fn summon_battle_fairy(
        &mut self,
        battle_fairy_enabled: bool,
        mode: i32,
        factory: &CGoodsFactory,
    ) -> BattleFairySummonReport {
        let player_id = self.player_id();
        let mut report = BattleFairySummonReport {
            player_id,
            outcome: BattleFairySummonOutcome::IgnoredMode,
            region_id: self.server_region_id,
            spatial_action: None,
            spatial_applied: false,
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySummonOutcome::FeatureDisabled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0023", 0xffff_ffff);
            return report;
        }
        if mode == 1 && self.war_soul_state == 1 {
            report.outcome = BattleFairySummonOutcome::AlreadySummoned;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0024", 0xffff_ffff);
            return report;
        }
        if mode == -1 && self.base_properties.battle_fairy_recall {
            report.outcome = BattleFairySummonOutcome::AlreadyRecalled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0025", 0xffff_ffff);
            return report;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            report.outcome = BattleFairySummonOutcome::MissingHeadgear;
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySummonOutcome::InvalidHeadgear;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0009", 0xffff_ffff);
            return report;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) < 1 {
            report.outcome = BattleFairySummonOutcome::NoHitPoints;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0026", 0xffff_0000);
            return report;
        }
        if self.has_pet() {
            report.outcome = BattleFairySummonOutcome::ActivePet;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0027", 0xffff_ffff);
            return report;
        }
        if self.move_shape.current_skill_id() == Some(MONSTER_TAMING_SKILL_ID) {
            report.outcome = BattleFairySummonOutcome::MonsterTamingActive;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0028", 0xffff_ffff);
            return report;
        }
        let player_position = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => WarSoulPoint { x, y },
            (Err(error), _) | (_, Err(error)) => {
                report.outcome = BattleFairySummonOutcome::CoordinateBlocked(error);
                return report;
            }
        };

        match mode {
            1 => {
                self.battle_fairy_summoned = true;
                self.war_soul_state = 1;
                self.base_properties.battle_fairy_recall = false;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (player_position.x as f32).to_bits();
                self.war_soul_visual_y_bits = (player_position.y as f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Summoned;
                report.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
                    previous: self.war_soul_point,
                    target: player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
                    player_id,
                    values: vec![player_id, 700, player_position.x, player_position.y],
                });
                // `SetWarSoulStaus(1)` наблюдает уже записанный state `1` и
                // поэтому публикует exact `0xbf930 {400, player_id}`.
                let _broadcast_previous = self.set_war_soul_status(1);
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, player_id],
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_SUMMON_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, 1],
                });
            }
            -1 => {
                self.battle_fairy_summoned = false;
                self.war_soul_state = 0;
                self.base_properties.battle_fairy_recall = true;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (-1.0f32).to_bits();
                self.war_soul_visual_y_bits = (-1.0f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Recalled;
                report.spatial_action = Some(BattleFairyWarSoulAction::Delete {
                    previous: self.war_soul_point,
                    player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, -1],
                });
            }
            _ => {}
        }
        report
            .effects
            .push(BattleFairySummonEffect::PropertiesChanged { player_id });
        report
    }

    /// Полный player-tail успешного `CEquipmentContainer::Remove`: container
    /// mutation предшествует callback-ам, поэтому removed slot уже отсутствует
    /// во время injected результата virtual `PropertiesChanged`.
    pub(crate) fn remove_equipment_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentRemoveRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> PlayerEquipmentRemoveReport {
        let player_id = self.player_id();
        let outcome = self.equipment.remove(
            ex_id,
            factory,
            EquipmentRemoveRuntimeFacts {
                owner_player_present: true,
                pack_add_enabled: runtime.pack_add_enabled,
                player_goods_package_extension: runtime.player_goods_package_extension,
                active_war_soul_blocks_headgear: runtime.active_war_soul_blocks_headgear,
            },
        );
        let mut effects = Vec::new();
        if let EquipmentRemoveOutcome::Removed(removed) = &outcome
            && let Some(player_effects) = removed.event.player_effects
        {
            if player_effects.clear_war_soul_status && self.set_war_soul_status(0) {
                effects.push(PlayerEquipmentRemoveEffect::WarSoulStatusAround {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: [400, player_id],
                });
            }
            if player_effects.delete_war_soul_skill {
                for (skill_id, _) in war_soul_skill_entries_from_goods(&removed.goods, factory) {
                    let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
                    effects.push(PlayerEquipmentRemoveEffect::WarSoulSkillDetached { skill_id });
                    if let Some(skill) = self.move_shape.skill(skill_id) {
                        effects.push(PlayerEquipmentRemoveEffect::SkillRemoved(
                            BattleFairySkillRemoved {
                                message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                                player_id,
                                skill_id,
                                skill_name: skill.name().to_vec(),
                            },
                        ));
                    }
                }
            }
            if player_effects.recompute_without_removed_slot {
                let properties = recompute_properties(self);
                self.apply_recomputed_combat_properties(properties);
                effects.push(
                    PlayerEquipmentRemoveEffect::PropertiesChangedWithoutRemovedSlot {
                        column: removed.event.column,
                        combat_properties: self.combat_properties,
                    },
                );
            }
            if player_effects.clamp_hp_and_mp {
                let previous_health = self.health();
                let previous_mana = self.mana();
                self.set_health(previous_health);
                self.set_mana(previous_mana);
                effects.push(PlayerEquipmentRemoveEffect::VitalsClamped {
                    previous_health,
                    current_health: self.health(),
                    previous_mana,
                    current_mana: self.mana(),
                });
            }
            effects.push(PlayerEquipmentRemoveEffect::AroundUpdate(
                player_effects.around_update,
            ));
        }
        PlayerEquipmentRemoveReport {
            player_id,
            outcome,
            effects,
            deliveries: Vec::new(),
        }
    }

    /// Полный player-tail positional `CEquipmentContainer::Add`. Timed и
    /// goods-AI partial effects остаются наблюдаемы даже при late block; skill,
    /// properties, around и package-log выполняются только после commit.
    pub(crate) fn add_equipment_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentAddRuntimeFacts,
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> PlayerEquipmentAddReport {
        let player_id = self.player_id();
        let previous_expanded_package_num = self.equipment.expanded_package_num();
        let can_mount_result = incoming
            .as_ref()
            .map_or(0, |goods| self.can_mount_equip(goods, factory));
        let outcome = self.equipment.add_at(
            position,
            incoming,
            factory,
            EquipmentAddRuntimeFacts {
                owner_player: Some(EquipmentOwnerPlayerFacts { can_mount_result }),
                pack_add_enabled: runtime.pack_add_enabled,
                now: runtime.now,
            },
            register_with_goods_ai,
        );
        let mut effects = Vec::new();
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(player_effects) = added.player_effects
        {
            if added.package_extension_applied {
                self.equipment
                    .set_expanded_package_num_snapshot(previous_expanded_package_num);
            }
            if player_effects.add_war_soul_skill
                && let Some(goods) = self.equipment.get_goods(added.column.position())
            {
                for (skill_id, level) in war_soul_skill_entries_from_goods(goods, factory) {
                    let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
                    effects
                        .push(PlayerEquipmentAddEffect::WarSoulSkillAttached { skill_id, level });
                    if let Some(skill) = self.move_shape.skill(skill_id) {
                        effects.push(PlayerEquipmentAddEffect::SkillAdded(
                            battle_fairy_skill_snapshot(player_id, skill),
                        ));
                    }
                }
            }
            if player_effects.recompute_properties {
                let properties = recompute_properties(self);
                self.apply_recomputed_combat_properties(properties);
                effects.push(PlayerEquipmentAddEffect::PropertiesChanged {
                    combat_properties: self.combat_properties,
                });
            }
            effects.push(PlayerEquipmentAddEffect::AroundUpdate(
                player_effects.around_update,
            ));
            if added.package_extension_applied {
                self.equipment.set_expanded_package_num_snapshot(
                    previous_expanded_package_num.wrapping_add(added.package_extension_delta),
                );
                effects.push(PlayerEquipmentAddEffect::PackageExtensionLogged {
                    category: "PackExpand",
                    string_id: "KR002",
                    expanded_package_num: self.equipment.expanded_package_num(),
                });
            }
        }
        PlayerEquipmentAddReport {
            player_id,
            outcome,
            effects,
            deliveries: Vec::new(),
        }
    }

    /// Завершает CGame-owned area tail. Recall всегда копирует player point
    /// после попытки `DelWarSoul`, даже если old area отсутствовала; это
    /// literal последняя запись `CPlayer::DelWarSoul`.
    pub(crate) const fn apply_war_soul_action(
        &mut self,
        action: BattleFairyWarSoulAction,
        spatial_applied: bool,
    ) {
        match action {
            BattleFairyWarSoulAction::SetPosition { target, .. } if spatial_applied => {
                self.war_soul_point = target;
            }
            BattleFairyWarSoulAction::Delete {
                player_position, ..
            } => {
                self.war_soul_point = player_position;
            }
            BattleFairyWarSoulAction::SetPosition { .. } => {}
        }
    }

    /// Один живой `ComputeWarSoulXY` tick. `Some(false)` означает найденный
    /// current war-soul skill с `IsRestored()==0`; `None` точно соответствует
    /// отсутствующему skill и не блокирует follow.
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        current_war_soul_skill_restored: Option<bool>,
    ) -> BattleFairyFollowReport {
        let player_id = self.player_id();
        let mut report = BattleFairyFollowReport {
            player_id,
            outcome: BattleFairyFollowOutcome::NotSummoned,
            region_id: self.server_region_id,
            visual_x_bits: self.war_soul_visual_x_bits,
            visual_y_bits: self.war_soul_visual_y_bits,
            spatial_action: None,
            spatial_applied: false,
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if current_war_soul_skill_restored == Some(false) {
            report.outcome = BattleFairyFollowOutcome::ActiveSkill;
            return report;
        }
        if self.war_soul_state != 1 {
            return report;
        }
        let (tile_x, tile_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => (x, y),
            (Err(error), _) | (_, Err(error)) => {
                report.outcome = BattleFairyFollowOutcome::CoordinateBlocked(error);
                return report;
            }
        };
        let current_x = tile_x as f32;
        let current_y = tile_y as f32;
        let mut visual_x = f32::from_bits(self.war_soul_visual_x_bits);
        let mut visual_y = f32::from_bits(self.war_soul_visual_y_bits);
        let delta_x = current_x - visual_x;
        let delta_y = current_y - visual_y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().abs();
        if !distance.is_finite() {
            report.outcome = BattleFairyFollowOutcome::NonFiniteVisualState;
            return report;
        }
        if distance < 0.5 {
            report.outcome = BattleFairyFollowOutcome::InsideDeadZone;
            return report;
        }

        let (target, outcome) = if distance <= 5.0 {
            let coefficient = if distance > 3.75 {
                0.265f32
            } else if distance > 0.75 {
                0.065f32
            } else {
                0.045f32
            };
            let step = distance * (coefficient + coefficient);
            if (current_x - visual_x).abs() > 0.1 {
                visual_x = if current_x <= visual_x {
                    visual_x - step
                } else {
                    visual_x + step
                };
            }
            if (current_y - visual_y).abs() > 0.1 {
                visual_y = if current_y <= visual_y {
                    visual_y - step
                } else {
                    visual_y + step
                };
            }
            (
                WarSoulPoint {
                    // EXE временно ставит x87 RC=truncate перед обоими fistp.
                    x: visual_x.trunc() as i32,
                    y: visual_y.trunc() as i32,
                },
                BattleFairyFollowOutcome::Moved,
            )
        } else {
            visual_x = current_x;
            visual_y = current_y;
            (
                WarSoulPoint {
                    x: tile_x,
                    y: tile_y,
                },
                BattleFairyFollowOutcome::Snapped,
            )
        };
        self.war_soul_visual_x_bits = visual_x.to_bits();
        self.war_soul_visual_y_bits = visual_y.to_bits();
        report.visual_x_bits = self.war_soul_visual_x_bits;
        report.visual_y_bits = self.war_soul_visual_y_bits;
        report.outcome = outcome;
        report.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
            previous: self.war_soul_point,
            target,
        });
        report.effects.push(BattleFairyFollowEffect::AroundMove {
            message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
            player_id,
            object_type: 700,
            x: visual_x.to_bits(),
            y: visual_y.to_bits(),
        });
        report
    }

    /// Periodic prefix `CPlayer::AI`: нулевой HP equipped battle fairy каждый
    /// tick повторно нормализует четыре state-поля и вызывает PropertiesChanged.
    /// Исходник не удаляет stale area-map entry и не посылает status broadcast.
    pub(crate) fn refresh_battle_fairy_death(
        &mut self,
        factory: &CGoodsFactory,
    ) -> BattleFairyDeathReport {
        let player_id = self.player_id();
        let mut report = BattleFairyDeathReport {
            player_id,
            outcome: BattleFairyDeathOutcome::MissingHeadgear,
            effects: Vec::new(),
            property_delivery: None,
        };
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyDeathOutcome::NotBattleFairy;
            return report;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
            report.outcome = BattleFairyDeathOutcome::Alive;
            return report;
        }
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        self.set_battle_fairy_recall(true);
        self.set_battle_fairy_died(true);
        report.outcome = BattleFairyDeathOutcome::Died;
        report
            .effects
            .push(BattleFairyDeathEffect::PropertiesChanged { player_id });
        report
    }

    fn apply_battle_fairy_property(
        &mut self,
        cell: BattleFairyCell,
        addons: BattleFairyGearAddons,
        delta: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyDefaultGoodsUpdate> {
        if delta == 0 || !is_battle_fairy_property_cell(cell) {
            return None;
        }
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let player_id = self.player_id();
        let battle_fairy = self.equipment.get_goods_mut(10)?;
        if battle_fairy.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return None;
        }

        for (source, target) in [
            (addons.attack, GAP_BF_ATTACK),
            (addons.sprite, GAP_BF_SPRITE),
            (addons.strength, GAP_BF_STRENGH),
            (addons.brave, GAP_BF_BRAVE),
            (addons.agility, GAP_BF_AGILITY),
            (addons.spiritualism, GAP_BF_SPRITUALISM),
            (addons.blast, GAP_BF_BLAST),
            (addons.cut_hurt, GAP_BF_CUT_HURT_SCALE),
        ] {
            add_battle_fairy_addon(battle_fairy, factory, target, source.wrapping_mul(delta));
        }
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_HP,
            addons
                .strength
                .wrapping_add(addons.life)
                .wrapping_mul(delta),
        );
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_MP,
            addons
                .spiritualism
                .wrapping_add(addons.mana)
                .wrapping_mul(delta),
        );
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_HP, GAP_BF_MAX_HP);
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_MP, GAP_BF_MAX_MP);

        let strength = f64::from(addons.strength) * f64::from(delta) * 0.00001;
        let brave = f64::from(addons.brave) * f64::from(delta) * 0.00001;
        let agility = f64::from(addons.agility) * f64::from(delta) * 0.00001;
        let spiritualism = f64::from(addons.spiritualism) * f64::from(delta) * 0.00001;
        let combat = &mut self.combat_properties;
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.maximum_attack = add_battle_fairy_u32(
            combat.maximum_attack,
            brave * f64::from(coefficients.str_to_max_attack[occupation]),
        );
        combat.burden = add_battle_fairy_u16(
            combat.burden,
            brave * f64::from(coefficients.str_to_burden[occupation]),
        );
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);
        combat.minimum_attack = add_battle_fairy_u32(
            combat.minimum_attack,
            agility * f64::from(coefficients.dex_to_min_attack[occupation]),
        );
        combat.reank = add_battle_fairy_u16(
            combat.reank,
            agility * f64::from(coefficients.dex_to_stiff[occupation]),
        );
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.element_modify = add_battle_fairy_i32(
            combat.element_modify,
            spiritualism * f64::from(coefficients.int_to_element[occupation]),
        );
        combat.maximum_mp = add_battle_fairy_u32(
            combat.maximum_mp,
            spiritualism * f64::from(coefficients.int_to_max_mp[occupation]),
        );
        combat.element_resistance = add_battle_fairy_u32(
            combat.element_resistance,
            spiritualism * f64::from(coefficients.int_to_resistant[occupation]),
        );

        // Подтверждённый RU quirk: четыре основных значения применяются
        // повторно после производных коэффициентов.
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);

        Some(BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id,
            goods: battle_fairy.identity(),
            old_client_payload: encode_old_client(battle_fairy),
        })
    }

    /// Exact positional `CBattleFairyContainer::Add`: для gear-ячеек
    /// `BFPropertyAdd(+1)` является ранним partial effect и сохраняется даже
    /// если base storage затем отвергнет товар.
    pub(crate) fn add_battle_fairy_goods(
        &mut self,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let early_property = incoming.as_ref().and_then(|goods| {
            self.battle_fairy_container
                .property_effect_before_add(cell, goods, factory)
                .map(|effect| (effect, BattleFairyGearAddons::read(goods, factory)))
        });
        let mut property_applied = false;
        let mut effects = Vec::new();
        if let Some((BattleFairyPropertyAddEffect { cell, delta }, addons)) = early_property
            && let Some(update) = self.apply_battle_fairy_property(
                cell,
                addons,
                delta,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            property_applied = true;
            effects.push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            effects.push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                update,
            ));
        }
        let outcome =
            self.battle_fairy_container
                .add_at(cell, incoming, factory, owner_progress_allows);
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: Some(cell),
            delta: 1,
            property_applied,
            outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
            effects,
            deliveries: Vec::new(),
        }
    }

    /// Exact `Remove`: base container отделяет goods до `BFPropertyAdd(-1)`;
    /// успешный property path сериализует battle fairy дважды — один раз в
    /// `BFPropertyAdd`, затем ещё раз в override `Remove`.
    pub(crate) fn remove_battle_fairy_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let position = self
            .battle_fairy_container
            .base()
            .query_goods_position(ex_id);
        let cell = position.and_then(BattleFairyCell::from_position);
        let addons = position
            .and_then(|position| self.battle_fairy_container.base().get_goods(position))
            .map(|goods| BattleFairyGearAddons::read(goods, factory));
        let Some(outcome) = self.battle_fairy_container.base_mut().remove_goods(ex_id) else {
            return BattleFairyEquipmentMutationReport {
                player_id,
                cell,
                delta: -1,
                property_applied: false,
                outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
                effects: Vec::new(),
                deliveries: Vec::new(),
            };
        };
        let mut report = BattleFairyEquipmentMutationReport {
            player_id,
            cell,
            delta: -1,
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::Removed(outcome),
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if let (Some(cell), Some(addons)) = (cell, addons)
            && let Some(first_update) = self.apply_battle_fairy_property(
                cell,
                addons,
                -1,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            report.property_applied = true;
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                    first_update,
                ));
            if let Some(battle_fairy) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: battle_fairy.identity(),
                            old_client_payload: encode_old_client(battle_fairy),
                        },
                    ));
            }
        }
        report
    }

    /// Полный player-side opcode `0x8FC2A`. `allocations` содержат пары
    /// property/client-points прямо из packet-а: legacy outer caller суммирует
    /// unscaled points, но передаёт каждому `AllocatePotential` wrapping
    /// `points * 10000`. `std::map::insert` сохраняет первую запись ключа.
    pub(crate) fn allocate_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        allocations: &[(i32, i32)],
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialAllocationReport {
        let player_id = self.player_id();
        let aggregate_client_points = allocations
            .iter()
            .fold(0i32, |total, (_, points)| total.wrapping_add(*points));
        let mut report = BattleFairyPotentialAllocationReport {
            player_id,
            outcome: BattleFairyPotentialAllocationOutcome::MissingHeadgear,
            aggregate_client_points,
            processed_properties: Vec::new(),
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialAllocationOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }
        if goods
            .addon_property_value(factory, GAP_BF_POTENTIAL, 1)
            .wrapping_sub(aggregate_client_points)
            < 0
        {
            report.outcome = BattleFairyPotentialAllocationOutcome::AggregateInsufficient;
            return report;
        }

        let mut ordered = BTreeMap::new();
        for &(property, points) in allocations {
            ordered.entry(property).or_insert(points);
        }
        for (property, points) in ordered {
            if !battle_fairy_enabled {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::Notification {
                        player_id,
                        string_id: "ZHGS0008",
                        color: 0xffff_0000,
                    });
                continue;
            }
            let amount = points.wrapping_mul(10_000);
            self.allocate_one_battle_fairy_potential(property, amount, factory, coefficients);
            report.processed_properties.push(property);
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::PropertiesChanged { player_id });
            if let Some(goods) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: goods.identity(),
                            old_client_payload: encode_old_client(goods),
                        },
                    ));
            }
        }

        // Outer goods-message сериализует headgear ещё раз независимо от
        // feature-disabled/unknown-property результата внутренних вызовов.
        if let Some(goods) = self.war_soul_goods(factory) {
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
        }
        report.outcome = BattleFairyPotentialAllocationOutcome::Processed;
        report
    }

    fn allocate_one_battle_fairy_potential(
        &mut self,
        property: i32,
        amount: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
    ) {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let mut player_delta = None;
        {
            let Some(goods) = self.equipment.get_goods_mut(10) else {
                return;
            };
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            if potential.wrapping_sub(amount) < 0 {
                return;
            }
            let (tracked_property, applied_amount) = match property {
                GAP_BF_ATTACK => (
                    GAP_BF_ATTACK_POTENTIAL,
                    (f64::from(amount) * 1.5).round() as i32,
                ),
                GAP_BF_SPRITE => (
                    GAP_BF_SPRITE_POTENTIAL,
                    (f64::from(amount) * 1.5).round() as i32,
                ),
                GAP_BF_BLAST => (GAP_BF_BLAST_POTENTIAL, amount),
                GAP_BF_BRAVE => (GAP_BF_BRAVE_POTENTIAL, amount),
                GAP_BF_AGILITY => (GAP_BF_AGILITY_POTENTIAL, amount),
                GAP_BF_SPRITUALISM => (GAP_BF_SPRITUALISM_POTENTIAL, amount),
                GAP_BF_STRENGH => (GAP_BF_STRENGH_POTENTIAL, amount),
                _ => return,
            };
            add_battle_fairy_addon(goods, factory, property, applied_amount);
            add_battle_fairy_addon(goods, factory, tracked_property, applied_amount);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_sub(amount),
            );
            if property == GAP_BF_SPRITUALISM {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_MP, amount);
            } else if property == GAP_BF_STRENGH {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_HP, amount);
            }
            if matches!(
                property,
                GAP_BF_BRAVE | GAP_BF_AGILITY | GAP_BF_SPRITUALISM | GAP_BF_STRENGH
            ) {
                player_delta = Some((property, f64::from(amount) * 0.00001));
            }
        }

        let Some((property, delta)) = player_delta else {
            return;
        };
        let combat = &mut self.combat_properties;
        match property {
            GAP_BF_BRAVE => {
                combat.strength = add_battle_fairy_u32(combat.strength, delta);
                combat.maximum_attack = add_battle_fairy_u32(
                    combat.maximum_attack,
                    delta * f64::from(coefficients.str_to_max_attack[occupation]),
                );
                combat.burden = add_battle_fairy_u16(
                    combat.burden,
                    delta * f64::from(coefficients.str_to_burden[occupation]),
                );
            }
            GAP_BF_AGILITY => {
                combat.dexterity = add_battle_fairy_u32(combat.dexterity, delta);
                combat.minimum_attack = add_battle_fairy_u32(
                    combat.minimum_attack,
                    delta * f64::from(coefficients.dex_to_min_attack[occupation]),
                );
                combat.reank = add_battle_fairy_u16(
                    combat.reank,
                    delta * f64::from(coefficients.dex_to_stiff[occupation]),
                );
            }
            GAP_BF_SPRITUALISM => {
                combat.intelligence = add_battle_fairy_u32(combat.intelligence, delta);
                combat.element_modify = add_battle_fairy_i32(
                    combat.element_modify,
                    delta * f64::from(coefficients.int_to_element[occupation]),
                );
                combat.maximum_mp = add_battle_fairy_u32(
                    combat.maximum_mp,
                    delta * f64::from(coefficients.int_to_max_mp[occupation]),
                );
                combat.element_resistance = add_battle_fairy_u32(
                    combat.element_resistance,
                    delta * f64::from(coefficients.int_to_resistant[occupation]),
                );
            }
            GAP_BF_STRENGH => {
                combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, delta);
            }
            _ => {}
        }
    }

    pub(crate) fn upgrade_battle_fairy_equipment(
        &mut self,
        factory: &CGoodsFactory,
        log_gates: BattleFairyUpgradeLogGates,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyUpgradeReport {
        let player_id = self.player_id();
        let price = self.battle_fairy_container.upgrade_price(factory);
        let mut report = BattleFairyUpgradeReport {
            player_id,
            outcome: BattleFairyUpgradeOutcome::MissingRegion,
            price,
            probability: 0,
            previous_level: None,
            resulting_level: None,
            consumed_gems: Vec::new(),
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if self.server_region_id.is_none() {
            return report;
        }
        if self.wallet.currency_amount() < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0015", Some(price));
            return report;
        }
        let Some(equipment) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::Equipment.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        };
        if !equipment.can_battle_fairy_equipment_upgrade(factory) {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        }
        let current_level = equipment.addon_property_value(factory, GAP_BF_WEAPON_LEVEL, 1);
        report.previous_level = Some(current_level);
        let target = BattleFairyUpgradeGoodsSnapshot::capture(equipment);
        let Some(base_gem) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::GemBase.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::MissingBaseGem;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0013", None);
            return report;
        };
        let minimum = base_gem.addon_property_value(factory, GAP_GEM_LEVEL, 1);
        let maximum = base_gem
            .addon_property_value(factory, GAP_GEM_LEVEL, 2)
            .max(minimum);
        if current_level < minimum || maximum < current_level {
            report.outcome = BattleFairyUpgradeOutcome::GemLevelMismatch;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0012", None);
            return report;
        }
        if 98 < current_level as u32 {
            report.outcome = BattleFairyUpgradeOutcome::MaximumLevel;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0021", None);
            return report;
        }
        report.probability = self.battle_fairy_container.probability(factory);
        if self.wallet.currency_amount() < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0020", None);
            return report;
        }
        let gems = [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ]
        .map(|cell| {
            self.battle_fairy_container
                .base()
                .get_goods(cell.position())
                .map(BattleFairyUpgradeGoodsSnapshot::capture)
        });
        let money = self.decrease_money(price, factory);
        report.effects.push(BattleFairyUpgradeEffect::MoneyChanged {
            player_id,
            previous: money.previous,
            current: money.current,
            outcome: money.outcome,
        });

        let audit_player = BattleFairyUpgradePlayerSnapshot {
            pk_count: self.base_properties.pk_count,
            money: self.money,
            depot_money: self.depot_money(),
            region_id: self.server_region_id.unwrap_or_default(),
            tile_x: self.shape().get_tile_x().unwrap_or_default(),
            tile_y: self.shape().get_tile_y().unwrap_or_default(),
            client_ip: self.client_ip,
        };

        let success = (random(100) as u32).wrapping_add(1) <= report.probability;
        let mut target_present = true;
        if success {
            let increase = self.battle_fairy_container.success_result(factory, random);
            let target_level = (current_level as u32).wrapping_add(increase).min(99) as i32;
            if let Some(goods) = self
                .battle_fairy_container
                .base_mut()
                .get_goods_mut(BattleFairyCell::Equipment.position())
            {
                let _upgraded = factory.upgrade_battle_fairy_equipment(goods, target_level);
            }
            report.outcome = BattleFairyUpgradeOutcome::Succeeded;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0002", None);
            if log_gates.success {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 1,
                    player_id,
                    player: audit_player,
                    target: target.clone(),
                    gems: gems.clone(),
                });
            }
        } else {
            if log_gates.failure {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 2,
                    player_id,
                    player: audit_player,
                    target: target.clone(),
                    gems: gems.clone(),
                });
            }
            match self.battle_fairy_container.fail_result(factory) {
                1 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0016", None);
                }
                2 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDowngraded;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0017", None);
                    if current_level != 0
                        && let Some(goods) = self
                            .battle_fairy_container
                            .base_mut()
                            .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory
                            .upgrade_battle_fairy_equipment(goods, current_level.wrapping_sub(1));
                    }
                }
                3 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedReset;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0018", None);
                    if let Some(goods) = self
                        .battle_fairy_container
                        .base_mut()
                        .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory.upgrade_battle_fairy_equipment(goods, 0);
                    }
                }
                4 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDestroyed;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0019", None);
                    if log_gates.lost_target {
                        report.effects.push(BattleFairyUpgradeEffect::Audit {
                            message_type: 0x0006_0202,
                            event: 5,
                            player_id,
                            player: audit_player,
                            target: target.clone(),
                            gems: gems.clone(),
                        });
                    }
                    if let Some((_goods, removal)) =
                        self.battle_fairy_container.delete_upgrade_target()
                    {
                        target_present = false;
                        report
                            .effects
                            .push(BattleFairyUpgradeEffect::TargetDeleted {
                                player_id,
                                goods: target.clone(),
                                position: BattleFairyCell::Equipment.position(),
                                removal,
                            });
                    }
                }
                _ => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                }
            }
        }
        if target_present
            && let Some(goods) = self
                .battle_fairy_container
                .base()
                .get_goods(BattleFairyCell::Equipment.position())
        {
            report.resulting_level =
                Some(goods.addon_property_value(factory, GAP_BF_WEAPON_LEVEL, 1));
            report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: goods.identity(),
                    old_client_payload: encode_old_client(goods),
                },
            ));
        }

        for cell in [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ] {
            let was_present = self
                .battle_fairy_container
                .base()
                .get_goods(cell.position())
                .is_some();
            let Some(consumed) = self.battle_fairy_container.consume_upgrade_gem(cell) else {
                if was_present || cell == BattleFairyCell::GemBase {
                    report.outcome = BattleFairyUpgradeOutcome::ConsumptionStopped;
                    break;
                }
                continue;
            };
            report.consumed_gems.push(consumed.clone());
            report.effects.push(BattleFairyUpgradeEffect::GemConsumed {
                player_id,
                consumed: consumed.clone(),
            });
            if !consumed.removed
                && let Some(goods) = self
                    .battle_fairy_container
                    .base()
                    .get_goods(cell.position())
            {
                report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
            }
        }
        report
    }

    pub(crate) fn reset_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialResetReport {
        let player_id = self.player_id();
        let mut report = BattleFairyPotentialResetReport {
            player_id,
            outcome: BattleFairyPotentialResetOutcome::MissingHeadgear,
            recovered_potential: 0,
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyPotentialResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialResetOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }

        let reset_index = factory.query_goods_id_by_original_name(Some(b"ZHQLS01"));
        let reset_item = self
            .packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == reset_index)
            .map(|goods| (goods.identity(), goods.amount()));
        let Some((reset_identity, reset_amount)) = reset_item else {
            report.outcome = BattleFairyPotentialResetOutcome::MissingResetItem;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0010",
                    color: 0xffff_ffff,
                });
            return report;
        };
        let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
        let (remaining_amount, consumed, removal) = if reset_amount == 0 {
            (0, false, None)
        } else if reset_amount == 1 {
            let removal = self.packet.remove_goods(reset_identity.ex_id);
            (
                if removal.is_some() { 0 } else { reset_amount },
                removal.is_some(),
                removal,
            )
        } else {
            let remaining = reset_amount.wrapping_sub(1);
            let mut consumed = false;
            if let Some(position) = reset_position
                && let Some(goods) = self.packet.get_goods_mut(position)
            {
                goods.set_amount(remaining);
                consumed = true;
            }
            (
                if consumed { remaining } else { reset_amount },
                consumed,
                None,
            )
        };
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PacketItemConsumed {
                player_id,
                goods: reset_identity,
                position: reset_position,
                previous_amount: reset_amount,
                remaining_amount,
                consumed,
                removal,
            });

        let recovered = {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("headgear проверен до packet consumption");
            let mut take = |tracked, property| {
                let value = goods.addon_property_value(factory, tracked, 1);
                let _tracked_stored = goods.set_addon_property_value_core(tracked, 1, 0);
                let current = goods.addon_property_value(factory, property, 1);
                let _property_stored =
                    goods.set_addon_property_value_core(property, 1, current.wrapping_sub(value));
                value
            };
            let attack = take(GAP_BF_ATTACK_POTENTIAL, GAP_BF_ATTACK);
            let sprite = take(GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITE);
            let blast = take(GAP_BF_BLAST_POTENTIAL, GAP_BF_BLAST);
            let brave = take(GAP_BF_BRAVE_POTENTIAL, GAP_BF_BRAVE);
            let agility = take(GAP_BF_AGILITY_POTENTIAL, GAP_BF_AGILITY);
            let spiritualism = take(GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_SPRITUALISM);
            let strength = take(GAP_BF_STRENGH_POTENTIAL, GAP_BF_STRENGH);
            let recovered = ((f64::from(sprite) + f64::from(attack)) * (2.0 / 3.0)
                + f64::from(blast)
                + f64::from(brave)
                + f64::from(agility)
                + f64::from(spiritualism)
                + f64::from(strength))
            .round() as i32;
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_add(recovered),
            );
            (recovered, brave, agility, spiritualism, strength)
        };
        report.recovered_potential = recovered.0;
        self.set_strength(
            self.combat_properties
                .strength
                .wrapping_sub((f64::from(recovered.1) * 0.00001).round() as u32),
        );
        self.set_dexterity(
            self.combat_properties
                .dexterity
                .wrapping_sub((f64::from(recovered.2) * 0.00001).round() as u32),
        );
        self.set_maximum_hp(
            self.combat_properties
                .maximum_hp
                .wrapping_sub((f64::from(recovered.4) * 0.00001).round() as u32),
        );
        self.set_intelligence(
            self.combat_properties
                .intelligence
                .wrapping_sub((f64::from(recovered.3) * 0.00001).round() as u32),
        );
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PropertiesChanged { player_id });
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("reset не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairyPotentialResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairyPotentialResetOutcome::Reset;
        report
    }

    /// Полный player-side `CBattleFairyContainer::ResetSkill`. `consume_item`
    /// соответствует третьему native аргументу: script allocation передаёт
    /// ноль, прямой gameplay caller может потребовать `ZHJNS01/02`.
    pub(crate) fn reset_battle_fairy_skill(
        &mut self,
        battle_fairy_enabled: bool,
        position: i32,
        consume_item: bool,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairySkillResetReport {
        let player_id = self.player_id();
        let mut report = BattleFairySkillResetReport {
            player_id,
            position,
            outcome: BattleFairySkillResetOutcome::MissingHeadgear,
            previous_skill: None,
            selected_skill: None,
            detached_skill_ids: Vec::new(),
            attached_skill_ids: Vec::new(),
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySkillResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairySkillResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySkillResetOutcome::InvalidHeadgear;
            return report;
        }

        if consume_item {
            let reset_name = match position {
                3..=5 => Some(b"ZHJNS01".as_slice()),
                6 => Some(b"ZHJNS02".as_slice()),
                _ => None,
            };
            if let Some(reset_name) = reset_name {
                let reset_index = factory.query_goods_id_by_original_name(Some(reset_name));
                let reset_item = self
                    .packet
                    .base()
                    .traversing_goods()
                    .find(|goods| goods.base_properties_index() == reset_index)
                    .map(|goods| (goods.identity(), goods.amount()));
                let Some((reset_identity, reset_amount)) = reset_item else {
                    report.outcome = BattleFairySkillResetOutcome::MissingResetItem;
                    report
                        .effects
                        .push(BattleFairySkillResetEffect::Notification {
                            player_id,
                            string_id: BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING,
                            color: 0xffff_ffff,
                        });
                    return report;
                };
                let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
                let (remaining_amount, consumed, removal) = if reset_amount == 0 {
                    (0, false, None)
                } else if reset_amount == 1 {
                    let removal = self.packet.remove_goods(reset_identity.ex_id);
                    (
                        if removal.is_some() { 0 } else { reset_amount },
                        removal.is_some(),
                        removal,
                    )
                } else {
                    let remaining = reset_amount.wrapping_sub(1);
                    let mut consumed = false;
                    if let Some(reset_position) = reset_position
                        && let Some(goods) = self.packet.get_goods_mut(reset_position)
                    {
                        goods.set_amount(remaining);
                        consumed = true;
                    }
                    (
                        if consumed { remaining } else { reset_amount },
                        consumed,
                        None,
                    )
                };
                report
                    .effects
                    .push(BattleFairySkillResetEffect::PacketItemConsumed {
                        player_id,
                        goods: reset_identity,
                        previous_amount: reset_amount,
                        remaining_amount,
                        consumed,
                        removal,
                    });
            }
        }

        let (current_skills, current_all_skill) = {
            let goods = self
                .equipment
                .get_goods(10)
                .expect("headgear остаётся equipped после reset-item consumption");
            (
                [
                    goods.addon_property_value(factory, GAP_BF_SKY_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_EARTH_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_MAN_SKILL, 2) as u32,
                ],
                goods.addon_property_value(factory, GAP_BF_ALL_SKILL, 2) as u32,
            )
        };
        let (property, previous_skill, replaced) = match position {
            3..=5 => {
                let replaced = (position - 3) as usize;
                (
                    GAP_BF_SKY_SKILL + replaced as i32,
                    current_skills[replaced],
                    Some(replaced),
                )
            }
            6 => (GAP_BF_ALL_SKILL, current_all_skill, None),
            _ => {
                report.outcome = BattleFairySkillResetOutcome::InvalidPosition;
                return report;
            }
        };
        report.previous_skill = Some(previous_skill);

        // В каждом native switch-case полный detach расположен перед первым
        // random(), а не только перед addon mutation.
        let old_entries = self.war_soul_skill_entries(factory);
        for (skill_id, _) in old_entries {
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            report.detached_skill_ids.push(skill_id);
            // Native `DelWarSoulSkillInPlayer` вызывает TellClient после
            // DelSkill. Поэтому packet удаления существует лишь если skill
            // пережил отказ category lookup.
            if let Some(skill) = self.move_shape.skill(skill_id) {
                report
                    .effects
                    .push(BattleFairySkillResetEffect::SkillRemoved(
                        BattleFairySkillRemoved {
                            message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                            player_id,
                            skill_id,
                            skill_name: skill.name().to_vec(),
                        },
                    ));
            }
        }

        let selected_skill = match replaced {
            Some(replaced) => loop {
                let candidate = SKILL_POJIA.wrapping_add(random(13) as u32);
                if current_skills.contains(&candidate) {
                    continue;
                }
                let conflicts = unpaired_battle_fairy_skill(candidate).is_some_and(|paired| {
                    current_skills
                        .iter()
                        .enumerate()
                        .any(|(index, &skill)| index != replaced && skill == paired)
                });
                if !conflicts {
                    break candidate;
                }
            },
            None => loop {
                let candidate = SKILL_LEIMING.wrapping_add(random(3) as u32);
                if candidate != current_all_skill {
                    break candidate;
                }
            },
        };
        report.selected_skill = Some(selected_skill);

        {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("skill detach не отделяет equipped headgear");
            let _level_cleared = goods.set_addon_property_value_core(property, 1, 0);
            let _skill_cleared = goods.set_addon_property_value_core(property, 2, 0);
            let _level_stored = goods.set_addon_property_value_core(property, 1, 1);
            let _skill_stored =
                goods.set_addon_property_value_core(property, 2, selected_skill as i32);
        }

        let new_entries = self.war_soul_skill_entries(factory);
        for (skill_id, level) in new_entries {
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id) {
                report.attached_skill_ids.push(skill_id);
                report.effects.push(BattleFairySkillResetEffect::SkillAdded(
                    battle_fairy_skill_snapshot(player_id, skill),
                ));
            }
        }

        let Some(selected) = self.move_shape.skill(selected_skill) else {
            report.outcome = BattleFairySkillResetOutcome::SelectedSkillUnavailable;
            return report;
        };
        report
            .effects
            .push(BattleFairySkillResetEffect::SelectedSkillLearned(
                battle_fairy_skill_snapshot(player_id, selected),
            ));
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("ResetSkill не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairySkillResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairySkillResetOutcome::Reset;
        report
    }

    /// Player-owned `DelWarSoulSkillInPlayer` перед запуском reset-script.
    /// Native `TellClient(false)` уже после `DelSkill` не находит удалённый
    /// skill, поэтому наблюдаемым результатом остаётся ordered detach state.
    pub(crate) fn detach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<u32> {
        let mut detached = Vec::new();
        for (skill_id, _) in self.war_soul_skill_entries(factory) {
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            detached.push(skill_id);
        }
        detached
    }

    /// Player-owned `AddWarSoulSkillToPalyer` после reset-script: addon state
    /// перечитывается из того же equipped headgear, затем каждый достигнутый
    /// skill публикуется через обычный `TellClient(true)` snapshot.
    pub(crate) fn attach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillAdded> {
        let player_id = self.player_id();
        let mut attached = Vec::new();
        for (skill_id, level) in self.war_soul_skill_entries(factory) {
            if skill_id == 0 {
                continue;
            }
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id) {
                attached.push(battle_fairy_skill_snapshot(player_id, skill));
            }
        }
        attached
    }

    fn war_soul_skill_entries(&self, factory: &CGoodsFactory) -> [(u32, i32); 9] {
        let Some(goods) = self.equipment.get_goods(10) else {
            return [(0, 0); 9];
        };
        war_soul_skill_entries_from_goods(goods, factory)
    }

    /// Полный player-side `skillmessage 0x90001` после успешного decoder-а.
    /// Contend notification не блокирует запрос; `ClearEmotion` всегда
    /// предшествует authorization и AI dispatch.
    pub(crate) fn request_player_skill(
        &mut self,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> PlayerSkillRequestReport {
        self.request_player_skill_core(request, facts, None, "GS0090", skill_factory)
    }

    /// Item-skill `0x90004` использует переданный client level, а успешная
    /// ветвь добавляет ID в native ordered item-skill vector перед AI effect.
    pub(crate) fn request_item_skill(
        &mut self,
        request: PlayerSkillRequest,
        skill_level: i32,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> PlayerSkillRequestReport {
        self.request_player_skill_core(request, facts, Some(skill_level), "GS1039", skill_factory)
    }

    fn request_player_skill_core(
        &mut self,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        item_skill_level: Option<i32>,
        contend_string_id: &'static str,
        skill_factory: &CSkillFactory,
    ) -> PlayerSkillRequestReport {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut report = PlayerSkillRequestReport {
            player_id,
            region_id: self.server_region_id,
            skill_id,
            skill_level: 0,
            target_type: request.target_type,
            target_id: request.target_id,
            target_x: request.target_x,
            target_y: request.target_y,
            outcome: PlayerSkillRequestOutcome::Unauthorized,
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if self.contend_state && facts.symbol_attackable {
            report.effects.push(PlayerSkillRequestEffect::Notification {
                player_id,
                string_id: contend_string_id,
                color: 0xffff_ffff,
                message_type: 0xffff_0000,
            });
        }

        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
        report.effects.push(PlayerSkillRequestEffect::ClearEmotion);

        let skill_level = item_skill_level.unwrap_or_else(|| {
            self.move_shape
                .skill(skill_id)
                .map_or(0, MoveShapeSkill::level)
        });
        report.skill_level = skill_level;
        if skill_level == 0 {
            push_player_skill_reject(&mut report);
            return report;
        }
        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (target_x, target_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y())
            {
                (Ok(x), Ok(y)) => (x, y),
                (Err(error), _) | (_, Err(error)) => {
                    report.outcome = PlayerSkillRequestOutcome::CoordinateBlocked(error);
                    return report;
                }
            };
            report.target_type = self.shape().identity().object_type;
            report.target_id = player_id;
            report.target_x = target_x;
            report.target_y = target_y;
        }
        if !facts.player_ai_available {
            report.outcome = PlayerSkillRequestOutcome::AiUnavailable;
            return report;
        }

        let dispatch = if report.target_type == 0 || report.target_id == 0 {
            if report.target_x == 0 || report.target_y == 0 {
                PlayerSkillDispatch::SelfTarget {
                    skill_id,
                    player_id,
                }
            } else {
                PlayerSkillDispatch::Point {
                    skill_id,
                    x: report.target_x,
                    y: report.target_y,
                }
            }
        } else {
            if self.server_region_id.is_none() {
                report.outcome = PlayerSkillRequestOutcome::MissingRegion;
                return report;
            }
            let target = ShapeIdentity {
                object_type: report.target_type,
                id: report.target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if !facts.object_target_available {
                report.outcome = PlayerSkillRequestOutcome::MissingTarget;
                push_player_skill_reject(&mut report);
                return report;
            }
            PlayerSkillDispatch::Object { skill_id, target }
        };
        if item_skill_level.is_some() {
            self.move_shape.set_item_skill(skill_id);
        }
        report
            .effects
            .push(PlayerSkillRequestEffect::AiDispatch(dispatch));
        report.outcome = PlayerSkillRequestOutcome::Queued;
        report
    }

    /// Полный player-side `skillmessage` opcode `0x90005` после успешного
    /// packet decode. Contend notification намеренно не блокирует запрос.
    pub(crate) fn request_battle_fairy_skill(
        &self,
        battle_fairy_enabled: bool,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> BattleFairySkillRequestReport {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut report = BattleFairySkillRequestReport {
            player_id,
            skill_id,
            skill_level: 0,
            target_type: request.target_type,
            target_id: request.target_id,
            target_x: request.target_x,
            target_y: request.target_y,
            outcome: BattleFairySkillRequestOutcome::MissingHeadgear,
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySkillRequestOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairySkillRequestEffect::Notification {
                    player_id,
                    string_id: "ZHGS0037",
                    color: 0xffff_0000,
                    message_type: 0,
                });
            return report;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(goods_factory, GAP_BF_HP, 1) == 0 {
            report.outcome = BattleFairySkillRequestOutcome::NoHitPoints;
            return report;
        }
        if self.contend_state && facts.symbol_attackable {
            report
                .effects
                .push(BattleFairySkillRequestEffect::Notification {
                    player_id,
                    string_id: "ZHGS0038",
                    color: 0xffff_ffff,
                    message_type: 0xffff_0000,
                });
        }

        let skill_level =
            check_battle_fairy_skill(goods, goods_factory, request.property_offset, skill_id);
        report.skill_level = skill_level;
        if skill_level == 0 {
            report.outcome = BattleFairySkillRequestOutcome::Unauthorized;
            push_battle_fairy_skill_reject(&mut report);
            return report;
        }

        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (target_x, target_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y())
            {
                (Ok(x), Ok(y)) => (x, y),
                (Err(error), _) | (_, Err(error)) => {
                    report.outcome = BattleFairySkillRequestOutcome::CoordinateBlocked(error);
                    return report;
                }
            };
            report.target_type = self.shape().identity().object_type;
            report.target_id = player_id;
            report.target_x = target_x;
            report.target_y = target_y;
        }
        if !facts.player_ai_available {
            report.outcome = BattleFairySkillRequestOutcome::AiUnavailable;
            return report;
        }

        let dispatch = if report.target_type == 0 || report.target_id == 0 {
            if report.target_x == 0 || report.target_y == 0 {
                BattleFairySkillDispatch::SelfTarget {
                    skill_id,
                    player_id,
                }
            } else {
                BattleFairySkillDispatch::Point {
                    skill_id,
                    x: report.target_x,
                    y: report.target_y,
                }
            }
        } else {
            if self.server_region_id.is_none() {
                report.outcome = BattleFairySkillRequestOutcome::MissingRegion;
                return report;
            }
            let target = ShapeIdentity {
                object_type: report.target_type,
                id: report.target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if !facts.object_target_available {
                report.outcome = BattleFairySkillRequestOutcome::MissingTarget;
                push_battle_fairy_skill_reject(&mut report);
                return report;
            }
            BattleFairySkillDispatch::Object { skill_id, target }
        };
        report
            .effects
            .push(BattleFairySkillRequestEffect::AiDispatch(dispatch));
        report.outcome = BattleFairySkillRequestOutcome::Queued;
        report
    }

    /// Достигнутая часть exact `RefreshContainerOwners`: owner ID должен быть
    /// перепривязан после создания player identity или его восстановления.
    pub(crate) const fn refresh_reached_container_owners(&mut self, player_id: i32) {
        self.bank.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.depot
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.hand.set_owner(PLAYER_TYPE, player_id);
        self.enhancement
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.packet.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.wallet.set_owner(PLAYER_TYPE, player_id);
        self.yuan_bao.set_owner(PLAYER_TYPE, player_id);
        self.ji_fen.set_owner(PLAYER_TYPE, player_id);
        self.equipment.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.auction_listing
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.auction_goods
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.auction_wallet.set_owner(PLAYER_TYPE, player_id);
        self.ci_qing.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.ci_qing_compose
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.fairy_container
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.battle_fairy_container
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
    }

    pub(crate) const fn set_pk_count(&mut self, value: u16) {
        self.base_properties.pk_count = value;
    }

    pub(crate) const fn set_occupation(&mut self, occupation: u8) {
        self.base_properties.occupation = occupation;
    }

    pub(crate) const fn set_ci_qing_open(&mut self, value: bool) {
        self.ci_qing_open = value;
    }

    pub(crate) const fn set_experience(&mut self, value: u32) {
        self.base_properties.experience = value;
    }

    pub(crate) const fn experience(&self) -> u32 {
        self.base_properties.experience
    }

    pub(crate) const fn vigour(&self) -> u32 {
        self.base_properties.vigour
    }

    pub(crate) const fn set_vigour(&mut self, value: u32) {
        self.base_properties.vigour = value;
    }

    pub(crate) const fn set_script_vigour(&mut self, value: i32) -> i32 {
        self.base_properties.vigour = value as u32;
        value
    }

    pub(crate) const fn set_script_experience(&mut self, value: i32) -> i32 {
        self.base_properties.experience = value as u32;
        value
    }

    pub(crate) const fn fairy_container_enabled(&self) -> bool {
        self.base_properties.fairy_container_enabled
    }

    /// Граница восстановления `m_BaseProperty.bFairyContainerEnabled` из
    /// persisted player snapshot; default остаётся выключенным до decode.
    pub(crate) const fn set_fairy_container_enabled(&mut self, value: bool) {
        self.base_properties.fairy_container_enabled = value;
    }

    pub(crate) const fn restore_appearance_and_mode(
        &mut self,
        head_picture: i32,
        face_picture: i32,
        mode: u32,
    ) {
        self.base_properties.head_picture = head_picture;
        self.base_properties.face_picture = face_picture;
        self.base_properties.mode = mode;
    }

    pub(crate) const fn appearance_and_mode(&self) -> (i32, i32, u32) {
        (
            self.base_properties.head_picture,
            self.base_properties.face_picture,
            self.base_properties.mode,
        )
    }

    pub(crate) const fn health(&self) -> u32 {
        self.base_properties.health
    }

    pub(crate) const fn maximum_health(&self) -> u32 {
        self.combat_properties.maximum_hp
    }

    pub(crate) const fn maximum_mana(&self) -> u32 {
        self.combat_properties.maximum_mp
    }

    /// Owned scalar tail `OnRelive` после external passive/enter/update hooks.
    pub(crate) fn apply_relive_scalars(
        &mut self,
    ) -> Result<PlayerReliveMutation, ShapeCoordinateBlock> {
        let previous_x = self.shape().get_tile_x()?;
        let previous_y = self.shape().get_tile_y()?;
        let direction = self.shape().get_direction();
        self.set_health(self.maximum_health());
        self.set_mana(self.maximum_mana());
        self.move_shape.shape_mut().set_action(0);
        self.move_shape.shape_mut().set_position(0);
        Ok(PlayerReliveMutation {
            player_id: self.player_id(),
            previous_x,
            previous_y,
            direction,
            health: self.health(),
            mana: self.mana(),
        })
    }

    pub(crate) const fn mana(&self) -> u32 {
        self.base_properties.mana
    }

    pub(crate) const fn set_maximum_hp(&mut self, value: u32) {
        self.combat_properties.maximum_hp = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_mp(&mut self, value: u32) {
        self.combat_properties.maximum_mp = clamp_combat_scalar(value);
    }

    /// Exact `SetHP` сначала записывает вход, затем перечитывает виртуальный
    /// `GetMaxHP`; в typed owner-е это текущее combat поле.
    pub(crate) const fn set_health(&mut self, value: u32) {
        self.base_properties.health = if self.combat_properties.maximum_hp < value {
            self.combat_properties.maximum_hp
        } else {
            value
        };
    }

    pub(crate) const fn set_mana(&mut self, value: u32) {
        self.base_properties.mana = if self.combat_properties.maximum_mp < value {
            self.combat_properties.maximum_mp
        } else {
            value
        };
    }

    pub(crate) const fn set_strength(&mut self, value: u32) {
        self.combat_properties.strength = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_dexterity(&mut self, value: u32) {
        self.combat_properties.dexterity = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_constitution(&mut self, value: u32) {
        self.combat_properties.constitution = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_intelligence(&mut self, value: u32) {
        self.combat_properties.intelligence = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_minimum_attack(&mut self, value: u32) {
        self.combat_properties.minimum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_attack(&mut self, value: u32) {
        self.combat_properties.maximum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_defense(&mut self, value: u32) {
        self.combat_properties.defense = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_element_resistance(&mut self, value: u32) {
        self.combat_properties.element_resistance = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_blast_defense_scale(&mut self, value: f32) {
        self.combat_properties.blast_defense_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_full_miss_scale(&mut self, value: f32) {
        self.combat_properties.full_miss_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_critical_rate(&mut self, value: f32) {
        self.combat_properties.critical_rate_bits =
            (if value < 1.0 { 1.0 } else { value }).to_bits();
    }

    pub(crate) const fn set_contribution(&mut self, value: i32) {
        self.contribution = if value < CONTRIBUTION_MINIMUM {
            CONTRIBUTION_MINIMUM
        } else if value > CONTRIBUTION_MAXIMUM {
            CONTRIBUTION_MAXIMUM
        } else {
            value
        };
    }

    /// `lMaxFetchPower` в exact сравнивался после unsigned cast, поэтому
    /// отрицательный setup limit становится большим unsigned пределом.
    pub(crate) const fn set_fetch_power(&mut self, value: u32, setup_maximum: i32) {
        let maximum = setup_maximum as u32;
        self.base_properties.fetch_power = if maximum < value { maximum } else { value };
    }

    pub(crate) const fn fetch_power(&self) -> u32 {
        self.base_properties.fetch_power
    }

    /// Player-owned mutation `ReviveBattleFairy`; client goods/state wire
    /// остаётся у вызывающего `CGame`, уже после изменения всех полей.
    pub(crate) fn revive_battle_fairy(&mut self, factory: &CGoodsFactory) -> bool {
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return false;
        };
        if goods.addon_property_value(factory, GAP_BF_HP, 1) > 0 {
            return false;
        }
        let maximum_hp = goods.addon_property_value(factory, GAP_BF_MAX_HP, 1);
        let maximum_mp = goods.addon_property_value(factory, GAP_BF_MAX_MP, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, maximum_hp);
        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, maximum_mp);
        self.base_properties.battle_fairy_recall = true;
        self.base_properties.battle_fairy_died = false;
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        true
    }

    pub(crate) const fn set_battle_fairy_recall(&mut self, value: bool) {
        self.base_properties.battle_fairy_recall = value;
    }

    pub(crate) const fn set_battle_fairy_died(&mut self, value: bool) {
        self.base_properties.battle_fairy_died = value;
    }

    /// Exact `SetSilence`: начало хранится в минутах `timeGetTime`, а
    /// не абсолютным deadline в миллисекундах.
    pub(crate) const fn set_silence(&mut self, minutes: i32, now_milliseconds: u32) {
        if minutes > 0 {
            self.silence_minutes = minutes;
            self.silence_timestamp_minutes = now_milliseconds / 60_000;
        } else {
            self.silence_minutes = 0;
            self.silence_timestamp_minutes = 0;
        }
    }

    /// Exact `IsInSilence`: равенство deadline ещё считается silence; после
    /// первой просроченной проверки оба legacy поля обнуляются.
    pub(crate) const fn is_in_silence(&mut self, now_milliseconds: u32) -> bool {
        if self.silence_minutes == 0 {
            return false;
        }
        let deadline =
            (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
        if now_milliseconds / 60_000 <= deadline {
            return true;
        }
        self.silence_minutes = 0;
        self.silence_timestamp_minutes = 0;
        false
    }

    /// Player caller `CheckBattleFairyCombine` всегда передаёт собственный ID
    /// в исходный owner; global compose configuration остаётся явным входом.
    pub(crate) fn check_battle_fairy_combine(
        &self,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> BattleFairyCombineCheck {
        self.battle_fairy_container.check_battle_fairy_combine(
            Some(self.player_id()),
            factory,
            compose,
        )
    }

    /// Полный player-side `BatllteFairyCombine`: gate, validation, exact
    /// random/deplete/remove order, creation, skill-state и адресные effects.
    /// Battle cell не является gear slot, поэтому этот caller намеренно не
    /// запускает `BFPropertyAdd` и общий player property recalc. Transport
    /// получает уже ordered report, не подменяя неизвестные поля исторических
    /// packet-encoder-ов выдуманными нулями.
    pub(crate) fn combine_battle_fairy<Create>(
        &mut self,
        battle_fairy_enabled: bool,
        setup_maximum_fetch_power: i32,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        create_goods: &mut Create,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyCombineReport
    where
        Create: FnMut(u32, &mut dyn FnMut(i32) -> i32) -> Option<CGoods>,
    {
        let player_id = self.player_id();
        let mut report = BattleFairyCombineReport {
            player_id,
            outcome: BattleFairyCombineOutcome::Rejected,
            removed_inputs: Vec::with_capacity(3),
            effects: Vec::new(),
            deliveries: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyCombineOutcome::FeatureDisabled;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0008",
                color: 0xffff_0000,
            });
            return report;
        }

        let recipe = match self
            .battle_fairy_container
            .battle_fairy_combine_recipe(factory, compose)
        {
            Ok(recipe) => recipe,
            Err(notification) => {
                report.effects.push(BattleFairyCombineEffect::Notification {
                    player_id,
                    string_id: notification.string_id(),
                    color: 0xffff_ffff,
                });
                return report;
            }
        };
        let fetch_power = self.base_properties.fetch_power;
        if fetch_power < recipe.deplete_fetch {
            report.outcome = BattleFairyCombineOutcome::InsufficientFetchPower;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0061",
                color: 0xffff_ffff,
            });
            return report;
        }

        let success = (random(100) as f32) < recipe.success_rate;
        if !success {
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0006",
                color: 0xffff_ffff,
            });
        }
        self.set_fetch_power(
            fetch_power.wrapping_sub(recipe.deplete_fetch),
            setup_maximum_fetch_power,
        );
        report
            .effects
            .push(BattleFairyCombineEffect::FetchPowerChanged {
                message_type: BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE,
                player_id,
                subject_id: player_id,
                property_name: "dwFetchPower",
                value: self.base_properties.fetch_power,
            });

        for cell in [
            super::container::cbattlefairycontainer::BattleFairyCell::FetchBody,
            super::container::cbattlefairycontainer::BattleFairyCell::FetchStone,
            super::container::cbattlefairycontainer::BattleFairyCell::Material,
        ] {
            let Some(removed) = self
                .battle_fairy_container
                .remove_battle_fairy_combine_input(cell)
            else {
                report.outcome = BattleFairyCombineOutcome::InputRemovalStopped;
                return report;
            };
            report.effects.push(BattleFairyCombineEffect::ObjectMove(
                BattleFairyObjectMove {
                    operation: BattleFairyObjectMoveOperation::Delete,
                    player_id,
                    container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                    goods: removed.goods,
                    position: removed.cell.position(),
                    amount: removed.amount,
                    old_client_payload: None,
                },
            ));
            report.removed_inputs.push(removed);
        }

        if !success {
            report.outcome = BattleFairyCombineOutcome::Failed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0007",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        }

        let Some(created) = create_goods(recipe.index, random) else {
            report.outcome = BattleFairyCombineOutcome::CreationFailed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0003",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        };
        let created_identity = created.identity();
        let created_amount = created.amount();
        let mut incoming = Some(created);
        let stored = matches!(
            self.battle_fairy_container.add_at(
                super::container::cbattlefairycontainer::BattleFairyCell::Battle,
                &mut incoming,
                factory,
                true,
            ),
            BattleFairyContainerAddOutcome::Stored {
                base: VolumeGoodsAddOutcome::Added(_),
                ..
            }
        );
        if !stored {
            report.outcome = BattleFairyCombineOutcome::CreationRejected;
            return report;
        }

        let old_client_payload = {
            let goods = self
                .battle_fairy_container
                .base()
                .get_goods(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи сохранил goods в Battle cell");
            encode_old_client(goods)
        };
        report.effects.push(BattleFairyCombineEffect::ObjectMove(
            BattleFairyObjectMove {
                operation: BattleFairyObjectMoveOperation::New,
                player_id,
                container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                goods: created_identity,
                position: BattleFairyCell::Battle.position(),
                amount: created_amount,
                old_client_payload: Some(old_client_payload),
            },
        ));
        report.effects.push(BattleFairyCombineEffect::Notification {
            player_id,
            string_id: "ZHGS0004",
            color: 0xffff_ffff,
        });

        let mut skill_effects = Vec::with_capacity(3);
        let default_properties = {
            let (move_shape, container) = (&mut self.move_shape, &mut self.battle_fairy_container);
            let goods = container
                .base_mut()
                .get_goods_mut(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи оставляет Battle cell доступной");
            let mut register_skill = |skill: BattleFairyDefaultSkill| {
                if !move_shape.add_skill(skill.id, skill.level, skill_factory) {
                    return false;
                }
                let stored = move_shape
                    .skill(skill.id)
                    .expect("успешный AddSkill публикует найденный skill");
                skill_effects.push(BattleFairyCombineEffect::SkillAdded(
                    BattleFairySkillAdded {
                        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
                        player_id,
                        skill_id: stored.id(),
                        skill_level: stored.level(),
                        skill_type: stored.skill_type(),
                        skill_name: stored.name().to_vec(),
                    },
                ));
                true
            };
            CBattleFairyContainer::load_default_properties(
                Some(player_id),
                goods,
                factory,
                &mut register_skill,
                encode_old_client,
            )
            .expect("existing player ID разрешает LoadBFDefualtProperty")
        };
        report.effects.extend(skill_effects);
        report.effects.push(BattleFairyCombineEffect::GoodsUpdated(
            default_properties.goods_update,
        ));
        let goods_name = self
            .battle_fairy_container
            .base()
            .get_goods(super::container::cbattlefairycontainer::BattleFairyCell::Battle.position())
            .expect("созданная боевая фея остаётся в Battle cell")
            .name()
            .to_vec();
        report
            .effects
            .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                string_id: "ZHGS0005",
                account: self.account.clone(),
                goods_name,
            }));
        report.outcome = BattleFairyCombineOutcome::Created;
        report
    }

    pub(crate) fn shape_view(&self) -> Option<ShapeView> {
        let identity = self.shape().identity();
        Some(ShapeView {
            identity,
            tile_x: self.shape().get_tile_x().ok()?,
            tile_y: self.shape().get_tile_y().ok()?,
            pos_x_bits: self.shape().get_pos_x().to_bits(),
            pos_y_bits: self.shape().get_pos_y().to_bits(),
            figure: self.figure,
        })
    }
}

fn push_battle_fairy_summon_notification(
    report: &mut BattleFairySummonReport,
    string_id: &'static str,
    color: u32,
) {
    report.effects.push(BattleFairySummonEffect::Notification {
        player_id: report.player_id,
        string_id,
        color,
    });
}

fn push_battle_fairy_upgrade_notification(
    report: &mut BattleFairyUpgradeReport,
    string_id: &'static str,
    format_value: Option<u32>,
) {
    report.effects.push(BattleFairyUpgradeEffect::Notification {
        player_id: report.player_id,
        string_id,
        color: 0xffff_ffff,
        format_value,
    });
}

fn battle_fairy_skill_snapshot(player_id: i32, skill: &MoveShapeSkill) -> BattleFairySkillAdded {
    BattleFairySkillAdded {
        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
        player_id,
        skill_id: skill.id(),
        skill_level: skill.level(),
        skill_type: skill.skill_type(),
        skill_name: skill.name().to_vec(),
    }
}

fn war_soul_skill_entries_from_goods(goods: &CGoods, factory: &CGoodsFactory) -> [(u32, i32); 9] {
    let entry = |property| {
        (
            goods.addon_property_value(factory, property, 2) as u32,
            goods.addon_property_value(factory, property, 1),
        )
    };
    let mut entries = [
        entry(GAP_BF_SKY),
        entry(GAP_BF_EARTH),
        entry(GAP_BF_MAN),
        entry(GAP_BF_SKY_SKILL),
        entry(GAP_BF_EARTH_SKILL),
        entry(GAP_BF_MAN_SKILL),
        entry(GAP_BF_ALL_SKILL),
        entry(GAP_BF_HUOXIESHU_SKILL),
        entry(GAP_BF_LINGZHISHU_SKILL),
    ];
    entries[7].1 = 1;
    entries[8].1 = 1;
    entries
}

fn check_battle_fairy_skill(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property_offset: i32,
    requested_skill: u32,
) -> i32 {
    if property_offset == 0 {
        return 1;
    }
    let property = GAP_BF_MAN.wrapping_add(property_offset);
    if goods.addon_property_value(factory, property, 2) as u32 == requested_skill {
        return goods.addon_property_value(factory, property, 1);
    }
    if goods.addon_property_value(factory, GAP_BF_HUOXIESHU_SKILL, 2) == 0x222
        || goods.addon_property_value(factory, GAP_BF_LINGZHISHU_SKILL, 2) == 0x223
    {
        return 1;
    }
    0
}

fn push_battle_fairy_skill_reject(report: &mut BattleFairySkillRequestReport) {
    report
        .effects
        .push(BattleFairySkillRequestEffect::SocketReject {
            message_type: SKILL_EFFECT_MESSAGE_TYPE,
            reason: SKILL_REJECT_WAR_SOUL_REASON,
            code: SKILL_REJECT_CODE,
        });
}

fn push_player_skill_reject(report: &mut PlayerSkillRequestReport) {
    report.effects.push(PlayerSkillRequestEffect::SocketReject {
        message_type: SKILL_EFFECT_MESSAGE_TYPE,
        reason: SKILL_REJECT_REASON,
        code: SKILL_REJECT_CODE,
    });
}

/// Exact constructor map `m_UnPairSkills`, подтверждённый immediate-ами
/// `gameserver.exe` по адресу `0x00504052..0x00504149`.
const fn unpaired_battle_fairy_skill(skill_id: u32) -> Option<u32> {
    Some(match skill_id {
        530 => 534,
        531 => 535,
        532 => 536,
        533 => 537,
        534 => 530,
        535 => 531,
        536 => 532,
        537 => 533,
        _ => return None,
    })
}

const fn is_battle_fairy_property_cell(cell: BattleFairyCell) -> bool {
    matches!(
        cell,
        BattleFairyCell::Weapon
            | BattleFairyCell::Body
            | BattleFairyCell::Huxinjing
            | BattleFairyCell::Jewelry
            | BattleFairyCell::Glove
            | BattleFairyCell::Pifeng
            | BattleFairyCell::Yaodai
            | BattleFairyCell::Xiezi
    )
}

fn add_battle_fairy_addon(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    property_type: i32,
    delta: i32,
) {
    let value = goods
        .addon_property_value(factory, property_type, 1)
        .wrapping_add(delta);
    let _stored = goods.set_addon_property_value_core(property_type, 1, value);
}

fn clamp_battle_fairy_current(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    current_property: i32,
    maximum_property: i32,
) {
    let maximum = goods.addon_property_value(factory, maximum_property, 1);
    if maximum < goods.addon_property_value(factory, current_property, 1) {
        let _stored = goods.set_addon_property_value_core(current_property, 1, maximum);
    }
}

fn add_battle_fairy_u32(current: u32, delta: f64) -> u32 {
    let next = f64::from(current) + delta;
    if next < 0.0 {
        0
    } else {
        clamp_combat_scalar(next.round() as u32)
    }
}

fn add_battle_fairy_u16(current: u16, delta: f64) -> u16 {
    let next = f64::from(current) + delta;
    if next < 0.0 { 0 } else { next.round() as u16 }
}

fn add_battle_fairy_i32(current: i32, delta: f64) -> i32 {
    let next = f64::from(current) + delta;
    if next < 0.0 { 0 } else { next.round() as i32 }
}

const fn clamp_combat_scalar(value: u32) -> u32 {
    if LEGACY_COMBAT_MAXIMUM < value {
        LEGACY_COMBAT_MAXIMUM
    } else {
        value
    }
}

fn read_player_game_save_slice<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    field: &'static str,
    needed: usize,
) -> Result<&'a [u8], PlayerGameSaveCodecError> {
    let offset = *cursor;
    let Some(end) = offset.checked_add(needed) else {
        return Err(PlayerGameSaveCodecError::UnexpectedEnd {
            field,
            offset,
            needed,
            available: source.len().saturating_sub(offset),
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(PlayerGameSaveCodecError::UnexpectedEnd {
            field,
            offset,
            needed,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(bytes)
}

fn read_player_game_save_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], PlayerGameSaveCodecError> {
    Ok(read_player_game_save_slice(source, cursor, field, N)?
        .try_into()
        .expect("player wire slice имеет запрошенную длину"))
}

fn read_player_game_save_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, PlayerGameSaveCodecError> {
    Ok(read_player_game_save_slice(source, cursor, field, 1)?[0])
}

fn read_player_game_save_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, PlayerGameSaveCodecError> {
    Ok(u16::from_le_bytes(read_player_game_save_array(
        source, cursor, field,
    )?))
}

fn read_player_game_save_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, PlayerGameSaveCodecError> {
    Ok(u32::from_le_bytes(read_player_game_save_array(
        source, cursor, field,
    )?))
}

fn read_player_game_save_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, PlayerGameSaveCodecError> {
    Ok(i32::from_le_bytes(read_player_game_save_array(
        source, cursor, field,
    )?))
}

fn read_player_game_save_count(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<usize, PlayerGameSaveCodecError> {
    let count = read_player_game_save_i32(source, cursor, field)?;
    usize::try_from(count).map_err(|_| PlayerGameSaveCodecError::NegativeCount { field, count })
}

fn read_player_game_save_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    maximum: usize,
) -> Result<Vec<u8>, PlayerGameSaveCodecError> {
    let offset = *cursor;
    let tail = source.get(offset..).unwrap_or_default();
    let Some(length) = tail.iter().position(|byte| *byte == 0) else {
        return Err(PlayerGameSaveCodecError::UnexpectedEnd {
            field,
            offset,
            needed: tail.len().saturating_add(1),
            available: tail.len(),
        });
    };
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    *cursor += length + 1;
    Ok(tail[..length].to_vec())
}

fn append_player_game_save_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    length: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let count = i32::try_from(length)
        .map_err(|_| PlayerGameSaveCodecError::CollectionTooLarge { field, length })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_player_game_save_string(
    destination: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
    maximum: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    destination.extend_from_slice(&value[..length]);
    destination.push(0);
    Ok(())
}

fn read_player_wire_u16(wire: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        wire[offset..offset + 2]
            .try_into()
            .expect("base/property wire offset проверен layout-константой"),
    )
}

fn read_player_wire_u32(wire: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        wire[offset..offset + 4]
            .try_into()
            .expect("base/property wire offset проверен layout-константой"),
    )
}

fn write_player_wire_u16(wire: &mut [u8], offset: usize, value: u16) {
    wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_player_wire_u32(wire: &mut [u8], offset: usize, value: u32) {
    wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp

// ============================================================================
// FUNCTION: CPlayer::GetAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:371
// RVA: 0x00002860
// ADDRESS: 00402860
// PROTOTYPE: char * __thiscall GetAccount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPkCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:304
// RVA: 0x0001E580
// ADDRESS: 0041e580
// PROTOTYPE: void __thiscall SetPkCount(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCiQingOpenFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:250
// RVA: 0x0002ACD0
// ADDRESS: 0042acd0
// PROTOTYPE: void __thiscall SetCiQingOpenFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:289
// RVA: 0x0002ACE0
// ADDRESS: 0042ace0
// PROTOTYPE: void __thiscall SetExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:390
// RVA: 0x0002ACF0
// ADDRESS: 0042acf0
// PROTOTYPE: void __thiscall SetMaxHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:392
// RVA: 0x0002AD10
// ADDRESS: 0042ad10
// PROTOTYPE: void __thiscall SetMaxMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:398
// RVA: 0x0002AD30
// ADDRESS: 0042ad30
// PROTOTYPE: void __thiscall SetStr(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:400
// RVA: 0x0002AD50
// ADDRESS: 0042ad50
// PROTOTYPE: void __thiscall SetDex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:402
// RVA: 0x0002AD70
// ADDRESS: 0042ad70
// PROTOTYPE: void __thiscall SetCon(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:404
// RVA: 0x0002AD90
// ADDRESS: 0042ad90
// PROTOTYPE: void __thiscall SetInt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:406
// RVA: 0x0002ADB0
// ADDRESS: 0042adb0
// PROTOTYPE: void __thiscall SetMinAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:408
// RVA: 0x0002ADD0
// ADDRESS: 0042add0
// PROTOTYPE: void __thiscall SetMaxAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:416
// RVA: 0x0002ADF0
// ADDRESS: 0042adf0
// PROTOTYPE: void __thiscall SetDef(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:422
// RVA: 0x0002AE10
// ADDRESS: 0042ae10
// PROTOTYPE: void __thiscall SetElementResistant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:460
// RVA: 0x0002AE30
// ADDRESS: 0042ae30
// PROTOTYPE: void __thiscall SetBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFullMissScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:466
// RVA: 0x0002AE60
// ADDRESS: 0042ae60
// PROTOTYPE: void __thiscall SetFullMissScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCriticalRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:471
// RVA: 0x0002AE90
// ADDRESS: 0042ae90
// PROTOTYPE: void __thiscall SetCriticalRate(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetContribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:546
// RVA: 0x0002AEC0
// ADDRESS: 0042aec0
// PROTOTYPE: void __thiscall SetContribute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CPlayer::SetFetchPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:680
// RVA: 0x0002AF20
// ADDRESS: 0042af20
// PROTOTYPE: void __thiscall SetFetchPower(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// MATERIALIZED: personal-shop flag storage lives above. Non-zero assignment is
// reached only after CSessionFactory seller ownership validation in the message caller;
// `(0, 0)` remains the unconditional terminal reset.
// ============================================================================
// FUNCTION: CPlayer::GetDefaultAttackSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:562
// RVA: 0x0002AFD0
// ADDRESS: 0042afd0
// PROTOTYPE: tagSkillID __thiscall GetDefaultAttackSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDecreaseMurdererSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2941
// RVA: 0x0002B030
// ADDRESS: 0042b030
// PROTOTYPE: void __thiscall OnDecreaseMurdererSign(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CPlayer::OnUpdateMurdererSign` входит в `apply_confirmed_kill` выше.

// ============================================================================
// FUNCTION: CPlayer::IsBadman
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3089
// RVA: 0x0002B160
// ADDRESS: 0042b160
// PROTOTYPE: bool __thiscall IsBadman(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5078
// RVA: 0x0002B190
// ADDRESS: 0042b190
// PROTOTYPE: bool __thiscall IsInArea(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5123
// RVA: 0x0002B230
// ADDRESS: 0042b230
// PROTOTYPE: bool __thiscall IsInRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ActiveEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6432
// RVA: 0x0002B240
// ADDRESS: 0042b240
// PROTOTYPE: void __thiscall ActiveEquip(CGoods * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountFuMoProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7230
// RVA: 0x0002B690
// ADDRESS: 0042b690
// PROTOTYPE: int __thiscall MountFuMoProperty(GOODS_ADDON_PROPERTIES param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8582
// RVA: 0x0002C310
// ADDRESS: 0042c310
// PROTOTYPE: long __thiscall CanMountEquip(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeSkillsFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9096
// RVA: 0x0002C5C0
// ADDRESS: 0042c5c0
// PROTOTYPE: void __thiscall DecodeSkillsFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9195
// RVA: 0x0002C620
// ADDRESS: 0042c620
// PROTOTYPE: void __thiscall OnChangeProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `SetSilence/IsInSilence`
// RVA `0x0002C8A0/0x0002C8F0` материализованы выше и достигнуты GM
// `0x7FC0B/0x7FC0E`; покрытый raw удалён.
// ============================================================================
// FUNCTION: CPlayer::UpdateCurrentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9313
// RVA: 0x0002C940
// ADDRESS: 0042c940
// PROTOTYPE: void __thiscall UpdateCurrentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCriminalState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9336
// RVA: 0x0002C9A0
// ADDRESS: 0042c9a0
// PROTOTYPE: void __thiscall EnterCriminalState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterResidentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9351
// RVA: 0x0002CA40
// ADDRESS: 0042ca40
// PROTOTYPE: void __thiscall EnterResidentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterPeaceState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9361
// RVA: 0x0002CAC0
// ADDRESS: 0042cac0
// PROTOTYPE: void __thiscall EnterPeaceState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCombatState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9373
// RVA: 0x0002CB50
// ADDRESS: 0042cb50
// PROTOTYPE: void __thiscall EnterCombatState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById_FromPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9501
// RVA: 0x0002CC70
// ADDRESS: 0042cc70
// PROTOTYPE: CGoods * __thiscall GetGoodsById_FromPackage(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9513
// RVA: 0x0002CC90
// ADDRESS: 0042cc90
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendNotifyMessageA
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9558
// RVA: 0x0002CCD0
// ADDRESS: 0042ccd0
// PROTOTYPE: void __thiscall SendNotifyMessageA(char * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSystemInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9570
// RVA: 0x0002CD70
// ADDRESS: 0042cd70
// PROTOTYPE: void __thiscall SendSystemInfo(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendOtherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9581
// RVA: 0x0002CE00
// ADDRESS: 0042ce00
// PROTOTYPE: void __thiscall SendOtherInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9644
// RVA: 0x0002CE80
// ADDRESS: 0042ce80
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10030
// RVA: 0x0002CF40
// ADDRESS: 0042cf40
// PROTOTYPE: ulong __thiscall GetMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10035
// RVA: 0x0002CF50
// ADDRESS: 0042cf50
// PROTOTYPE: ulong __thiscall GetYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDepotMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10045
// RVA: 0x0002CF60
// ADDRESS: 0042cf60
// PROTOTYPE: ulong __thiscall GetDepotMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10050
// RVA: 0x0002CF70
// ADDRESS: 0042cf70
// PROTOTYPE: int __thiscall SetMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10094
// RVA: 0x0002D120
// ADDRESS: 0042d120
// PROTOTYPE: int __thiscall SetYuanBao(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10224
// RVA: 0x0002D2D0
// ADDRESS: 0042d2d0
// PROTOTYPE: undefined __thiscall CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::~CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10229
// RVA: 0x0002D2E0
// ADDRESS: 0042d2e0
// PROTOTYPE: void __thiscall ~CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10234
// RVA: 0x0002D2F0
// ADDRESS: 0042d2f0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10285
// RVA: 0x0002D3F0
// ADDRESS: 0042d3f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10309
// RVA: 0x0002D430
// ADDRESS: 0042d430
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10327
// RVA: 0x0002D450
// ADDRESS: 0042d450
// PROTOTYPE: void __thiscall RejectUseSkillRequest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CPlayer::PerformEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11059
// RVA: 0x0002D590
// ADDRESS: 0042d590
// PROTOTYPE: void __thiscall PerformEmotion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainerListener::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11134
// RVA: 0x0002D720
// ADDRESS: 0042d720
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsFactionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11200
// RVA: 0x0002D930
// ADDRESS: 0042d930
// PROTOTYPE: bool __thiscall IsFactionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsUnionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11211
// RVA: 0x0002D950
// ADDRESS: 0042d950
// PROTOTYPE: bool __thiscall IsUnionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11410
// RVA: 0x0002D980
// ADDRESS: 0042d980
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponDamageLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11439
// RVA: 0x0002D9F0
// ADDRESS: 0042d9f0
// PROTOTYPE: ulong __thiscall GetWeaponDamageLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::end_business
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12095
// RVA: 0x0002DBF0
// ADDRESS: 0042dbf0
// PROTOTYPE: void __thiscall end_business(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSessionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12173
// RVA: 0x0002DCC0
// ADDRESS: 0042dcc0
// PROTOTYPE: char * __thiscall GetSessionID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetIpAddress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12179
// RVA: 0x0002DCD0
// ADDRESS: 0042dcd0
// PROTOTYPE: char * __thiscall GetIpAddress(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12861
// RVA: 0x0002DD20
// ADDRESS: 0042dd20
// PROTOTYPE: int __thiscall DeleteSkillItem(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetWarSoulXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13071
// RVA: 0x0002DF50
// ADDRESS: 0042df50
// PROTOTYPE: void __thiscall SetWarSoulXY(tagPOINT param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13109
// RVA: 0x0002E0A0
// ADDRESS: 0042e0a0
// PROTOTYPE: void __thiscall DelWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReplacePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13276
// RVA: 0x0002E260
// ADDRESS: 0042e260
// PROTOTYPE: void __thiscall ReplacePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestorePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13300
// RVA: 0x0002E400
// ADDRESS: 0042e400
// PROTOTYPE: void __thiscall RestorePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClientMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13315
// RVA: 0x0002E4D0
// ADDRESS: 0042e4d0
// PROTOTYPE: void __thiscall TellClientMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13324
// RVA: 0x0002E570
// ADDRESS: 0042e570
// PROTOTYPE: void __thiscall TellClient(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequestWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13358
// RVA: 0x0002E720
// ADDRESS: 0042e720
// PROTOTYPE: void __thiscall RejectUseSkillRequestWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14782
// RVA: 0x0002EEF0
// ADDRESS: 0042eef0
// PROTOTYPE: ulong __thiscall GetAuctionMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AuctionLimit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14998
// RVA: 0x0002EF00
// ADDRESS: 0042ef00
// PROTOTYPE: bool __thiscall AuctionLimit(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCutLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15115
// RVA: 0x0002F0A0
// ADDRESS: 0042f0a0
// PROTOTYPE: void __thiscall SendCutLog(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountScoreAdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15155
// RVA: 0x0002F250
// ADDRESS: 0042f250
// PROTOTYPE: int __cdecl CountScoreAdd(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcWeekClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15176
// RVA: 0x0002F2D0
// ADDRESS: 0042f2d0
// PROTOTYPE: void __thiscall JJcWeekClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcSeasonClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15190
// RVA: 0x0002F320
// ADDRESS: 0042f320
// PROTOTYPE: void __thiscall JJcSeasonClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddPreItemToPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15373
// RVA: 0x0002F360
// ADDRESS: 0042f360
// PROTOTYPE: void __thiscall AddPreItemToPlayer(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17480
// RVA: 0x0002FB50
// ADDRESS: 0042fb50
// PROTOTYPE: void __thiscall SetCurFlash(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17528
// RVA: 0x0002FC50
// ADDRESS: 0042fc50
// PROTOTYPE: void __thiscall DoneFlash(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:368
// RVA: 0x000300D0
// ADDRESS: 004300d0
// PROTOTYPE: void __thiscall SetMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetRP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:370
// RVA: 0x000300F0
// ADDRESS: 004300f0
// PROTOTYPE: void __thiscall SetRP(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetVigour
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:518
// RVA: 0x00030120
// ADDRESS: 00430120
// PROTOTYPE: void __thiscall SetVigour(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PeriodicalUpdate
// STATUS: PARTIALLY_IMPLEMENTED_DEATH_STATE_TAIL
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2348
// RVA: 0x00030140
// ADDRESS: 00430140
// PROTOTYPE: void __thiscall PeriodicalUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseRp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9417
// RVA: 0x000302F0
// ADDRESS: 004302f0
// PROTOTYPE: void __thiscall IncreaseRp(int param_1, ushort param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteGoodsDelLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12633
// RVA: 0x00030430
// ADDRESS: 00430430
// PROTOTYPE: void __thiscall WriteGoodsDelLog(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14737
// RVA: 0x00030E60
// ADDRESS: 00430e60
// PROTOTYPE: bool __thiscall SetAuctionMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGM
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5044
// RVA: 0x00031430
// ADDRESS: 00431430
// PROTOTYPE: bool __thiscall IsGM(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGMLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5052
// RVA: 0x00031480
// ADDRESS: 00431480
// PROTOTYPE: long __thiscall GetGMLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10594
// RVA: 0x000314E0
// ADDRESS: 004314e0
// PROTOTYPE: ulong __thiscall DeleteGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2, ulong param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsbyGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12330
// RVA: 0x00031800
// ADDRESS: 00431800
// PROTOTYPE: int __thiscall DeleteGoodsbyGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15294
// RVA: 0x00031DE0
// ADDRESS: 00431de0
// PROTOTYPE: void __thiscall AddTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15351
// RVA: 0x00031E70
// ADDRESS: 00431e70
// PROTOTYPE: void __thiscall AddTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCiQingTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15362
// RVA: 0x00031EE0
// ADDRESS: 00431ee0
// PROTOTYPE: void __thiscall AddCiQingTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:272
// RVA: 0x00032630
// ADDRESS: 00432630
// PROTOTYPE: int __thiscall DropGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNumSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8968
// RVA: 0x00032AA0
// ADDRESS: 00432aa0
// PROTOTYPE: long __thiscall GetNumSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddSkillsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8998
// RVA: 0x00032B80
// ADDRESS: 00432b80
// PROTOTYPE: void __thiscall AddSkillsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9234
// RVA: 0x00033080
// ADDRESS: 00433080
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsEnemyFactionMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9397
// RVA: 0x00033210
// ADDRESS: 00433210
// PROTOTYPE: long __thiscall IsEnemyFactionMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCityWarEneymyFactionMemeber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9407
// RVA: 0x00033240
// ADDRESS: 00433240
// PROTOTYPE: long __thiscall IsCityWarEneymyFactionMemeber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsInPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10857
// RVA: 0x00033270
// ADDRESS: 00433270
// PROTOTYPE: void __thiscall DeleteGoodsInPacket(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11226
// RVA: 0x00033310
// ADDRESS: 00433310
// PROTOTYPE: bool __thiscall AddQuestDataByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReUseSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12832
// RVA: 0x00033610
// ADDRESS: 00433610
// PROTOTYPE: int __thiscall ReUseSkillItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JudgeZhaoMuStatus
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13478
// RVA: 0x000336B0
// ADDRESS: 004336b0
// PROTOTYPE: bool __thiscall JudgeZhaoMuStatus(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanPreAndSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15265
// RVA: 0x00033700
// ADDRESS: 00433700
// PROTOTYPE: void __thiscall CleanPreAndSkillList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15273
// RVA: 0x000337A0
// ADDRESS: 004337a0
// PROTOTYPE: void __thiscall DelTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15841
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall AddByteCiQing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddOrgSysToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1518
// RVA: 0x00033C30
// ADDRESS: 00433c30
// PROTOTYPE: bool __thiscall AddOrgSysToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteGS2WS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13511
// RVA: 0x00033E40
// ADDRESS: 00433e40
// PROTOTYPE: void __thiscall AddByteGS2WS(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10335
// RVA: 0x00034300
// ADDRESS: 00434300
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::do_coutribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11514
// RVA: 0x00034A00
// ADDRESS: 00434a00
// PROTOTYPE: void __thiscall do_coutribute(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddExploitToMurdererInCountryWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12038
// RVA: 0x00035DF0
// ADDRESS: 00435df0
// PROTOTYPE: void __thiscall AddExploitToMurdererInCountryWar(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DrawAwards
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12205
// RVA: 0x00035F70
// ADDRESS: 00435f70
// PROTOTYPE: long __thiscall DrawAwards(long param_1, int param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeBodyCheck
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12918
// RVA: 0x00036080
// ADDRESS: 00436080
// PROTOTYPE: int __thiscall ChangeBodyCheck(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAuctionMoneyMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14846
// RVA: 0x000369B0
// ADDRESS: 004369b0
// PROTOTYPE: bool __thiscall CheckAuctionMoneyMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15853
// RVA: 0x00038EA0
// ADDRESS: 00438ea0
// PROTOTYPE: void __thiscall DeByteCiQing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:357
// RVA: 0x0003A020
// ADDRESS: 0043a020
// PROTOTYPE: int __thiscall CheckGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10566
// RVA: 0x0003A300
// ADDRESS: 0043a300
// PROTOTYPE: CGUID __thiscall DeleteGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10712
// RVA: 0x0003A4D0
// ADDRESS: 0043a4d0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10776
// RVA: 0x0003A860
// ADDRESS: 0043a860
// PROTOTYPE: void __thiscall DropParticularGoodsWhenLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenRecall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10818
// RVA: 0x0003AAC0
// ADDRESS: 0043aac0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenRecall(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15230
// RVA: 0x0003AD30
// ADDRESS: 0043ad30
// PROTOTYPE: void __thiscall AddItemToTaoZhuangSkillList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15243
// RVA: 0x0003AD70
// ADDRESS: 0043ad70
// PROTOTYPE: void __thiscall AddItemToTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToCiQingTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15254
// RVA: 0x0003ADB0
// ADDRESS: 0043adb0
// PROTOTYPE: void __thiscall AddItemToCiQingTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurrentTypeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16080
// RVA: 0x0003ADF0
// ADDRESS: 0043adf0
// PROTOTYPE: void __thiscall GetCurrentTypeValue(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordOrgSysFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1459
// RVA: 0x0003C230
// ADDRESS: 0043c230
// PROTOTYPE: bool __thiscall DecordOrgSysFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquipRide
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6516
// RVA: 0x0003C5E0
// ADDRESS: 0043c5e0
// PROTOTYPE: void __thiscall MountEquipRide(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExitRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9661
// RVA: 0x0003DB50
// ADDRESS: 0043db50
// PROTOTYPE: void __thiscall OnExitRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10910
// RVA: 0x0003DB60
// ADDRESS: 0043db60
// PROTOTYPE: ulong __thiscall IncExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11240
// RVA: 0x0003E1C0
// ADDRESS: 0043e1c0
// PROTOTYPE: bool __thiscall AddQuestDataByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12365
// RVA: 0x0003E3F0
// ADDRESS: 0043e3f0
// PROTOTYPE: bool __thiscall AddItemToDelList(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelAllItemInDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12401
// RVA: 0x0003E4E0
// ADDRESS: 0043e4e0
// PROTOTYPE: bool __thiscall DelAllItemInDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12427
// RVA: 0x0003E670
// ADDRESS: 0043e670
// PROTOTYPE: void __thiscall DoneDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputeTicket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12459
// RVA: 0x0003E820
// ADDRESS: 0043e820
// PROTOTYPE: ulong __thiscall ComputeTicket(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAddGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12497
// RVA: 0x0003E920
// ADDRESS: 0043e920
// PROTOTYPE: ulong __thiscall CheckAddGoods(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12518
// RVA: 0x0003E980
// ADDRESS: 0043e980
// PROTOTYPE: bool __thiscall AddItemToMap(ulong param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12543
// RVA: 0x0003EA60
// ADDRESS: 0043ea60
// PROTOTYPE: bool __thiscall AddItemToGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelItemFromGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12567
// RVA: 0x0003EAC0
// ADDRESS: 0043eac0
// PROTOTYPE: bool __thiscall DelItemFromGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12616
// RVA: 0x0003EBD0
// ADDRESS: 0043ebd0
// PROTOTYPE: void __thiscall DoneGoodsAiTree(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateGoodsGS2C
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12784
// RVA: 0x0003EC40
// ADDRESS: 0043ec40
// PROTOTYPE: void __thiscall UpdateGoodsGS2C(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastUseSkillItemTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12826
// RVA: 0x0003ED30
// ADDRESS: 0043ed30
// PROTOTYPE: void __thiscall SetLastUseSkillItemTime(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OutputBinaryStream
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15063
// RVA: 0x0003F890
// ADDRESS: 0043f890
// PROTOTYPE: void __thiscall OutputBinaryStream(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15214
// RVA: 0x0003FA20
// ADDRESS: 0043fa20
// PROTOTYPE: void __thiscall SendTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15636
// RVA: 0x0003FAF0
// ADDRESS: 0043faf0
// PROTOTYPE: void __thiscall CleanTaoZhuangItemList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15642
// RVA: 0x0003FB60
// ADDRESS: 0043fb60
// PROTOTYPE: void __thiscall AddItemToTaoZhuangItemList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCiQingGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16023
// RVA: 0x0003FE90
// ADDRESS: 0043fe90
// PROTOTYPE: void __thiscall SendCiQingGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:530
// RVA: 0x00040C30
// ADDRESS: 00440c30
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:742
// RVA: 0x00040DC0
// ADDRESS: 00440dc0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1692
// RVA: 0x00041460
// ADDRESS: 00441460
// PROTOTYPE: void __thiscall OnExit(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044156e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1712
// RVA: 0x0004156E
// ADDRESS: 0044156e
// PROTOTYPE: undefined Catch@0044156e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0044158b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1715
// RVA: 0x0004158B
// ADDRESS: 0044158b
// PROTOTYPE: undefined FUN_0044158b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1780
// RVA: 0x000417A0
// ADDRESS: 004417a0
// PROTOTYPE: void __thiscall OnLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEquipmentWaste
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2731
// RVA: 0x000419B0
// ADDRESS: 004419b0
// PROTOTYPE: void __thiscall OnEquipmentWaste(EQUIPMENT_COLUMN param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnArmorDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2818
// RVA: 0x00041AF0
// ADDRESS: 00441af0
// PROTOTYPE: void __thiscall OnArmorDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnWeaponDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2932
// RVA: 0x00041D50
// ADDRESS: 00441d50
// PROTOTYPE: void __thiscall OnWeaponDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2966
// RVA: 0x00041D80
// ADDRESS: 00441d80
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenMurdered
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3111
// RVA: 0x00042040
// ADDRESS: 00442040
// PROTOTYPE: void __thiscall OnBeenMurdered(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5239
// RVA: 0x00042610
// ADDRESS: 00442610
// PROTOTYPE: void __thiscall MountEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00443c5b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6069
// RVA: 0x00043C5B
// ADDRESS: 00443c5b
// PROTOTYPE: undefined Catch@00443c5b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7785
// RVA: 0x00044C80
// ADDRESS: 00444c80
// PROTOTYPE: int __thiscall RestoreHp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7804
// RVA: 0x00044D50
// ADDRESS: 00444d50
// PROTOTYPE: int __thiscall RestoreMp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::Mount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8513
// RVA: 0x00044E20
// ADDRESS: 00444e20
// PROTOTYPE: int __thiscall Mount(ulong param_1, ulong param_2, ulong param_3, char * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11095
// RVA: 0x000451A0
// ADDRESS: 004451a0
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordQuestDataFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11278
// RVA: 0x000452D0
// ADDRESS: 004452d0
// PROTOTYPE: bool __thiscall DecordQuestDataFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHpMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12717
// RVA: 0x000455D0
// ADDRESS: 004455d0
// PROTOTYPE: void __thiscall RestoreHpMp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AutoAddAuctionGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14862
// RVA: 0x000466A0
// ADDRESS: 004466a0
// PROTOTYPE: void __thiscall AutoAddAuctionGoods(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16353
// RVA: 0x00047000
// ADDRESS: 00447000
// PROTOTYPE: void __thiscall MountCiQingEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00448346
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17020
// RVA: 0x00048346
// ADDRESS: 00448346
// PROTOTYPE: undefined Catch@00448346()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::~CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:256
// RVA: 0x00049860
// ADDRESS: 00449860
// PROTOTYPE: void __thiscall ~CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:286
// RVA: 0x0004A2D0
// ADDRESS: 0044a2d0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:288
// RVA: 0x0004A2E0
// ADDRESS: 0044a2e0
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:365
// RVA: 0x0004A2F0
// ADDRESS: 0044a2f0
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:366
// RVA: 0x0004A300
// ADDRESS: 0044a300
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:389
// RVA: 0x0004A340
// ADDRESS: 0044a340
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:405
// RVA: 0x0004A350
// ADDRESS: 0044a350
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:407
// RVA: 0x0004A360
// ADDRESS: 0044a360
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:409
// RVA: 0x0004A370
// ADDRESS: 0044a370
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCCH
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:413
// RVA: 0x0004A380
// ADDRESS: 0044a380
// PROTOTYPE: ushort __thiscall GetCCH(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:415
// RVA: 0x0004A390
// ADDRESS: 0044a390
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:417
// RVA: 0x0004A3A0
// ADDRESS: 0044a3a0
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:419
// RVA: 0x0004A3B0
// ADDRESS: 0044a3b0
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:421
// RVA: 0x0004A3C0
// ADDRESS: 0044a3c0
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:423
// RVA: 0x0004A3D0
// ADDRESS: 0044a3d0
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:425
// RVA: 0x0004A3E0
// ADDRESS: 0044a3e0
// PROTOTYPE: ushort __thiscall GetMpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:428
// RVA: 0x0004A3F0
// ADDRESS: 0044a3f0
// PROTOTYPE: ulong __thiscall GetExalt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:429
// RVA: 0x0004A400
// ADDRESS: 0044a400
// PROTOTYPE: void __thiscall SetExalt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:432
// RVA: 0x0004A410
// ADDRESS: 0044a410
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:434
// RVA: 0x0004A420
// ADDRESS: 0044a420
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:436
// RVA: 0x0004A430
// ADDRESS: 0044a430
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:441
// RVA: 0x0004A440
// ADDRESS: 0044a440
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:444
// RVA: 0x0004A450
// ADDRESS: 0044a450
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:446
// RVA: 0x0004A460
// ADDRESS: 0044a460
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetFullMiss
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:448
// RVA: 0x0004A470
// ADDRESS: 0044a470
// PROTOTYPE: ushort __thiscall GetFullMiss(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:941
// RVA: 0x0004A480
// ADDRESS: 0044a480
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1252
// RVA: 0x0004BA80
// ADDRESS: 0044ba80
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1867
// RVA: 0x0004C400
// ADDRESS: 0044c400
// PROTOTYPE: bool __thiscall ChangeRegion(long param_1, long param_2, long param_3, long param_4, long param_5, long param_6, long param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnStandOnSwitchPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2596
// RVA: 0x0004D420
// ADDRESS: 0044d420
// PROTOTYPE: int __thiscall OnStandOnSwitchPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDied
// STATUS: PARTIALLY_IMPLEMENTED_NATION_AND_GODS_BATTLE
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3306
// RVA: 0x0004D850
// ADDRESS: 0044d850
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:4930
// RVA: 0x00052DD0
// ADDRESS: 00452dd0
// PROTOTYPE: long __thiscall CheckLevel(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountAllEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5132
// RVA: 0x00053480
// ADDRESS: 00453480
// PROTOTYPE: void __thiscall MountAllEquip(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004535ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5183
// RVA: 0x000535EE
// ADDRESS: 004535ee
// PROTOTYPE: undefined Catch@004535ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitNameValueMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8693
// RVA: 0x00055000
// ADDRESS: 00455000
// PROTOTYPE: void __thiscall InitNameValueMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8858
// RVA: 0x000577B0
// ADDRESS: 004577b0
// PROTOTYPE: ulong __thiscall GetValue(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8889
// RVA: 0x00057AF0
// ADDRESS: 00457af0
// PROTOTYPE: ulong __thiscall SetValue(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8927
// RVA: 0x00057E60
// ADDRESS: 00457e60
// PROTOTYPE: ulong __thiscall ChangeValue(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseContinuousKill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9593
// RVA: 0x00058390
// ADDRESS: 00458390
// PROTOTYPE: void __thiscall IncreaseContinuousKill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PlayerRunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11397
// RVA: 0x000584E0
// ADDRESS: 004584e0
// PROTOTYPE: long __thiscall PlayerRunScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AdjustHonorRank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15201
// RVA: 0x000585B0
// ADDRESS: 004585b0
// PROTOTYPE: bool __thiscall AdjustHonorRank(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputerAddValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15311
// RVA: 0x000585D0
// ADDRESS: 004585d0
// PROTOTYPE: void __thiscall ComputerAddValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneTaoZhuang
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15678
// RVA: 0x00058810
// ADDRESS: 00458810
// PROTOTYPE: void __thiscall DoneTaoZhuang(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RunQuestCompleteScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17798
// RVA: 0x00058940
// ADDRESS: 00458940
// PROTOTYPE: long __thiscall RunQuestCompleteScript(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:89
// RVA: 0x000589B0
// ADDRESS: 004589b0
// PROTOTYPE: undefined __thiscall CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:601
// RVA: 0x000593E0
// ADDRESS: 004593e0
// PROTOTYPE: void __thiscall UpdateProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AI
// STATUS: PARTIALLY_IMPLEMENTED_BATTLE_FAIRY_DEATH_PREFIX
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2264
// RVA: 0x00059FF0
// ADDRESS: 00459ff0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9681
// RVA: 0x0005A170
// ADDRESS: 0045a170
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNetExID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1037
// RVA: 0x0007BD10
// ADDRESS: 0047bd10
// PROTOTYPE: long __thiscall GetNetExID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1173
// RVA: 0x00093440
// ADDRESS: 00493440
// PROTOTYPE: char * __thiscall GetLastContainerScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCharged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:317
// RVA: 0x000AEB30
// ADDRESS: 004aeb30
// PROTOTYPE: void __thiscall SetCharged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:526
// RVA: 0x000AEB40
// ADDRESS: 004aeb40
// PROTOTYPE: void __thiscall SetMaxEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCreateFactionOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1023
// RVA: 0x000AEB60
// ADDRESS: 004aeb60
// PROTOTYPE: void __thiscall SetCreateFactionOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetApplyJoinOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1026
// RVA: 0x000AEB70
// ADDRESS: 004aeb70
// PROTOTYPE: void __thiscall SetApplyJoinOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFactionDeclareWarOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1029
// RVA: 0x000AEB80
// ADDRESS: 004aeb80
// PROTOTYPE: void __thiscall SetFactionDeclareWarOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:524
// RVA: 0x000AED90
// ADDRESS: 004aed90
// PROTOTYPE: void __thiscall SetEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1172
// RVA: 0x000AEFC0
// ADDRESS: 004aefc0
// PROTOTYPE: void __thiscall SetLastContainerScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PushItemToCiQingList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:252
// RVA: 0x000AF1E0
// ADDRESS: 004af1e0
// PROTOTYPE: void __thiscall PushItemToCiQingList(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:458
// RVA: 0x001DFD60
// ADDRESS: 005dfd60
// PROTOTYPE: void __thiscall SetBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:462
// RVA: 0x001DFD90
// ADDRESS: 005dfd90
// PROTOTYPE: void __thiscall SetElementBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:464
// RVA: 0x001DFDC0
// ADDRESS: 005dfdc0
// PROTOTYPE: void __thiscall SetElementBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
