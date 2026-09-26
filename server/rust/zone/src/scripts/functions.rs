//! Данные диспетчера сценарных функций Zone: идентификаторы исторического
//! плотного диспетчера `CScript::RunFunction`, маршрутизация видов его
//! параметров и упаковка сценарного локального времени без игрового состояния.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходное владение —
//! `server/gameserver/appserver/script/function.cpp`; перенос сохраняет
//! нумерацию диспетчера и позиционные правила параметров дословно, исполнение
//! функций остаётся в старом пакете до перехода диспетчера.

use nebokrai_shared::values::TagTime;

pub const SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY: i32 = 9351;
pub const SCRIPT_FUNCTION_OPEN_DA_KONG: i32 = 9350;
pub const SCRIPT_FUNCTION_DA_KONG_MODIFY: i32 = 9352;
pub const SCRIPT_FUNCTION_DA_KONG_DELUX_MODIFY: i32 = 9353;
pub const SCRIPT_FUNCTION_MODIFY_GOODS_TIME: i32 = 9509;
pub const SCRIPT_FUNCTION_MODIFY_AUCTION_SPACE: i32 = 9513;
pub const SCRIPT_FUNCTION_AUTO_ADD_AUCTION_GOODS: i32 = 9514;
pub const SCRIPT_FUNCTION_ADD_TIME_GOODS: i32 = 9605;
pub const SCRIPT_FUNCTION_DELETE_GOODS_FROM_CI_QING: i32 = 9627;
pub const SCRIPT_FUNCTION_OPEN_CI_QING_PAGE: i32 = 9628;
pub const SCRIPT_FUNCTION_PUSH_ITEM_TO_CI_QING: i32 = 9629;
pub const SCRIPT_FUNCTION_GET_EQUIPPED_GOODS_INDEX: i32 = 9734;
pub const SCRIPT_FUNCTION_OPEN_EQUIPMENT_COMPOSE: i32 = 9354;
pub const SCRIPT_FUNCTION_OPEN_EQUIPMENT_UPGRADE: i32 = 2216;
pub const SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX: i32 = 2221;
pub const SCRIPT_FUNCTION_GET_PRECIOUS_ITEM: i32 = 2222;
pub const SCRIPT_FUNCTION_DELETE_USED_GOODS: i32 = 2231;
pub const SCRIPT_FUNCTION_CHECK_USED_GOODS: i32 = 2232;
pub const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1: i32 = 2233;
pub const SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2: i32 = 2234;
pub const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1: i32 = 2235;
pub const SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2: i32 = 2236;
pub const SCRIPT_FUNCTION_CLOSE_PRECIOUS_BOX: i32 = 2237;
pub const SCRIPT_FUNCTION_GET_CURRENT_DURABILITY: i32 = 2243;
pub const SCRIPT_FUNCTION_SET_CURRENT_DURABILITY: i32 = 2244;
pub const SCRIPT_FUNCTION_GET_SELECTED_DURABILITY: i32 = 2245;
pub const SCRIPT_FUNCTION_SET_SELECTED_DURABILITY: i32 = 2246;
pub const SCRIPT_FUNCTION_FAIRY_EXP_UP: i32 = 2249;
pub const SCRIPT_FUNCTION_RANDOM: i32 = 8;
pub const SCRIPT_FUNCTION_RGB: i32 = 9;
pub const SCRIPT_FUNCTION_TIME: i32 = 13;
pub const SCRIPT_FUNCTION_YEAR: i32 = 14;
pub const SCRIPT_FUNCTION_MONTH: i32 = 15;
pub const SCRIPT_FUNCTION_DAY: i32 = 16;
pub const SCRIPT_FUNCTION_HOUR: i32 = 17;
pub const SCRIPT_FUNCTION_MINUTE: i32 = 18;
pub const SCRIPT_FUNCTION_DAY_OF_WEEK: i32 = 19;
pub const SCRIPT_FUNCTION_HOUR_DIFF: i32 = 20;
pub const SCRIPT_FUNCTION_MINUTE_DIFF: i32 = 21;
pub const SCRIPT_FUNCTION_SECOND: i32 = 23;
pub const SCRIPT_FUNCTION_GET_STRING_BY_ID: i32 = 2000;
pub const SCRIPT_FUNCTION_CHANGE_ME: i32 = 2001;
pub const SCRIPT_FUNCTION_GET_ME: i32 = 2002;
pub const SCRIPT_FUNCTION_SET_ME: i32 = 2003;
pub const SCRIPT_FUNCTION_CHECK_LEVEL: i32 = 2004;
pub const SCRIPT_FUNCTION_SET_ENERGY: i32 = 2005;
pub const SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY: i32 = 2006;
pub const SCRIPT_FUNCTION_GET_ENERGY: i32 = 2007;
pub const SCRIPT_FUNCTION_GET_MAXIMUM_ENERGY: i32 = 2008;
pub const SCRIPT_FUNCTION_WALK_STEP: i32 = 2100;
pub const SCRIPT_FUNCTION_RUN_STEP: i32 = 2101;
pub const SCRIPT_FUNCTION_SET_PLAYER_POSITION: i32 = 2102;
pub const SCRIPT_FUNCTION_SET_PLAYER_DIRECTION: i32 = 2103;
pub const SCRIPT_FUNCTION_GET_EQUIPMENT_ID_BY_POSITION: i32 = 2206;
pub const SCRIPT_FUNCTION_UPGRADE_EQUIPMENT: i32 = 2208;
pub const SCRIPT_FUNCTION_RE_LIVE: i32 = 2400;
pub const SCRIPT_FUNCTION_GET_STATES_NUMBER: i32 = 2322;
pub const SCRIPT_FUNCTION_ADD_STATE: i32 = 2323;
pub const SCRIPT_FUNCTION_PLAYER_TALK: i32 = 2308;
pub const SCRIPT_FUNCTION_GET_NAME: i32 = 2998;
pub const SCRIPT_FUNCTION_IS_CHARGED: i32 = 2516;
pub const SCRIPT_FUNCTION_SET_CHARGED: i32 = 2517;
pub const SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE: i32 = 2518;
pub const SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE: i32 = 2519;
pub const SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE: i32 = 2520;
pub const SCRIPT_FUNCTION_CHANGE_BODY_CHECK: i32 = 2521;
pub const SCRIPT_FUNCTION_CHECK_MODE: i32 = 2522;
pub const SCRIPT_FUNCTION_GET_PROGRESS: i32 = 2576;
pub const SCRIPT_FUNCTION_ADD_EX_STATE: i32 = 2550;
pub const SCRIPT_FUNCTION_DELETE_EX_STATE: i32 = 2551;
pub const SCRIPT_FUNCTION_GET_EX_STATE: i32 = 2552;
pub const SCRIPT_FUNCTION_ADD_EX_STATE_NEW: i32 = 2553;
pub const SCRIPT_FUNCTION_DELETE_EX_STATE_NEW: i32 = 2554;
pub const SCRIPT_FUNCTION_GET_EX_STATE_NEW: i32 = 2555;
pub const SCRIPT_FUNCTION_ADD_UNDEAD_STATE: i32 = 2556;
pub const SCRIPT_FUNCTION_DELETE_UNDEAD_STATE: i32 = 2557;
pub const SCRIPT_FUNCTION_GET_UNDEAD_STATE: i32 = 2558;
pub const SCRIPT_FUNCTION_SET_HOTKEY: i32 = 2560;
pub const SCRIPT_FUNCTION_ADD_LOG: i32 = 2571;
pub const SCRIPT_FUNCTION_IS_COMBAT_STATE: i32 = 2574;
pub const SCRIPT_FUNCTION_DRAW_AWARDS: i32 = 2575;
pub const SCRIPT_FUNCTION_CHANGE_PLAYER: i32 = 2999;
pub const SCRIPT_FUNCTION_SET_PLAYER: i32 = 3000;
pub const SCRIPT_FUNCTION_GET_PLAYER: i32 = 3001;
pub const SCRIPT_FUNCTION_SET_PLAYER_LEVEL: i32 = 3002;
pub const SCRIPT_FUNCTION_IS_PLAYER_ONLINE: i32 = 3003;
pub const SCRIPT_FUNCTION_GET_PLAYER_ID: i32 = 3004;
pub const SCRIPT_FUNCTION_GET_REGION_ID: i32 = 3005;
pub const SCRIPT_FUNCTION_SET_PLAYER_REGION: i32 = 3006;
pub const SCRIPT_FUNCTION_SET_PLAYER_REGION_EX: i32 = 3007;
pub const SCRIPT_FUNCTION_KICK_PLAYER_EX: i32 = 3008;
pub const SCRIPT_FUNCTION_GET_PLAYER_ALL_PROPERTIES: i32 = 3009;
pub const SCRIPT_FUNCTION_FORCE_MOVE: i32 = 3010;
pub const SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME: i32 = 3011;
pub const SCRIPT_FUNCTION_GET_MONEY_BY_NAME: i32 = 3012;
pub const SCRIPT_FUNCTION_SET_MONEY_BY_NAME: i32 = 3013;
pub const SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID: i32 = 3014;
pub const SCRIPT_FUNCTION_GET_MONEY_BY_ID: i32 = 3015;
pub const SCRIPT_FUNCTION_SET_MONEY_BY_ID: i32 = 3016;
pub const SCRIPT_FUNCTION_GET_PLAYER_ALL_VARIABLES: i32 = 3017;
pub const SCRIPT_FUNCTION_DELETE_SKILL: i32 = 3102;
pub const SCRIPT_FUNCTION_SET_SKILL_LEVEL: i32 = 3103;
pub const SCRIPT_FUNCTION_ADD_SKILL: i32 = 3101;
pub const SCRIPT_FUNCTION_GET_SKILL_LEVEL: i32 = 3104;
pub const SCRIPT_FUNCTION_KICK_PLAYER: i32 = 3201;
pub const SCRIPT_FUNCTION_BAN_PLAYER: i32 = 3202;
pub const SCRIPT_FUNCTION_SILENCE_PLAYER: i32 = 3203;
pub const SCRIPT_FUNCTION_CREATE_FACTION: i32 = 6001;
pub const SCRIPT_FUNCTION_APPLY_JOIN_FACTION: i32 = 6002;
pub const SCRIPT_FUNCTION_QUIT_JOIN_FACTION: i32 = 6003;
pub const SCRIPT_FUNCTION_OPERATOR_CITY_GATE: i32 = 6004;
pub const SCRIPT_FUNCTION_OBTAIN_TAX_PAYMENT: i32 = 6005;
pub const SCRIPT_FUNCTION_ADJUST_TAX_RATE: i32 = 6006;
pub const SCRIPT_FUNCTION_TURN_ON_LEAVE_WORD: i32 = 6007;
pub const SCRIPT_FUNCTION_BUY_FACTION_LOGO: i32 = 6008;
pub const SCRIPT_FUNCTION_ASK_ATTACK_CITY_TIME: i32 = 6009;
pub const SCRIPT_FUNCTION_GET_FACTION_BILLBOARD: i32 = 6010;
pub const SCRIPT_FUNCTION_UPGRADE_FACTION: i32 = 6011;
pub const SCRIPT_FUNCTION_UPLOAD_FACTION_ICON: i32 = 6012;
pub const SCRIPT_FUNCTION_GET_FACTION_LEVEL_BY_PLAYER_ID: i32 = 6013;
pub const SCRIPT_FUNCTION_GET_FACTION_EXP_BY_PLAYER_ID: i32 = 6014;
pub const SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME: i32 = 6015;
pub const SCRIPT_FUNCTION_GET_UNION_ID_BY_PLAYER_NAME: i32 = 6016;
pub const SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME: i32 = 6017;
pub const SCRIPT_FUNCTION_IS_UNION_MASTER_BY_PLAYER_NAME: i32 = 6018;
pub const SCRIPT_FUNCTION_GET_CITY_GATE_STATE: i32 = 6019;
pub const SCRIPT_FUNCTION_OPERATE_CITY_GATE: i32 = 6020;
pub const SCRIPT_FUNCTION_FACTION_DECLARE_WAR: i32 = 6030;
pub const SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME: i32 = 6040;
pub const SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME: i32 = 6041;
pub const SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR: i32 = 6042;
pub const SCRIPT_FUNCTION_ENTER_CONTEND_STATE: i32 = 6043;
pub const SCRIPT_FUNCTION_CITY_WAR_DECLARE: i32 = 6044;
pub const SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME: i32 = 6045;
pub const SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME: i32 = 6046;
pub const SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID: i32 = 6047;
pub const SCRIPT_FUNCTION_GET_OWNED_REGION_UNION_ID: i32 = 6048;
pub const SCRIPT_FUNCTION_GET_TOTAL_TAX_PAYMENT: i32 = 6049;
pub const SCRIPT_FUNCTION_GET_TODAY_TAX_PAYMENT: i32 = 6050;
pub const SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS: i32 = 6051;
pub const SCRIPT_FUNCTION_SET_TOTAL_TAX_PAYMENT: i32 = 6052;
pub const SCRIPT_FUNCTION_SET_TODAY_TAX_PAYMENT: i32 = 6053;
pub const SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK: i32 = 2625;
pub const SCRIPT_FUNCTION_GET_WEEKS_HONOR_RANK: i32 = 2626;
pub const SCRIPT_FUNCTION_GET_MONTHS_HONOR_RANK: i32 = 2627;
pub const SCRIPT_FUNCTION_GET_TOTAL_HONOR_RANK: i32 = 2628;
pub const SCRIPT_FUNCTION_GET_ATTEMPT_APPELLATION_ID: i32 = 2630;
pub const SCRIPT_FUNCTION_GET_PLAYER_RANK: i32 = 2631;
pub const SCRIPT_FUNCTION_DELETE_EX_STATE_BY_TYPE: i32 = 2632;
pub const SCRIPT_FUNCTION_SEND_TOTAL_HONOR_RANKS: i32 = 2634;
pub const SCRIPT_FUNCTION_ADD_APPELLATION_STATE: i32 = 2635;
pub const SCRIPT_FUNCTION_DEL_APPELLATION_STATE: i32 = 2636;
pub const SCRIPT_FUNCTION_GET_APPELLATION_STATE: i32 = 2637;
pub const SCRIPT_FUNCTION_SET_THING_COUNT: i32 = 2650;
pub const SCRIPT_FUNCTION_GET_THING_COUNT: i32 = 2651;
pub const SCRIPT_FUNCTION_SET_JING_LI_DAN: i32 = 2652;
pub const SCRIPT_FUNCTION_GET_JING_LI_DAN: i32 = 2653;
pub const SCRIPT_FUNCTION_GET_WAR_REGION_STATE: i32 = 6054;
pub const SCRIPT_FUNCTION_GET_COUNTRY_OWNING_REGION: i32 = 6055;
pub const SCRIPT_FUNCTION_GET_WAR_START_TIME: i32 = 6056;
pub const SCRIPT_FUNCTION_IS_GOODS_WAR_MEMBER: i32 = 6057;
pub const SCRIPT_FUNCTION_DELETE_GOODS_WAR_MEMBER: i32 = 6058;
pub const SCRIPT_FUNCTION_GOODS_WAR_WIN: i32 = 6059;
pub const SCRIPT_FUNCTION_ASK_GOODS_WAR_LIST: i32 = 6060;
pub const SCRIPT_FUNCTION_IS_PLAYER_IN_WAR_FACTION: i32 = 6061;
pub const SCRIPT_FUNCTION_APPLY_FOR_GOODS_WAR: i32 = 6062;
pub const SCRIPT_FUNCTION_GOODS_WAR_WIN_COUNT: i32 = 6063;
pub const SCRIPT_FUNCTION_CHANGE_REGION: i32 = 2304;
pub const SCRIPT_FUNCTION_ADD_GOODS: i32 = 2200;
pub const SCRIPT_FUNCTION_DELETE_GOODS: i32 = 2201;
pub const SCRIPT_FUNCTION_CHECK_GOODS: i32 = 2202;
pub const SCRIPT_FUNCTION_CHECK_SPACE: i32 = 2203;
pub const SCRIPT_FUNCTION_GET_GOODS_NUMBER: i32 = 2204;
pub const SCRIPT_FUNCTION_GET_FREE_SPACE: i32 = 2205;
pub const SCRIPT_FUNCTION_UPGRADE_SELECTED_EQUIPMENT: i32 = 2209;
pub const SCRIPT_FUNCTION_ADD_DEPOT_GOODS: i32 = 2210;
pub const SCRIPT_FUNCTION_DELETE_DEPOT_GOODS: i32 = 2211;
pub const SCRIPT_FUNCTION_CHECK_DEPOT_GOODS: i32 = 2212;
pub const SCRIPT_FUNCTION_CHECK_DEPOT_SPACE: i32 = 2213;
pub const SCRIPT_FUNCTION_GET_DEPOT_GOODS_NUMBER: i32 = 2214;
pub const SCRIPT_FUNCTION_GET_DEPOT_GOODS_FREE: i32 = 2215;
pub const SCRIPT_FUNCTION_DELETE_PLAYER_GOODS: i32 = 2217;
pub const SCRIPT_FUNCTION_GET_CONTAINER_ITEM_TYPE: i32 = 2218;
pub const SCRIPT_FUNCTION_OPEN_GOODS_CONTAINER: i32 = 2220;
pub const SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1: i32 = 2223;
pub const SCRIPT_FUNCTION_GET_GOODS_PROPERTY_2: i32 = 2224;
pub const SCRIPT_FUNCTION_DELETE_SPLIT_GOODS: i32 = 2225;
pub const SCRIPT_FUNCTION_GET_GOODS_ORIGINAL_NAME: i32 = 2226;
pub const SCRIPT_FUNCTION_GET_GOODS_PRICE: i32 = 2227;
pub const SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1: i32 = 2228;
pub const SCRIPT_FUNCTION_SET_GOODS_PROPERTY_2: i32 = 2229;
pub const SCRIPT_FUNCTION_RECREATE_GOODS_ADDON_PROPERTIES: i32 = 2230;
pub const SCRIPT_FUNCTION_GET_DIED_MONSTER_INDEX: i32 = 2240;
pub const SCRIPT_FUNCTION_GET_DIED_MONSTER_ORIGINAL_NAME: i32 = 2241;
pub const SCRIPT_FUNCTION_GET_DIED_MONSTER_LEVEL: i32 = 2242;
pub const SCRIPT_FUNCTION_OPEN_NPC_SHOP: i32 = 2300;
pub const SCRIPT_FUNCTION_OPEN_DEPOT: i32 = 2301;
pub const SCRIPT_FUNCTION_GET_TEAM_NUM: i32 = 2302;
pub const SCRIPT_FUNCTION_GET_TEAMER_NAME: i32 = 2303;
pub const SCRIPT_FUNCTION_ADD_INFO: i32 = 2305;
pub const SCRIPT_FUNCTION_GAME_MESSAGE: i32 = 2306;
pub const SCRIPT_FUNCTION_TALK_BOX: i32 = 2307;
pub const SCRIPT_FUNCTION_HELP: i32 = 2309;
pub const SCRIPT_FUNCTION_TALK_BOX_SMALL: i32 = 2324;
pub const SCRIPT_FUNCTION_ADD_GOODS_LOG: i32 = 2313;
pub const SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG: i32 = 2314;
pub const SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG: i32 = 2315;
pub const SCRIPT_FUNCTION_OPEN_NEW_HELP_WINDOW: i32 = 2319;
pub const SCRIPT_FUNCTION_GET_GOODS_PROPERTY: i32 = 2321;
pub const SCRIPT_FUNCTION_SET_REGION_FOR_TEAM: i32 = 2310;
pub const SCRIPT_FUNCTION_SET_TEAM_REGION: i32 = 2311;
pub const SCRIPT_FUNCTION_IS_TEAMMATES_AROUND_ME: i32 = 2312;
pub const SCRIPT_FUNCTION_SCRIPT_IS_RUNNING: i32 = 2316;
pub const SCRIPT_FUNCTION_REMOVE_SCRIPT: i32 = 2317;
pub const SCRIPT_FUNCTION_ADD_FU_MO_PROPERTY: i32 = 2320;
pub const SCRIPT_FUNCTION_IS_TEAM_CAPTAIN: i32 = 2325;
pub const SCRIPT_FUNCTION_GET_COUNTRY: i32 = 2500;
pub const SCRIPT_FUNCTION_CHANGE_COUNTRY: i32 = 2501;
pub const SCRIPT_FUNCTION_GET_CONTRIBUTION: i32 = 2502;
pub const SCRIPT_FUNCTION_SET_CONTRIBUTION: i32 = 2503;
pub const SCRIPT_FUNCTION_GET_YUAN_BAO: i32 = 2504;
pub const SCRIPT_FUNCTION_ADD_INCREMENT_LOG: i32 = 2570;
pub const SCRIPT_FUNCTION_LIST_ONLINE_GM: i32 = 5103;
pub const SCRIPT_FUNCTION_LIST_SILENCE_PLAYER: i32 = 5104;
pub const SCRIPT_FUNCTION_SAVE_ALL_PLAYERS: i32 = 5105;
pub const SCRIPT_FUNCTION_GET_ONLINE_PLAYERS: i32 = 5108;
pub const SCRIPT_FUNCTION_LIST_ONLINE_PLAYER: i32 = 5109;
pub const SCRIPT_FUNCTION_KICK_ALL: i32 = 5301;
pub const SCRIPT_FUNCTION_KICK_MAP: i32 = 5302;
pub const SCRIPT_FUNCTION_GET_COPY_NUMBER: i32 = 9314;
pub const SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE: i32 = 5411;
pub const SCRIPT_FUNCTION_GET_MAXIMUM_LEVEL: i32 = 5412;
pub const SCRIPT_FUNCTION_GET_AREA_ID: i32 = 5413;
pub const SCRIPT_FUNCTION_GET_AREA_TYPE: i32 = 5414;
pub const SCRIPT_FUNCTION_GET_WORLD_SERVER_ID: i32 = 5420;
pub const SCRIPT_FUNCTION_PLAY_EFFECT: i32 = 5404;
pub const SCRIPT_FUNCTION_WEATHER: i32 = 5403;
pub const SCRIPT_FUNCTION_PLAY_ACTION: i32 = 5405;
pub const SCRIPT_FUNCTION_INVISIBLE: i32 = 5401;
pub const SCRIPT_FUNCTION_GOD_MODE: i32 = 5402;
pub const SCRIPT_FUNCTION_RESIDENT_MODE: i32 = 5406;
pub const SCRIPT_FUNCTION_GM_MODE: i32 = 5407;
pub const SCRIPT_FUNCTION_PLAY_SOUND: i32 = 5410;
pub const SCRIPT_FUNCTION_RELOAD: i32 = 5001;
pub const SCRIPT_FUNCTION_POST_PLAYER_INFO: i32 = 3316;
pub const SCRIPT_FUNCTION_IS_RIDER: i32 = 3317;
pub const SCRIPT_FUNCTION_DROP_GOODS: i32 = 3401;
pub const SCRIPT_FUNCTION_AUTO_MOVE: i32 = 3402;
pub const SCRIPT_FUNCTION_POST_REGION_INFO: i32 = 5201;
pub const SCRIPT_FUNCTION_POST_WORLD_INFO: i32 = 5202;
pub const SCRIPT_FUNCTION_POST_COUNTRY_INFO: i32 = 5203;
pub const SCRIPT_FUNCTION_ADD_QUEST: i32 = 6200;
pub const SCRIPT_FUNCTION_COMPLETE_QUEST: i32 = 6201;
pub const SCRIPT_FUNCTION_DISBAND_QUEST: i32 = 6202;
pub const SCRIPT_FUNCTION_GET_QUEST_STATE: i32 = 6203;
pub const SCRIPT_FUNCTION_ADD_QUEST_FOR_TEAM: i32 = 6204;
pub const SCRIPT_FUNCTION_RUN_SCRIPT_FOR_TEAM: i32 = 6205;
pub const SCRIPT_FUNCTION_GET_VALID_QUEST_NUM: i32 = 6206;
pub const SCRIPT_FUNCTION_UPDATE_QUEST_POSITION: i32 = 6207;
pub const SCRIPT_FUNCTION_SET_FACTION_PARAMETER_BY_PLAYER: i32 = 7000;
pub const SCRIPT_FUNCTION_CHANGE_FACTION_PARAMETER_BY_PLAYER: i32 = 7001;
pub const SCRIPT_FUNCTION_NPC_TALK: i32 = 3301;
pub const SCRIPT_FUNCTION_CREATE_NPC: i32 = 3302;
pub const SCRIPT_FUNCTION_DELETE_NPC: i32 = 3303;
pub const SCRIPT_FUNCTION_MONSTER_TALK: i32 = 3304;
pub const SCRIPT_FUNCTION_CREATE_MONSTER: i32 = 3305;
pub const SCRIPT_FUNCTION_DELETE_MONSTER: i32 = 3306;
pub const SCRIPT_FUNCTION_KILL_MONSTER: i32 = 3307;
pub const SCRIPT_FUNCTION_PLAYER_MESSAGE: i32 = 3308;
pub const SCRIPT_FUNCTION_GET_MAP_INFO: i32 = 3309;
pub const SCRIPT_FUNCTION_OPEN_PLAYER_UI: i32 = 3310;
pub const SCRIPT_FUNCTION_CALL_MONSTER: i32 = 3311;
pub const SCRIPT_FUNCTION_ATTACK_PLAYER: i32 = 3312;
pub const SCRIPT_FUNCTION_DELETE_MONSTER_RECT: i32 = 3313;
pub const SCRIPT_FUNCTION_MOVE_PLAYER: i32 = 3314;
pub const SCRIPT_FUNCTION_DELETE_NPC_BY_NAME: i32 = 3315;
pub const SCRIPT_FUNCTION_REFRESH_BLOCK: i32 = 8000;
pub const SCRIPT_FUNCTION_REFRESH_ALL_BLOCKS: i32 = 8001;
pub const SCRIPT_FUNCTION_SET_BLOCK: i32 = 8002;
pub const SCRIPT_FUNCTION_GET_REGION_RANDOM_POSITION: i32 = 8003;
pub const SCRIPT_FUNCTION_REFRESH_BLOCK_AT: i32 = 8004;
pub const SCRIPT_FUNCTION_OPEN_CHANGE_PLAYER_NAME: i32 = 8100;
pub const SCRIPT_FUNCTION_GET_MONSTER_REFRESH_TIME: i32 = 8101;
pub const SCRIPT_FUNCTION_GET_REGION_PLAYER_COUNT: i32 = 8102;
pub const SCRIPT_FUNCTION_IS_QUEST_ENABLED: i32 = 3500;
pub const SCRIPT_FUNCTION_SET_QUEST_ENABLED: i32 = 3501;
pub const SCRIPT_FUNCTION_QUEST_TIME_BEGIN: i32 = 3502;
pub const SCRIPT_FUNCTION_QUEST_TIME_CLEAR: i32 = 3503;
pub const SCRIPT_FUNCTION_ADD_CARRIAGE: i32 = 3504;
pub const SCRIPT_FUNCTION_DELETE_CARRIAGE: i32 = 3505;
pub const SCRIPT_FUNCTION_GET_CARRIAGE_DISTANCE: i32 = 3506;
pub const SCRIPT_FUNCTION_GET_QUEST_TIME: i32 = 3507;
pub const SCRIPT_FUNCTION_GET_CARRIAGE_INDEX: i32 = 3508;
pub const SCRIPT_FUNCTION_OPEN_SYNTHESIS: i32 = 3510;
pub const SCRIPT_FUNCTION_SET_COUNTRY_POWER: i32 = 9001;
pub const SCRIPT_FUNCTION_GET_COUNTRY_POWER: i32 = 9000;
pub const SCRIPT_FUNCTION_GET_COUNTRY_TECH_LEVEL: i32 = 9002;
pub const SCRIPT_FUNCTION_SET_COUNTRY_TECH_LEVEL: i32 = 9003;
pub const SCRIPT_FUNCTION_SET_COUNTRY_TREASURY: i32 = 9009;
pub const SCRIPT_FUNCTION_SET_COUNTRY_MATERIAL: i32 = 9011;
pub const SCRIPT_FUNCTION_SET_COUNTRY_TECH: i32 = 9013;
pub const SCRIPT_FUNCTION_UPGRADE_COUNTRY_TECH_LEVEL: i32 = 9014;
pub const SCRIPT_FUNCTION_GET_COUNTRY_CI: i32 = 9004;
pub const SCRIPT_FUNCTION_SET_COUNTRY_CI: i32 = 9005;
pub const SCRIPT_FUNCTION_GET_COUNTRY_KING_ID: i32 = 9006;
pub const SCRIPT_FUNCTION_GET_COUNTRY_TREASURY: i32 = 9008;
pub const SCRIPT_FUNCTION_GET_COUNTRY_MATERIAL: i32 = 9010;
pub const SCRIPT_FUNCTION_GET_COUNTRY_TECH: i32 = 9012;
pub const SCRIPT_FUNCTION_GET_COUNTRY_OCCUPATION: i32 = 2633;
pub const SCRIPT_FUNCTION_GET_COUNTRY_IDENTITY: i32 = 9020;
pub const SCRIPT_FUNCTION_GET_QUEST_SWITCH: i32 = 9018;
pub const SCRIPT_FUNCTION_SET_QUEST_SWITCH: i32 = 9019;
pub const SCRIPT_FUNCTION_EXILE_TIME: i32 = 9021;
pub const SCRIPT_FUNCTION_SET_NEW_COUNTRY_DAY: i32 = 9200;
pub const SCRIPT_FUNCTION_IS_HOMELAND: i32 = 9301;
pub const SCRIPT_FUNCTION_GET_REGION_COUNTRY: i32 = 9302;
pub const SCRIPT_FUNCTION_IS_REGIONAL_PROTECTED: i32 = 9303;
pub const SCRIPT_FUNCTION_ADD_KING_POINT: i32 = 9317;
pub const SCRIPT_FUNCTION_GET_PLAYER_GODS_BATTLE_FACTION: i32 = 11121;
pub const SCRIPT_FUNCTION_GET_GODS_BATTLE_NPC_FACTION: i32 = 11122;
pub const SCRIPT_FUNCTION_IS_GODS_BATTLE_NPC_PLAYER_FACTION: i32 = 11123;
pub const SCRIPT_FUNCTION_GET_PLAYER_SZL: i32 = 11124;
pub const SCRIPT_FUNCTION_SET_GODS_BATTLE_FACTION_XYD: i32 = 11125;
pub const SCRIPT_FUNCTION_GET_GODS_BATTLE_FACTION_XYD: i32 = 11126;
pub const SCRIPT_FUNCTION_SET_PLAYER_GODS_BATTLE_FACTION: i32 = 11127;
pub const SCRIPT_FUNCTION_CHANGE_PLAYER_SZL: i32 = 11128;
pub const SCRIPT_FUNCTION_GET_GODS_BATTLE_TOP_TEN: i32 = 11129;
pub const SCRIPT_FUNCTION_ENTER_GODS_BATTLE_CONTEND: i32 = 11130;
pub const SCRIPT_FUNCTION_DECLARE_COUNTRY_WAR: i32 = 9100;
pub const SCRIPT_FUNCTION_IS_COUNTRY_WAR_DECLARE: i32 = 9101;
pub const SCRIPT_FUNCTION_IS_COUNTRY_DECLARED: i32 = 9102;
pub const SCRIPT_FUNCTION_IS_COUNTRY_WAR_PREPARE: i32 = 9103;
pub const SCRIPT_FUNCTION_IS_COUNTRY_WAR: i32 = 9104;
pub const SCRIPT_FUNCTION_ENTER_COUNTRY_CONTEND: i32 = 9105;
pub const SCRIPT_FUNCTION_IS_COUNTRY_WIN_SYMBOL: i32 = 9106;
pub const SCRIPT_FUNCTION_GET_COUNTRY_WAR_REGION: i32 = 9107;
pub const SCRIPT_FUNCTION_GET_COUNTRY_WAR_CAMP: i32 = 9108;
pub const SCRIPT_FUNCTION_GET_OTHER_WAR_COUNTRY: i32 = 9109;
pub const SCRIPT_FUNCTION_COUNTRY_WAR_VICTORY: i32 = 9110;
pub const SCRIPT_FUNCTION_GET_COUNTRY_WAR_RESULT: i32 = 9111;
pub const SCRIPT_FUNCTION_NATION_WAR_SEND_PLAYER_ID: i32 = 9304;
pub const SCRIPT_FUNCTION_NATION_WAR_CARRIAGE_BACK_TOWN: i32 = 9305;
pub const SCRIPT_FUNCTION_NATION_WAR_GET_FLAG_STATUS: i32 = 9306;
pub const SCRIPT_FUNCTION_NATION_WAR_GET_TIME: i32 = 9307;
pub const SCRIPT_FUNCTION_NATION_WAR_ENTER_CONTEND: i32 = 9308;
pub const SCRIPT_FUNCTION_NATION_WAR_COUNTRY_SIGN_UP: i32 = 9309;
pub const SCRIPT_FUNCTION_NATION_WAR_CLEAR_PLAYER_TIME: i32 = 9310;
pub const SCRIPT_FUNCTION_NATION_WAR_GET_NATION_STATUS: i32 = 9311;
pub const SCRIPT_FUNCTION_NATION_WAR_SET_PLAYER_TIME: i32 = 9312;
pub const SCRIPT_FUNCTION_NATION_WAR_IS_PLAYER_WEAK: i32 = 9313;
pub const SCRIPT_FUNCTION_NATION_WAR_GET_MORALE: i32 = 9315;
pub const SCRIPT_FUNCTION_NATION_WAR_CLEAR_MORALE: i32 = 9316;
pub const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL: i32 = 9400;
pub const SCRIPT_FUNCTION_GET_FETCH_POWER: i32 = 9401;
pub const SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9402;
pub const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL: i32 = 9403;
pub const SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL: i32 = 9404;
pub const SCRIPT_FUNCTION_DELETE_BATTLE_FAIRY_SKILL: i32 = 9405;
pub const SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY: i32 = 9406;
pub const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID: i32 = 9407;
pub const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL: i32 = 9408;
pub const SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE: i32 = 9409;
pub const SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE: i32 = 9410;
pub const SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES: i32 = 9411;
pub const SCRIPT_INT_PARAMETER_ERROR: i32 = 0x09ff_fff9;
pub const SCRIPT_FUNCTION_ARGUMENT_CAPACITY: usize = 32;
pub const MAXIMUM_SCRIPT_SPAWN_COUNT: i32 = 4096;
pub const SCRIPT_PLAYER_TYPE: i32 = 400;
pub const SCRIPT_NPC_TYPE: i32 = 500;

