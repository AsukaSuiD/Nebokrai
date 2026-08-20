#pragma once

#include "acclogqueue.h"

#include <asio.hpp>

#include <array>
#include <cstdint>
#include <filesystem>
#include <list>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <vector>

namespace LoginNet
{
class CMessage;
class CMyNetClientAuth;
class CMyNetServerClient;
class CMyNetServerWorld;
}

namespace Login
{
class AuthManager;
class CLoginQueue;

/*
 * Исходный владелец: loginserver/game.cpp / game.h — доказанный фрагмент CGame.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Материализуются только уже доказанные поля и методы; полная 1024-байтная
 * компоновка из PDB не подменяется фиктивными полями и расширяется по мере
 * восстановления.
 *
 * Владелец настроек: конструктор tagSetup 0x0040C6E0,
 * ChangeAllWorldSate 0x004075E0,
 * LoadSetupEx 0x0040D9E0, ReLoadSetupEx 0x0040DC60,
 * LoadSetup 0x0040E690, ReLoadSetup 0x0040F4E0,
 * LoadWorldSetup 0x00410F30, SetListWorldInfoBySetup 0x00411170,
 * ReLoadWorldSetup 0x00411FF0, UpdateWorldInfoToAllClient 0x00407860,
 * load_listen_port 0x0040DCB0, InitNetServer_Client 0x00402DA0,
 * InitNetServer_World 0x00402F90, LoadASList 0x004100F0,
 * InitAuthClient 0x0040C9D0, ReconnectAS 0x00406240,
 * ReassignAS 0x00409BF0, Release 0x00405C60 и значения конструктора
 * CGame по умолчанию 0x00414350.
 *
 * Asio заменяет WinSock/IOCP только в технической части. Отдельные порты из
 * port.ini, порядок пересоздания владельцев, сетевые пределы и локальный IPv4
 * сохраняются. Этот owner освобождает принадлежащую ему сеть; общий порядок
 * остановки очередей, DB-владельцев и фонового журнала задаёт LoginServer runtime.
 * Исходящий Auth-клиент остаётся отдельным соединением с самостоятельным
 * AuthServer-процессом. Частичное чтение aslist.ini, безусловный успех
 * InitAuthClient и отложенная через FIFO замена клиента сохраняются явно.
 * Блок World/CD-key и маршруты ролей перенесены целиком: AddWorld 0x00412910,
 * DelWorld 0x00412A60, AddCdkey 0x0040F600, ClearCDKey 0x00411720 и сообщения
 * 0x4FB02..0x4FB05. Ошибочная очистка login-map из Linux-донора не перенесена:
 * подтверждённый ClearCDKey очищает её только когда account не найден в World.
 */
class CGame
{
public:
    CGame();
    ~CGame();

    CGame(const CGame&) = delete;
    CGame& operator=(const CGame&) = delete;

    struct tagWorldInfo
    {
        std::string strName;
        std::int32_t lStateLvl{};
    };

    // Полный набор PDB-полей исходного tagSetup, необходимый для LoadSetup.
    // Значения конструктора восстановлены отдельно из tagSetup::tagSetup 0x40C6E0;
    // неинициализированные скалярные поля намеренно не получают фиктивные нули.
    struct tagSetup
    {
        tagSetup();

        std::int32_t _server_version;
        std::uint32_t dwListenPort_World;
        std::uint32_t dwListenPort_Client;
        std::string strSqlConType;
        std::string strSqlServerIP;
        std::string strSqlUserName;
        std::string strSqlPassWord;
        std::string strDBName;
        bool bCheckNet;
        std::uint32_t dwMaxByteNum;
        std::uint32_t dwMaxMsgLen;
        std::uint32_t dwBanIPTime;
        bool bCheckMsgCon;
        std::int32_t lMaxConnectNum;
        std::int32_t lMaxIOSendNum;
        std::int32_t lMaxClientSendBuf;
        bool bWorldCheckNet;
        std::uint32_t dwWorldMaxByteNum;
        std::uint32_t dwWorldMaxMsgLen;
        std::uint32_t dwWorldBanIPTime;
        bool bWorldCheckMsgCon;
        std::int32_t lWorldMaxConnectNum;
        std::int32_t lWorldMaxIOSendNum;
        std::int32_t lWorldMaxClientSendBuf;
        std::uint32_t dwRefeashInfoTime;
        std::uint32_t dwSaveInfoTime;
        std::uint32_t dwDoQueueInter;
        std::uint32_t dwSendMsgToQueInter;
        std::uint32_t dwWorldMaxPlayers;
        float fWorldBusyScale;
        float fWorldFullScale;
        std::uint32_t dwPingWorldServerTime;
        std::uint32_t dwPingWorldServerErrorTime;
        bool bCheckForbidIP;
        bool bCheckAllowIP;
        bool bCheckBetweenIP;
        std::uint32_t dwServerInfoLogTime;
        std::string strServerInfoLogProvider;
        std::string strServerInfoLogUID;
        std::string strServerInfoLogPWD;
        std::string strServerInfoLogIP;
        std::string strServerInfoLogDB;
        std::uint32_t authTimeOut;
        std::string _bindIP;
        std::uint16_t _bindPort;
        std::int32_t lforbitTime;
        std::int32_t lforbitNum;
        std::int32_t lMode;
        std::string strGasAreaUserID;
        std::string strGasAreaPasswd;
        std::string _db_ip;
        std::string _db_billing_name;
        std::string _db_user;
        std::string _db_psd;
        std::int32_t m_lIsInsideUse;
        std::string m_strVerificationAddr;
        std::int32_t m_lVerifiSignUpper;
    };

