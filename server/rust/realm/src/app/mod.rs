//! Композиция и сетевой край Realm: сообщения направлений, слушающие порты и
//! места диспетчеризации объединённого Realm-процесса по [карте владельцев].
//! Старого процессного `realmserver` ещё нет; компонент растёт по мере
//! переноса владельцев World/Auth/Login/Billing/Misc.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod auth_message;
pub mod auth_server;
pub mod auth_server_client;
pub mod billing_message;
pub mod billing_server;
pub mod billing_server_client;
pub mod countrymessage;
pub mod login_auth_client;
pub mod login_message;
pub mod login_server;
pub mod login_server_client;
pub mod login_world_server;
pub mod login_world_server_client;
pub mod misc_client;
pub mod misc_game;
pub mod misc_message;
pub mod miscservermessage;
pub mod onbillserver;
pub mod othermessage;
pub mod setup;
pub mod auction;
pub mod gmamessage;
pub mod gmmessage;
pub mod jjcsysmessage;
pub mod logmessage;
pub mod onmsg_m2w_auction;
pub mod organsysmessage;
pub mod playermessage;
pub mod player_base;
pub mod servermessage;
pub mod teammessage;
pub mod world_client;
pub mod world_game_view;
pub mod world_message;
pub mod world_network; // net-thread прокладка хода World: accept/I/O worker, опрос Login
pub mod world_organizing_view;
pub mod worldothermessage;
pub mod world_server;
pub mod world_server_client;
pub mod worldserver;