// Идентификатор buffskill-диспетчера исторического `CScript`; владение на
// стороне старого пакета ещё живёт в `appserver/script/buffskillfunc.rs`.
pub const SCRIPT_FUNCTION_ADD_JING_JIE_BUFF: i32 = 11131;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptFunctionParameterKind {
    Integer,
    String,
    Unused,
}

/// Маршрутизация `GetStringParam`/`GetIntParam` исторического плотного
/// диспетчера. `CScript` использует таблицу до вычисления выражения и сохраняет
/// позиционные строковые аргументы вместо прежнего особого случая только для
/// `9351`.
pub fn script_function_parameter_kind(
    function_id: i32,
    index: usize,
) -> ScriptFunctionParameterKind {
    use ScriptFunctionParameterKind::{Integer, String, Unused};
    match function_id {
        SCRIPT_FUNCTION_GET_STRING_BY_ID | SCRIPT_FUNCTION_GET_ME => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_ME | SCRIPT_FUNCTION_SET_ME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_ENERGY | SCRIPT_FUNCTION_SET_MAXIMUM_ENERGY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHECK_LEVEL
        | SCRIPT_FUNCTION_GET_ENERGY
        | SCRIPT_FUNCTION_GET_MAXIMUM_ENERGY => Unused,
        SCRIPT_FUNCTION_OPEN_NPC_SHOP => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_DEPOT => Unused,
        SCRIPT_FUNCTION_GET_NAME | SCRIPT_FUNCTION_GET_TEAMER_NAME => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_TEAM_NUM | SCRIPT_FUNCTION_IS_TEAM_CAPTAIN => Unused,
        SCRIPT_FUNCTION_IS_TEAMMATES_AROUND_ME => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_REGION_FOR_TEAM | SCRIPT_FUNCTION_SET_TEAM_REGION => match index {
            0..=5 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_FU_MO_PROPERTY => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAYER_TALK => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RE_LIVE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_STATES_NUMBER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_STATE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_LEVEL => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_PLAYER | SCRIPT_FUNCTION_SET_PLAYER => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_PLAYER_ONLINE | SCRIPT_FUNCTION_GET_PLAYER_ID => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_REGION_ID => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_REGION => match index {
            0 => String,
            1..=5 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_REGION_EX => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_KICK_PLAYER_EX => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER_ALL_PROPERTIES => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_FORCE_MOVE => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_NAME | SCRIPT_FUNCTION_SET_MONEY_BY_NAME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONEY_BY_NAME
        | SCRIPT_FUNCTION_GET_FACTION_ID_BY_PLAYER_NAME
        | SCRIPT_FUNCTION_GET_UNION_ID_BY_PLAYER_NAME
        | SCRIPT_FUNCTION_IS_FACTION_MASTER_BY_PLAYER_NAME
        | SCRIPT_FUNCTION_IS_UNION_MASTER_BY_PLAYER_NAME => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_MONEY_BY_ID | SCRIPT_FUNCTION_SET_MONEY_BY_ID => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONEY_BY_ID => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PLAYER_ALL_VARIABLES => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_KICK_PLAYER => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_BAN_PLAYER | SCRIPT_FUNCTION_SILENCE_PLAYER => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CREATE_FACTION => match index {
            0 | 2 | 3 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_APPLY_JOIN_FACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FACTION_BILLBOARD => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPERATOR_CITY_GATE | SCRIPT_FUNCTION_OPERATE_CITY_GATE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_APPLY_TIME
        | SCRIPT_FUNCTION_IS_ARRIVE_VILLAGE_WAR_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_DECLARE_TIME
        | SCRIPT_FUNCTION_IS_CITY_WAR_FIGHT_TIME
        | SCRIPT_FUNCTION_GET_OWNED_REGION_FACTION_ID
        | SCRIPT_FUNCTION_GET_OWNED_REGION_UNION_ID
        | SCRIPT_FUNCTION_GET_TOTAL_TAX_PAYMENT
        | SCRIPT_FUNCTION_GET_TODAY_TAX_PAYMENT
        | SCRIPT_FUNCTION_GET_WAR_REGION_STATE
        | SCRIPT_FUNCTION_GET_COUNTRY_OWNING_REGION
        | SCRIPT_FUNCTION_GET_WAR_START_TIME => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_APPLY_FOR_VILLAGE_WAR | SCRIPT_FUNCTION_CITY_WAR_DECLARE => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_TOTAL_TAX_PAYMENT | SCRIPT_FUNCTION_SET_TODAY_TAX_PAYMENT => {
            match index {
                0..=1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_ENTER_CONTEND_STATE => match index {
            0 | 1 => Integer,
            2..=5 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_CITY_GATE_STATE => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_GOODS_WAR_MEMBER
        | SCRIPT_FUNCTION_DELETE_GOODS_WAR_MEMBER
        | SCRIPT_FUNCTION_IS_PLAYER_IN_WAR_FACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_SKILL => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_SKILL_LEVEL => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_SKILL => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_SKILL_LEVEL => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_REGION => match index {
            0..=6 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SCRIPT_IS_RUNNING | SCRIPT_FUNCTION_REMOVE_SCRIPT => match index {
            0 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_YEAR
        | SCRIPT_FUNCTION_MONTH
        | SCRIPT_FUNCTION_DAY
        | SCRIPT_FUNCTION_HOUR
        | SCRIPT_FUNCTION_MINUTE
        | SCRIPT_FUNCTION_DAY_OF_WEEK => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_HOUR_DIFF | SCRIPT_FUNCTION_MINUTE_DIFF => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_REGION_RANDOM_POSITION
        | SCRIPT_FUNCTION_REFRESH_BLOCK
        | SCRIPT_FUNCTION_GET_REGION_PLAYER_COUNT => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_BLOCK => match index {
            0..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REFRESH_BLOCK_AT => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_CHARGED => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_DELETE_CHANGE_BODY_STATE
        | SCRIPT_FUNCTION_GET_CHANGE_BODY_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_EX_STATE
        | SCRIPT_FUNCTION_DELETE_EX_STATE
        | SCRIPT_FUNCTION_GET_EX_STATE
        | SCRIPT_FUNCTION_ADD_EX_STATE_NEW
        | SCRIPT_FUNCTION_DELETE_EX_STATE_NEW
        | SCRIPT_FUNCTION_GET_EX_STATE_NEW
        | SCRIPT_FUNCTION_ADD_UNDEAD_STATE
        | SCRIPT_FUNCTION_DELETE_UNDEAD_STATE
        | SCRIPT_FUNCTION_GET_UNDEAD_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_HOTKEY => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_WALK_STEP
        | SCRIPT_FUNCTION_RUN_STEP
        | SCRIPT_FUNCTION_SET_PLAYER_DIRECTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_POSITION => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_EQUIPMENT_ID_BY_POSITION => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_UPGRADE_EQUIPMENT => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_CHARGED
        | SCRIPT_FUNCTION_CHANGE_BODY_CHECK
        | SCRIPT_FUNCTION_CHECK_MODE
        | SCRIPT_FUNCTION_GET_PROGRESS
        | SCRIPT_FUNCTION_IS_COMBAT_STATE
        | SCRIPT_FUNCTION_IS_RIDER => Unused,
        SCRIPT_FUNCTION_OPEN_PLAYER_UI | SCRIPT_FUNCTION_CALL_MONSTER => Unused,
        SCRIPT_FUNCTION_CREATE_NPC => match index {
            0 | 8 => String,
            1..=7 | 9..=11 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_NPC_TALK | SCRIPT_FUNCTION_MONSTER_TALK => match index {
            0..=1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ATTACK_PLAYER => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_MOVE_PLAYER => match index {
            0..=9 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CREATE_MONSTER => match index {
            0 | 6 => String,
            1..=5 | 7 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_NPC
        | SCRIPT_FUNCTION_DELETE_MONSTER
        | SCRIPT_FUNCTION_KILL_MONSTER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAYER_MESSAGE => match index {
            0..=1 => String,
            2..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_MONSTER_RECT => match index {
            0..=4 => Integer,
            5 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DROP_GOODS => match index {
            0 | 1 | 3 => Integer,
            2 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_AUTO_MOVE => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_NPC_BY_NAME => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MAP_INFO => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_MONSTER_REFRESH_TIME => match index {
            0..=1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_COPY_NUMBER => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_LEVEL_EXPERIENCE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_TIME
        | SCRIPT_FUNCTION_SECOND
        | SCRIPT_FUNCTION_GET_COUNTRY
        | SCRIPT_FUNCTION_LIST_ONLINE_GM
        | SCRIPT_FUNCTION_LIST_SILENCE_PLAYER
        | SCRIPT_FUNCTION_SAVE_ALL_PLAYERS
        | SCRIPT_FUNCTION_GET_ONLINE_PLAYERS
        | SCRIPT_FUNCTION_LIST_ONLINE_PLAYER
        | SCRIPT_FUNCTION_KICK_ALL
        | SCRIPT_FUNCTION_GET_MAXIMUM_LEVEL
        | SCRIPT_FUNCTION_GET_AREA_ID
        | SCRIPT_FUNCTION_GET_AREA_TYPE
        | SCRIPT_FUNCTION_GET_WORLD_SERVER_ID
        | SCRIPT_FUNCTION_GET_GODS_BATTLE_NPC_FACTION
        | SCRIPT_FUNCTION_IS_GODS_BATTLE_NPC_PLAYER_FACTION
        | SCRIPT_FUNCTION_GET_PLAYER_SZL
        | SCRIPT_FUNCTION_OPEN_CHANGE_PLAYER_NAME => Unused,
        SCRIPT_FUNCTION_KICK_MAP => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_INVISIBLE
        | SCRIPT_FUNCTION_GOD_MODE
        | SCRIPT_FUNCTION_RESIDENT_MODE
        | SCRIPT_FUNCTION_GM_MODE => Unused,
        SCRIPT_FUNCTION_GET_PLAYER_GODS_BATTLE_FACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_GODS_BATTLE_FACTION_XYD => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GODS_BATTLE_FACTION_XYD => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_PLAYER_GODS_BATTLE_FACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHANGE_PLAYER_SZL => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GODS_BATTLE_TOP_TEN => Unused,
        SCRIPT_FUNCTION_ENTER_GODS_BATTLE_CONTEND => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAY_EFFECT => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_WEATHER | SCRIPT_FUNCTION_PLAY_ACTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_PLAY_SOUND => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_GOODS_FROM_CI_QING => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_CI_QING_PAGE => Unused,
        SCRIPT_FUNCTION_PUSH_ITEM_TO_CI_QING => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_EQUIPPED_GOODS_INDEX => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REQUEST_PLAYER_RANKS => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_DAYS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_WEEKS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_MONTHS_HONOR_RANK
        | SCRIPT_FUNCTION_GET_TOTAL_HONOR_RANK
        | SCRIPT_FUNCTION_GET_ATTEMPT_APPELLATION_ID
        | SCRIPT_FUNCTION_SEND_TOTAL_HONOR_RANKS
        | SCRIPT_FUNCTION_GET_PLAYER_RANK
        | SCRIPT_FUNCTION_GET_JING_LI_DAN => Unused,
        SCRIPT_FUNCTION_ADD_APPELLATION_STATE
        | SCRIPT_FUNCTION_DEL_APPELLATION_STATE
        | SCRIPT_FUNCTION_GET_APPELLATION_STATE => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_THING_COUNT => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_THING_COUNT => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_EX_STATE_BY_TYPE | SCRIPT_FUNCTION_SET_JING_LI_DAN => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_JING_JIE_BUFF => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RELOAD => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_POST_PLAYER_INFO
        | SCRIPT_FUNCTION_POST_REGION_INFO
        | SCRIPT_FUNCTION_POST_WORLD_INFO => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_POST_COUNTRY_INFO => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_INCREMENT_LOG => match index {
            0 | 3 => String,
            1 | 2 | 4 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_LOG => match index {
            0 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DRAW_AWARDS => match index {
            0..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_GOODS
        | SCRIPT_FUNCTION_DELETE_GOODS
        | SCRIPT_FUNCTION_CHECK_GOODS
        | SCRIPT_FUNCTION_ADD_INFO
        | SCRIPT_FUNCTION_TALK_BOX
        | SCRIPT_FUNCTION_HELP
        | SCRIPT_FUNCTION_TALK_BOX_SMALL
        | SCRIPT_FUNCTION_ADD_GOODS_LOG => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GAME_MESSAGE => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_GEM_EXCHANGE_LOG => match index {
            0..=1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_JEWELRY_MADE_LOG => match index {
            0..=2 => String,
            3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY => match index {
            0 => String,
            1..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RANDOM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RGB | SCRIPT_FUNCTION_CHECK_SPACE => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FREE_SPACE | SCRIPT_FUNCTION_UPGRADE_SELECTED_EQUIPMENT => {
            match index {
                0 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_ADD_DEPOT_GOODS | SCRIPT_FUNCTION_DELETE_DEPOT_GOODS => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_CHECK_DEPOT_GOODS | SCRIPT_FUNCTION_CHECK_DEPOT_SPACE => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_PLAYER_GOODS => match index {
            0..=1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_GOODS_CONTAINER => match index {
            0..=1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_GOODS_NUMBER
        | SCRIPT_FUNCTION_GET_DEPOT_GOODS_NUMBER
        | SCRIPT_FUNCTION_GET_DEPOT_GOODS_FREE
        | SCRIPT_FUNCTION_GET_CONTAINER_ITEM_TYPE => Unused,
        SCRIPT_FUNCTION_OPEN_NEW_HELP_WINDOW => Unused,
        SCRIPT_FUNCTION_CHANGE_COUNTRY | SCRIPT_FUNCTION_SET_CONTRIBUTION => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_CONTRIBUTION | SCRIPT_FUNCTION_GET_YUAN_BAO => Unused,
        SCRIPT_FUNCTION_GET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_GET_GOODS_PROPERTY_2 => {
            match index {
                0 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_SET_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_GOODS_PROPERTY_2 => {
            match index {
                0..=1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_DELETE_SPLIT_GOODS
        | SCRIPT_FUNCTION_GET_GOODS_ORIGINAL_NAME
        | SCRIPT_FUNCTION_GET_GOODS_PRICE
        | SCRIPT_FUNCTION_RECREATE_GOODS_ADDON_PROPERTIES
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_INDEX
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_ORIGINAL_NAME
        | SCRIPT_FUNCTION_GET_DIED_MONSTER_LEVEL => Unused,
        SCRIPT_FUNCTION_ADD_QUEST
        | SCRIPT_FUNCTION_COMPLETE_QUEST
        | SCRIPT_FUNCTION_DISBAND_QUEST
        | SCRIPT_FUNCTION_GET_QUEST_STATE => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_QUEST_FOR_TEAM => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RUN_SCRIPT_FOR_TEAM => match index {
            0 | 2 => Integer,
            1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_VALID_QUEST_NUM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_FACTION_PARAMETER_BY_PLAYER => match index {
            0 | 1 => String,
            2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_UPDATE_QUEST_POSITION => match index {
            0..=4 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_IS_QUEST_ENABLED
        | SCRIPT_FUNCTION_QUEST_TIME_CLEAR
        | SCRIPT_FUNCTION_GET_QUEST_TIME
        | SCRIPT_FUNCTION_DELETE_CARRIAGE
        | SCRIPT_FUNCTION_GET_CARRIAGE_DISTANCE
        | SCRIPT_FUNCTION_GET_CARRIAGE_INDEX
        | SCRIPT_FUNCTION_OPEN_SYNTHESIS => Unused,
        SCRIPT_FUNCTION_SET_QUEST_ENABLED | SCRIPT_FUNCTION_QUEST_TIME_BEGIN => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_CARRIAGE => match index {
            0 | 1 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_MODIFY_GOODS_TIME => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_MODIFY_AUCTION_SPACE => match index {
            0 | 1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_AUTO_ADD_AUCTION_GOODS => match index {
            0..=2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_TIME_GOODS => match index {
            0 => String,
            1..=31 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_DELETE_USED_GOODS
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_1
        | SCRIPT_FUNCTION_GET_USED_GOODS_PROPERTY_2
        | SCRIPT_FUNCTION_SET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_SET_SELECTED_DURABILITY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_1 | SCRIPT_FUNCTION_SET_USED_GOODS_PROPERTY_2 => {
            match index {
                0 | 1 => Integer,
                _ => Unused,
            }
        }
        SCRIPT_FUNCTION_CHECK_USED_GOODS
        | SCRIPT_FUNCTION_GET_CURRENT_DURABILITY
        | SCRIPT_FUNCTION_GET_SELECTED_DURABILITY => Unused,
        SCRIPT_FUNCTION_IS_HOMELAND | SCRIPT_FUNCTION_GET_REGION_COUNTRY => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_UPGRADE_COUNTRY_TECH_LEVEL
        | SCRIPT_FUNCTION_SET_NEW_COUNTRY_DAY
        | SCRIPT_FUNCTION_IS_REGIONAL_PROTECTED => Unused,
        SCRIPT_FUNCTION_FAIRY_EXP_UP => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_REFLUSH_EXTERN_PROPERTY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_OPEN_PRECIOUS_BOX => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_PRECIOUS_ITEM => match index {
            0 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_SKILL => match index {
            0 | 1 => String,
            2 | 3 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_GET_FETCH_POWER
        | SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SPECIAL_SKILL
        | SCRIPT_FUNCTION_REVIVE_BATTLE_FAIRY => match index {
            0 => String,
            _ => Unused,
        },
        SCRIPT_FUNCTION_SET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 | 2 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_ALLOCATE_BATTLE_FAIRY_SKILL
        | SCRIPT_FUNCTION_DELETE_BATTLE_FAIRY_SKILL
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_ID
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_SKILL_LEVEL
        | SCRIPT_FUNCTION_ADD_BATTLE_FAIRY_EXPERIENCE
        | SCRIPT_FUNCTION_GET_BATTLE_FAIRY_ATTRIBUTE => match index {
            0 => String,
            1 => Integer,
            _ => Unused,
        },
        SCRIPT_FUNCTION_RECREATE_BATTLE_FAIRY_ATTRIBUTES => match index {
            0 => String,
            1..=3 => Integer,
            _ => Unused,
        },
        _ if index < 3 => Integer,
        _ => Unused,
    }
}

/// Упакованный результат `time()` для сценарного селектора 13. Это не время
/// Unix: исходник хранит поля CRT `tm_year/tm_mon` и завершает значение тремя
/// битами дня недели.
pub fn pack_script_local_time(time: TagTime) -> i32 {
    i32::from(time.year)
        .wrapping_sub(1900)
        .wrapping_shl(4)
        .wrapping_add(i32::from(time.month).wrapping_sub(1))
        .wrapping_shl(5)
        .wrapping_add(i32::from(time.day))
        .wrapping_shl(5)
        .wrapping_add(i32::from(time.hour))
        .wrapping_shl(6)
        .wrapping_add(i32::from(time.minute))
        .wrapping_shl(3)
        .wrapping_add(i32::from(time.day_of_week))
}

pub fn script_packed_time_component(function_id: i32, packed: i32) -> i32 {
    match function_id {
        SCRIPT_FUNCTION_YEAR => (packed >> 23).wrapping_add(1900),
        SCRIPT_FUNCTION_MONTH => ((packed >> 19) & 0x0f).wrapping_add(1),
        SCRIPT_FUNCTION_DAY => (packed >> 14) & 0x1f,
        SCRIPT_FUNCTION_HOUR => (packed >> 9) & 0x1f,
        SCRIPT_FUNCTION_MINUTE => (packed >> 3) & 0x3f,
        SCRIPT_FUNCTION_DAY_OF_WEEK => packed & 0x07,
        _ => unreachable!("calendar component вызывается только для 14..19"),
    }
}