    struct tagSetupEx
    {
    // Прямой конструктор CGame оставляет без явного присваивания только iAreaId.
        std::int32_t iAreaId;
        std::int32_t lClientMaxBlockConNum{10};
        std::int32_t lClientValidDelayRecDataTime{4000};
        std::int32_t lWorldMaxBlockConNum{5};
        std::int32_t lWorldValidDelayRecDataTime{4000};
        std::int32_t lQuestPlayerDataInterval{3000};
        std::int32_t matrix_timeout{60000};
        std::int32_t bValidCode{};
        std::int32_t lValidCodeOvertime{60000};
        std::int32_t iValidErrUpperLimit{3};
        std::uint32_t dwValidErrStayTime{180000U};
    };

    struct ListenPorts
    {
        std::optional<std::uint32_t> client;
        std::optional<std::uint32_t> world;
    };

    struct ASConfig
    {
        std::string _ip;
        std::uint16_t _port{};
    };

    [[nodiscard]] bool LoadSetup();
    [[nodiscard]] bool load_listen_port(
        const std::filesystem::path& runtimeDirectory = ".");
    [[nodiscard]] bool InitNetServer_Client();
    [[nodiscard]] bool InitNetServer_World();
    [[nodiscard]] bool LoadASList(
        const std::filesystem::path& path = "aslist.ini");
    [[nodiscard]] asio::awaitable<bool> InitAuthClient();
    [[nodiscard]] bool IsConnectAS() const noexcept;
    void DisconnectAS() noexcept;
    [[nodiscard]] asio::awaitable<bool> ReconnectAS();
    [[nodiscard]] bool ReassignAS(
        std::shared_ptr<LoginNet::CMyNetClientAuth> authClient);
    void ReleaseNetworkOwners() noexcept;

    [[nodiscard]] LoginNet::CMyNetServerClient* GetNetServer_Client() noexcept;
    [[nodiscard]] const LoginNet::CMyNetServerClient* GetNetServer_Client() const noexcept;
    [[nodiscard]] LoginNet::CMyNetServerWorld* GetNetServer_World() noexcept;
    [[nodiscard]] const LoginNet::CMyNetServerWorld* GetNetServer_World() const noexcept;
    [[nodiscard]] LoginNet::CMyNetClientAuth* GetAuthClient() noexcept;
    [[nodiscard]] const LoginNet::CMyNetClientAuth* GetAuthClient() const noexcept;
    [[nodiscard]] asio::io_context& IoContext() noexcept;
    [[nodiscard]] const tagSetup& Setup() const noexcept;
    [[nodiscard]] const tagSetupEx& SetupEx() const noexcept;
    [[nodiscard]] AccLogQueue& AccountLogs() noexcept;
    void AttachLoginQueue(CLoginQueue* queue) noexcept;
    [[nodiscard]] std::size_t ConfiguredWorldCount() const noexcept;
    [[nodiscard]] std::size_t ActiveWorldCount() const noexcept;
    // Прямой EXE использовал глобальный gAuthMgr. Реконструкция Linux сохраняет
    // тот же побочный эффект тайм-аута, но передаёт явного владельца AuthManager.
    [[nodiscard]] bool ReLoadSetup(AuthManager& authManager);
    [[nodiscard]] bool LoadSetupEx();
    [[nodiscard]] bool ReLoadSetupEx();
    [[nodiscard]] bool LoadWorldSetup();
    void SetListWorldInfoBySetup();
    void UpdateWorldInfoToAllClient();
    [[nodiscard]] bool ReLoadWorldSetup();

