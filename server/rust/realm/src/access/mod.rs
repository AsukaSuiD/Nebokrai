//! Клиент до мирового входа, ожидающие проверки аккаунта и варианты допуска Realm.

pub mod acclog; // typed-записи account-журнала LoginServer.
pub mod acclogqueue; // FIFO account-журнала и его semaphore-сигнализация.
pub mod acclogthread; // consumer account-журнала: SQL-шаблоны записи в БД.
pub mod asmessage; // обработчики направления Auth/GMA у LoginServer.
pub mod authhandler; // embedded AuthHandler: listener-слоты password-проверки.
pub mod authgame; // runtime CGame роли AuthServer: конфиг, DB-очереди и lifecycle.
pub mod authmanager; // pending Auth-запросы и синхронные listener-callbacks Login.
pub mod authproc; // Auth MSSQL-команды (do_auth/do_lock/do_write_log) и lifecycle DB workers.
pub mod configreader; // позиционная конфигурация AuthServer из setup.ini.
pub mod dbcontext; // очереди требований/результатов и server info AuthServer.
pub mod dbqueue; // типизированные DB-формы AuthServer вместо tag + void*.
pub mod game; // runtime CGame LoginServer: Auth, маршруты World/client, CD-key и GAS.
pub mod gasoperator; // получение IP для GAS-запросов.
pub mod gasthread; // worker GAS-FIFO с выделенным HTTP и таблицей кодов ответа.
pub mod gmmessage; // GM-ветви Login: CDKeyBan и блокировки CD-key.
pub mod kl_ipfilter; // IPv4 allow/deny фильтр с wildcard-октетом.
pub mod kl_multi_list; // потокобезопасная FIFO-очередь Mutex/Condvar.
pub mod loginqueue; // очереди допуска Login: CD-key/player/GAS, пароль, valid-code и matrix.
pub mod logmessage; // Client/World-ветви Login: valid-code, matrix и списки ролей.
pub mod message; // выбор единственного владельца сообщения Login (Auth/GMA, GM, Log, Server).
pub mod message_func; // обработчики AuthServer: Login-соединения, DB-очереди, GM и server-info.
pub mod mywininet; // HTTP-владелец GAS-запросов (замена WinInet).
pub mod networkconfig; // сетевые параметры роли AuthServer.
pub mod rscdkey; // DB-владелец CRsCDKey: баны, IP-фильтры, matrix-card и GAS-процедура.
pub mod servermessage; // World lifecycle, CD-key snapshots и telemetry у Login.
pub mod servlogqueue; // FIFO серверного журнала Login.
pub mod validcode; // генератор valid-code по ValidCode.ini.