    [[nodiscard]] std::int32_t GetWorldIDByName(const char* worldName) const;
    [[nodiscard]] const char* GetWorldNameByID(std::int32_t worldId) const;
    [[nodiscard]] bool WorldServerIsOpenState(std::int32_t worldId) const;
    [[nodiscard]] bool IsExitWorld(const char* worldName) const;
    [[nodiscard]] std::int32_t
    GetLoginWorldPlayerNumByWorldName(const char* worldName) const;
    void AddWorldInfoToMsg(LoginNet::CMessage& message, const char* account) const;
    [[nodiscard]] bool SendMsg2World(const LoginNet::CMessage& message,
                                     std::int32_t worldId) const;

    [[nodiscard]] std::int32_t FindCdkey(const char* account) const;
    [[nodiscard]] bool L2W_PlayerBase_Send(const char* worldName,
                                           const char* account) const;
    [[nodiscard]] bool L2W_DeleteRole_Send(const char* worldName,
                                           const char* account,
                                           std::int32_t playerId,
                                           std::uint32_t ip) const;
    [[nodiscard]] bool L2W_RestoreRole_Send(const char* worldName,
                                            const char* account,
                                            std::uint32_t playerId) const;
    [[nodiscard]] bool L2W_CreateRole_Send(const char* worldName,
                                           const char* account,
                                           LoginNet::CMessage& message) const;
    [[nodiscard]] bool L2W_QuestDetail_Send(const char* worldName,
                                            const char* account,
                                            std::int32_t playerId,
                                            std::uint32_t ip) const;

    [[nodiscard]] bool AddCdkey(const char* account, std::int32_t worldId);
    void ClearLoginCdkey(const char* account);
    void ClearCDKey(const char* account);
    void ClearCDKeyByWorldServerID(std::int32_t worldId);
    [[nodiscard]] std::int32_t AddWorld(std::int32_t worldId,
                                        const char* worldName);
    [[nodiscard]] std::int32_t DelWorld(std::int32_t worldId);
    [[nodiscard]] std::int32_t GetLoginWorldCdkeyNumbers() const noexcept;
    [[nodiscard]] std::uint32_t GetCdkeyCount() const noexcept;

    [[nodiscard]] const char* GetLoginCdkeyWorldServer(const char* account) const;
    void SetLoginCdkeyWorldServer(const char* account, const char* worldServer);
    [[nodiscard]] bool KickOut(const char* account) const;

    void AccountEnterLog(const char* account, std::uint32_t ip);
    [[nodiscard]] std::int32_t PrepareEnter(const char* account,
                                            std::uint32_t ip,
                                            std::int32_t socketId,
                                            const char* worldServer,
                                            bool matrix);
    void EnterGame(const char* account,
                   std::uint32_t ip,
                   std::int32_t socketId,
                   const char* worldServer) const;

    // В исходном EXE getAccInfoEx вызывается ради побочного эффекта через ADO;
    // четвёртый int и @Result не участвуют в автомате состояний, функция всегда
    // возвращает false.
    [[nodiscard]] bool ExecuteProce(std::string userId,
                                    std::string userIp,
                                    char* passwordHex,
                                    std::int32_t unusedResult);

private:
    struct AuthConnectResult
    {
        std::shared_ptr<LoginNet::CMyNetClientAuth> client;
        std::optional<ASConfig> connected;
    };

    void ChangeAllWorldSate();
    void UpdateWorldStateFromCdkeyCount(std::int32_t worldId);
    void RecordTechnicalError(std::string detail);
    [[nodiscard]] asio::awaitable<AuthConnectResult> ConnectNewAuthClient();
    [[nodiscard]] std::int32_t SendLSInfoToAS();

    CLoginQueue* m_pLoginQueue{};
    asio::io_context m_IoContext;
    std::unique_ptr<LoginNet::CMyNetServerClient> s_pNetServer_Client;
    std::unique_ptr<LoginNet::CMyNetServerWorld> s_pNetServer_World;
    std::vector<ASConfig> m_ASList;
    std::optional<ASConfig> m_CurASCfg;
    std::shared_ptr<LoginNet::CMyNetClientAuth> m_ASClient;

    std::map<std::string, std::string> m_LoginCdkeyWorld;
    std::map<std::int32_t, std::list<std::string>> s_listCdkey;
    std::array<std::int32_t, 4> m_StateLvl{-1, 10000, 20000, 30000};
    std::map<std::int32_t, tagWorldInfo> m_listWorldInfo;
    std::map<std::int32_t, tagWorldInfo> m_WorldInfoSetup;
    tagSetup m_Setup;
    tagSetupEx m_SetupEx;
    ListenPorts m_ListenPorts;
    AccLogQueue _acc_logs;
};
}
