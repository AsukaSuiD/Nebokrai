#include "game.h"

#include "authmanager.h"
#include "loginqueue.h"
#include "../dbaccess/myadobase.h"
#include "../dbaccess/odbc.h"
#include "../public/readwrite.h"
#include "../public/textcodec.h"
#include "../public/tools.h"
#include "../nets/netlogin/message.h"
#include "../nets/netlogin/mynetserver_client.h"
#include "../nets/netlogin/mynetserver_world.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <array>
#include <bit>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <limits>
#include <new>
#include <sstream>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace Login
{
namespace
{
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::int32_t kWorldInfoUpdateMessageType = 0x000AF509;
constexpr std::int32_t kPlayerBaseMessageType = 0x0004FB01;
constexpr std::int32_t kKickWorldAccountMessageType = 0x0004FB07;
constexpr std::size_t kLegacyAccountLogBufferSize = 0x200U;

std::span<const std::uint8_t> CStringBytes(const char* value)
{
    if (value == nullptr) {
        return {};
    }
    return {reinterpret_cast<const std::uint8_t*>(value), std::strlen(value)};
}

std::vector<std::uint8_t> OwnedCStringBytes(const char* value)
{
    const auto bytes = CStringBytes(value);
    return {bytes.begin(), bytes.end()};
}

std::string LegacyIpv4Text(std::uint32_t address)
{
    return std::to_string(address & 0xFFU) + '.' +
           std::to_string((address >> 8U) & 0xFFU) + '.' +
           std::to_string((address >> 16U) & 0xFFU) + '.' +
           std::to_string((address >> 24U) & 0xFFU);
}

template <typename Value>
void ReadLabeled(std::istream& input, std::string& label, Value& value)
{
    input >> label >> value;
}

bool ReadLegacySetupBody(std::istream& input, CGame::tagSetup& setup)
{
    std::string label;
    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ 0x0040E690, ветка setup.dat: старая схема
    // начинается сразу с портов Client/World и заканчивается на _bindPort.
    ReadLabeled(input, label, setup.dwListenPort_Client);
    ReadLabeled(input, label, setup.dwListenPort_World);
    ReadLabeled(input, label, setup.strSqlConType);
    ReadLabeled(input, label, setup.strSqlServerIP);
    ReadLabeled(input, label, setup.strSqlUserName);
    ReadLabeled(input, label, setup.strSqlPassWord);
    ReadLabeled(input, label, setup.strDBName);
    ReadLabeled(input, label, setup.bCheckNet);
    ReadLabeled(input, label, setup.dwMaxByteNum);
    ReadLabeled(input, label, setup.dwMaxMsgLen);
    ReadLabeled(input, label, setup.dwBanIPTime);
    ReadLabeled(input, label, setup.bCheckMsgCon);
    ReadLabeled(input, label, setup.lMaxConnectNum);
    ReadLabeled(input, label, setup.lMaxIOSendNum);
    ReadLabeled(input, label, setup.lMaxClientSendBuf);
    ReadLabeled(input, label, setup.bWorldCheckNet);
    ReadLabeled(input, label, setup.dwWorldMaxByteNum);
    ReadLabeled(input, label, setup.dwWorldMaxMsgLen);
    ReadLabeled(input, label, setup.dwWorldBanIPTime);
    ReadLabeled(input, label, setup.bWorldCheckMsgCon);
    ReadLabeled(input, label, setup.lWorldMaxConnectNum);
    ReadLabeled(input, label, setup.lWorldMaxIOSendNum);
    ReadLabeled(input, label, setup.lWorldMaxClientSendBuf);
    ReadLabeled(input, label, setup.dwRefeashInfoTime);
    ReadLabeled(input, label, setup.dwSaveInfoTime);
    ReadLabeled(input, label, setup.dwDoQueueInter);
    ReadLabeled(input, label, setup.dwSendMsgToQueInter);
    ReadLabeled(input, label, setup.dwWorldMaxPlayers);
    ReadLabeled(input, label, setup.fWorldBusyScale);
    ReadLabeled(input, label, setup.fWorldFullScale);
    ReadLabeled(input, label, setup.dwPingWorldServerTime);
    ReadLabeled(input, label, setup.dwPingWorldServerErrorTime);
    ReadLabeled(input, label, setup.bCheckForbidIP);
    ReadLabeled(input, label, setup.bCheckAllowIP);
    ReadLabeled(input, label, setup.bCheckBetweenIP);
    ReadLabeled(input, label, setup.dwServerInfoLogTime);
    ReadLabeled(input, label, setup.strServerInfoLogProvider);
    ReadLabeled(input, label, setup.strServerInfoLogUID);
    ReadLabeled(input, label, setup.strServerInfoLogPWD);
    ReadLabeled(input, label, setup.strServerInfoLogIP);
    ReadLabeled(input, label, setup.strServerInfoLogDB);
    ReadLabeled(input, label, setup.authTimeOut);
    ReadLabeled(input, label, setup._bindIP);
    ReadLabeled(input, label, setup._bindPort);
    return !input.fail();
}

bool ReadPlainSetup(std::istream& input, CGame::tagSetup& setup)
{
    std::string label;
    // Открытый setup.ini содержит одно новое поле-префикс, затем точное старое тело.
    ReadLabeled(input, label, setup._server_version);
    const bool legacyBodyComplete = ReadLegacySetupBody(input, setup);

    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ: расширение только открытого файла после _bindPort.
    ReadLabeled(input, label, setup.lforbitTime);
    ReadLabeled(input, label, setup.lforbitNum);
    ReadLabeled(input, label, setup.lMode);
    ReadLabeled(input, label, setup.strGasAreaUserID);
    ReadLabeled(input, label, setup.strGasAreaPasswd);
    ReadLabeled(input, label, setup._db_ip);
    ReadLabeled(input, label, setup._db_billing_name);
    ReadLabeled(input, label, setup._db_user);
    ReadLabeled(input, label, setup._db_psd);
    const bool requiredExtensionComplete = !input.fail();
    ReadLabeled(input, label, setup.m_lIsInsideUse);
    ReadLabeled(input, label, setup.m_strVerificationAddr);
    ReadLabeled(input, label, setup.m_lVerifiSignUpper);
    return legacyBodyComplete && requiredExtensionComplete;
}

std::optional<std::int32_t> LegacyScaledWorldLevel(std::uint32_t worldMax,
                                                    float scale)
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x0040EDA2 / 0x0040F402: MSVC преобразует
    // беззнаковый dword через знаковый FILD и добавляет ровно 2^32 при старшем бите.
    const long double product = static_cast<long double>(worldMax) *
                                static_cast<long double>(scale);
    if (!std::isfinite(product) ||
        product < static_cast<long double>(std::numeric_limits<std::int32_t>::min()) ||
        product > static_cast<long double>(std::numeric_limits<std::int32_t>::max())) {
        return std::nullopt;
    }
    return static_cast<std::int32_t>(product);
}
}

CGame::tagSetup::tagSetup()
{
    // ПОДТВЕРЖДЕНО НАПРЯМУЮ: tagSetup::tagSetup 0x0040C6E0. Фундаментальные
    // поля, не перечисленные здесь, исходный конструктор явно не инициализировал.
    bCheckNet = true;
    dwMaxByteNum = 5000U;
    dwMaxMsgLen = 0x19000U;
    dwBanIPTime = 10U;
    bCheckMsgCon = true;
    dwRefeashInfoTime = 1000U;
    dwSaveInfoTime = 60000U;
    authTimeOut = 5000U;
    _bindIP = "0.0.0.0";
    _bindPort = 0U;
    lMode = 0;
    strGasAreaUserID.clear();
    strGasAreaPasswd.clear();
    _db_ip.clear();
    _db_billing_name.clear();
    _db_user.clear();
    _db_psd.clear();
    m_lIsInsideUse = 1;
}

bool CGame::LoadSetup()
{
    // ПОДТВЕРЖДЕНО НАПРЯМУЮ 0x0040E690: открытый setup.ini имеет приоритет.
    // setup.dat читается только тогда, когда setup.ini открыть невозможно.
    std::ifstream plain("setup.ini");
    if (plain.is_open()) {
        const bool requiredFieldsComplete = ReadPlainSetup(plain, m_Setup);
        if (!requiredFieldsComplete) {
            // Обязательная часть заканчивается на _db_psd: сохранённый штатный
            // setup.ini содержит все эти 54 пары. Их отсутствие оставило бы
            // сетевые, временные или DB-поля без доказанного значения.
            RecordTechnicalError("LoadSetup: обязательная часть setup.ini неполна");
            return false;
        }

        // Сохранённый штатный setup.ini содержит ровно 54 пары и заканчивается
        // после _db_psd. Три последующих извлечения исходно получают состояние
        // ошибки, но полная обязательная часть остаётся успехом; значения
        // конструктора m_lIsInsideUse/m_strVerificationAddr сохраняются без
        // выдуманных значений хвоста.
    } else {
        char setupDatName[] = "setup.dat";
        const int fileLength = GetFileLength(setupDatName);
        std::FILE* encodedFile = std::fopen(setupDatName, "rb");
        if (encodedFile == nullptr) {
            return false;
        }

        if (fileLength < 0 ||
            static_cast<std::size_t>(fileLength) >
                (std::numeric_limits<std::size_t>::max() - 2U) / 2U) {
            std::fclose(encodedFile);
            RecordTechnicalError("LoadSetup: длина setup.dat не помещается в допустимый диапазон");
            return false;
        }

        try {
            // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ: кодированный буфер = len+1,
            // декодированный = len*2+2; оба обнуляются перед fread/IniDecoder.
            std::vector<char> encoded(static_cast<std::size_t>(fileLength) + 1U, '\0');
            std::vector<char> decoded(static_cast<std::size_t>(fileLength) * 2U + 2U, '\0');

            // Прямой владелец игнорирует результат fread. Исходный буфер заранее
            // обнулён, поэтому короткое чтение оставляет нули в непрочитанном хвосте.
            static_cast<void>(std::fread(encoded.data(),
                                         static_cast<std::size_t>(fileLength),
                                         1U,
                                         encodedFile));
            std::fclose(encodedFile);
            encodedFile = nullptr;

            IniDecoder(encoded.data(), decoded.data(), fileLength);

            // Прямой код помещает декодированную C-строку в stringstream и
            // вызывает seekg(0), поэтому встроенный NUL обрывает разбор при
            // размере 2*len+2.
            std::stringstream decodedStream;
            decodedStream << decoded.data();
            decodedStream.seekg(0, std::ios::beg);
            if (!ReadLegacySetupBody(decodedStream, m_Setup)) {
                RecordTechnicalError(
                    "LoadSetup: обязательная часть декодированного setup.dat неполна");
                return false;
            }
        } catch (const std::bad_alloc&) {
            if (encodedFile != nullptr) {
                std::fclose(encodedFile);
            }
            RecordTechnicalError("LoadSetup: не удалось выделить буфер для setup.dat");
            return false;
        }
    }

    const auto busyLevel =
        LegacyScaledWorldLevel(m_Setup.dwWorldMaxPlayers, m_Setup.fWorldBusyScale);
    const auto fullLevel =
        LegacyScaledWorldLevel(m_Setup.dwWorldMaxPlayers, m_Setup.fWorldFullScale);
    if (!busyLevel || !fullLevel) {
        RecordTechnicalError("LoadSetup: порог состояния мира не помещается в старый long");
        return false;
    }

    // ПОДТВЕРЖДЕНА последовательность x87 после разбора в обеих ветках.
    m_StateLvl[0] = -1;
    m_StateLvl[1] = *busyLevel;
    m_StateLvl[2] = *fullLevel;
    m_StateLvl[3] = std::bit_cast<std::int32_t>(m_Setup.dwWorldMaxPlayers);
    ChangeAllWorldSate();

    if (m_pLoginQueue != nullptr) {
        m_pLoginQueue->OnInitial(m_Setup.dwDoQueueInter,
                                 m_Setup.dwSendMsgToQueInter,
                                 m_Setup.dwWorldMaxPlayers);
    }

    // Прямой владелец добавляет успешный AddLogText только после этой точки.
    return true;
}

bool CGame::ReLoadSetup(AuthManager& authManager)
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x0040F4E9: результат намеренно игнорируется.
    static_cast<void>(LoadSetup());

    if (s_pNetServer_Client == nullptr) {
        // Прямой EXE разыменовал бы здесь null. Сохраняем поведение допустимого
        // состояния, а невозможный пробел владельца обозначаем технической границей.
        RecordTechnicalError("ReLoadSetup: отсутствует владелец клиентского сервера");
        return false;
    }
    s_pNetServer_Client->ConfigureClientTransportForReload(
        m_Setup.bCheckNet,
        m_Setup.bCheckMsgCon,
        m_Setup.dwMaxByteNum,
        m_Setup.dwBanIPTime,
        m_Setup.lMaxConnectNum,
        m_Setup.lMaxIOSendNum,
        m_Setup.dwMaxMsgLen,
        m_Setup.lMaxClientSendBuf);

    if (s_pNetServer_World == nullptr) {
        // Эта граница намеренно расположена после обновления клиента, как и точка
        // падения прямого кода: настройки клиента уже могли быть перезаписаны.
        RecordTechnicalError("ReLoadSetup: отсутствует владелец сервера мира");
        return false;
    }
    s_pNetServer_World->ConfigureWorldTransportForReload(
        m_Setup.bWorldCheckNet,
        m_Setup.bWorldCheckMsgCon,
        m_Setup.dwWorldMaxByteNum,
        m_Setup.dwWorldBanIPTime,
        m_Setup.lWorldMaxConnectNum,
        m_Setup.lWorldMaxIOSendNum,
        m_Setup.dwWorldMaxMsgLen,
        m_Setup.lWorldMaxClientSendBuf);

    // Прямой владелец последним записывает gAuthMgr._time_out. Сам gAuthMgr не
    // воскрешается: AuthManager уже явно представлен в другой части восстановления.
    authManager.SetTimeout(m_Setup.authTimeOut);
    return true;
}

bool CGame::LoadSetupEx()
{
    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ 0x0040D9E0: буквальное имя файла в нижнем
    // регистре и обычный поток с пробельным разделением; каждая подпись читается
    // и отбрасывается перед значением. После успешного открытия исходник НЕ
    // проверяет failbit и всегда возвращает true, поэтому неполное извлечение
    // намеренно оставляет уже прочитанные поля.
    std::ifstream input("setupex.ini");
    if (!input.is_open()) {
        return false;
    }

    std::string label;
    input >> label >> m_SetupEx.iAreaId
          >> label >> m_SetupEx.lClientMaxBlockConNum
          >> label >> m_SetupEx.lClientValidDelayRecDataTime
          >> label >> m_SetupEx.lWorldMaxBlockConNum
          >> label >> m_SetupEx.lWorldValidDelayRecDataTime
          >> label >> m_SetupEx.lQuestPlayerDataInterval
          >> label >> m_SetupEx.matrix_timeout
          >> label >> m_SetupEx.bValidCode
          >> label >> m_SetupEx.lValidCodeOvertime
          >> label >> m_SetupEx.iValidErrUpperLimit
          >> label >> m_SetupEx.dwValidErrStayTime;

    // Затем прямой владелец только вызывает AddLogText("load setupex.ini...ok!").
    // Общий журнал ещё не восстановлен; запись об успехе не является побочным
    // эффектом машины состояний.
    return true;
}

bool CGame::ReLoadSetupEx()
{
    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ 0x0040DC60.
    if (!LoadSetupEx()) {
        return false;
    }

    if (s_pNetServer_Client == nullptr || s_pNetServer_World == nullptr) {
        // EXE здесь напрямую разыменовывает обоих владельцев. В Linux не повторяем
        // разыменование null: это типизированная техническая граница, а не новый
        // результат входа.
        RecordTechnicalError("ReLoadSetupEx: отсутствует владелец сетевого сервера");
        return false;
    }

    // Прямые поля CServer +0x160/+0x164 = m_lMaxBlockConnetNum/m_lSendInterTime.
    s_pNetServer_Client->ConfigureAcceptLimitsAfterHost(
        m_SetupEx.lClientMaxBlockConNum,
        m_SetupEx.lClientValidDelayRecDataTime);
    s_pNetServer_World->ConfigureAcceptLimitsAfterHost(
        m_SetupEx.lWorldMaxBlockConNum,
        m_SetupEx.lWorldValidDelayRecDataTime);
    return true;
}

bool CGame::LoadWorldSetup()
{
    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ 0x00410F30: clear выполняется до открытия,
    // поэтому ошибка открытия намеренно оставляет карту настроек пустой.
    m_WorldInfoSetup.clear();

    std::error_code pathError;
    const auto path = ResolveLegacyFileAsciiCase(
        std::filesystem::path{"."}, "WorldInfoSetup.ini", pathError);
    if (!path) {
        RecordTechnicalError(
            "LoadWorldSetup: не удалось просмотреть рабочий каталог: " +
            pathError.message());
        return false;
    }

    std::ifstream input(*path);
    if (!input.is_open()) {
        return false;
    }

    while (ReadTo(input, "#")) {
        std::int32_t worldId;
        std::string worldName;
        std::int32_t state;
        if (!(input >> worldId >> worldName >> state)) {
            // После неудачного извлечения прямой код использовал бы
            // неинициализированные значения стека. Linux сохраняет уже разобранную
            // карту и останавливается до неопределённой записи; успешное открытие
            // остаётся успешным.
            RecordTechnicalError("LoadWorldSetup: неполная запись мира");
            break;
        }

        auto& world = m_WorldInfoSetup[worldId];
        world.strName = std::move(worldName);
        world.lStateLvl = state;
    }
    return true;
}

void CGame::SetListWorldInfoBySetup()
{
    // ПОДТВЕРЖДЕНО ДЕКОМПИЛЯЦИЕЙ 0x00411170: очистить/скопировать карту настроек,
    // затем принудительно обнулить каждое живое состояние; имена и ключи те же.
    m_listWorldInfo = m_WorldInfoSetup;
    for (auto& [worldId, world] : m_listWorldInfo) {
        static_cast<void>(worldId);
        world.lStateLvl = 0;
    }
}

void CGame::UpdateWorldInfoToAllClient()
{
    // ПОДТВЕРЖДЕНО НАПРЯМУЮ 0x00407860: обновление 0xAF509 получают только
    // учётные записи с пустым сохранённым World. Порядок задаёт m_LoginCdkeyWorld.
    for (const auto& [account, worldServer] : m_LoginCdkeyWorld) {
        if (!worldServer.empty()) {
            continue;
        }

        LoginNet::CMessage response(kWorldInfoUpdateMessageType);
        AddWorldInfoToMsg(response, account.c_str());
        if (s_pNetServer_Client == nullptr) {
            // Прямой CMessage::SendToClient достигал отсутствующего владельца
            // клиента только при наличии подходящей учётной записи.
            RecordTechnicalError(
                "UpdateWorldInfoToAllClient: отсутствует владелец клиентского сервера");
            return;
        }
        static_cast<void>(response.SendToClientCdkey(
            s_pNetServer_Client->CommandHandle(), CStringBytes(account.c_str())));
    }
}

bool CGame::ReLoadWorldSetup()
{
    // ПОДТВЕРЖДЕНО НАПРЯМУЮ 0x00411FF0: оба результата/побочных эффекта
    // безусловны; ошибка LoadWorldSetup намеренно игнорируется.
    static_cast<void>(LoadWorldSetup());
    UpdateWorldInfoToAllClient();

    // Последний исходный вызов UpdateDisplayWorldInfo() лишь перестраивал список
    // администратора Win32 через SendMessageA; сетевого состояния в Linux нет.
    return true;
}

void CGame::ChangeAllWorldSate()
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x004075E0. Пересчитываются только миры с уже
    // существующей записью s_listCdkey; отсутствующие сохраняют прежнее состояние.
    for (auto& [worldId, world] : m_listWorldInfo) {
        const auto accounts = s_listCdkey.find(worldId);
        if (accounts == s_listCdkey.end()) {
            continue;
        }

        const std::uint32_t rawCount =
            static_cast<std::uint32_t>(accounts->second.size());
        const std::int32_t signedCount = std::bit_cast<std::int32_t>(rawCount);

        std::int32_t state = 1;
        while (state <= 3 && signedCount >= m_StateLvl[static_cast<std::size_t>(state)]) {
            ++state;
        }
        if (state > 3) {
            state = 3;
        }
        world.lStateLvl = state;
    }
}

std::int32_t CGame::GetWorldIDByName(const char* worldName) const
{
    if (worldName == nullptr) {
        return -1;
    }
    for (const auto& [worldId, world] : m_listWorldInfo) {
        if (std::strcmp(worldName, world.strName.c_str()) == 0) {
            return world.lStateLvl == 0 ? -1 : worldId;
        }
    }
    return -1;
}

bool CGame::WorldServerIsOpenState(std::int32_t worldId) const
{
    const auto found = m_WorldInfoSetup.find(worldId);
    return found != m_WorldInfoSetup.end() && found->second.lStateLvl != 0;
}

void CGame::AddWorldInfoToMsg(LoginNet::CMessage& message, const char* account) const
{
    // VERIFIED_ASSEMBLY 0x004074E0: map size идёт short, затем для каждого
    // live world — long state и C-string name в map-key order.
    const auto lowCount = static_cast<std::uint16_t>(m_listWorldInfo.size());
    message.Base().Add(std::bit_cast<std::int16_t>(lowCount));

    for (const auto& [worldId, world] : m_listWorldInfo) {
        const auto setup = m_WorldInfoSetup.find(worldId);
        bool visible =
            setup != m_WorldInfoSetup.end() && setup->second.lStateLvl != 0;
        if (!visible && m_pLoginQueue != nullptr) {
            visible = m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        }
        message.Base().Add(visible ? world.lStateLvl : 0);
        message.Base().Add(world.strName.c_str());
    }
}

std::int32_t CGame::FindCdkey(const char* account) const
{
    if (account == nullptr) {
        return -1;
    }
    for (const auto& [worldId, accounts] : s_listCdkey) {
        if (std::find(accounts.begin(), accounts.end(), account) != accounts.end()) {
            return worldId;
        }
    }
    return -1;
}

bool CGame::L2W_PlayerBase_Send(const char* worldName, const char* account) const
{
    if (worldName == nullptr || account == nullptr || account[0] == '\0') {
        return false;
    }

    const std::int32_t worldId = GetWorldIDByName(worldName);
    if (worldId == -1) {
        return false;
    }

    if (!WorldServerIsOpenState(worldId)) {
        const bool noQueue =
            m_pLoginQueue != nullptr &&
            m_pLoginQueue->IsInNoQueueList(CStringBytes(account));
        if (!noQueue) {
            return false;
        }
    }

    LoginNet::CMessage message(kPlayerBaseMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    // Direct EXE возвращает true после логических проверок и не анализирует send.
    return true;
}

const char* CGame::GetLoginCdkeyWorldServer(const char* account) const
{
    if (account == nullptr) {
        return nullptr;
    }
    const auto found = m_LoginCdkeyWorld.find(account);
    return found == m_LoginCdkeyWorld.end() ? nullptr : found->second.c_str();
}

void CGame::SetLoginCdkeyWorldServer(const char* account, const char* worldServer)
{
    if (account == nullptr || worldServer == nullptr) {
        return;
    }
    m_LoginCdkeyWorld[account] = worldServer;
}

bool CGame::KickOut(const char* account) const
{
    if (account == nullptr) {
        return false;
    }

    if (GetLoginCdkeyWorldServer(account) != nullptr) {
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(
                s_pNetServer_Client->QuitClientByMapName(CStringBytes(account)));
        }
        return true;
    }

    const std::int32_t worldId = FindCdkey(account);
    if (worldId == -1) {
        return false;
    }

    LoginNet::CMessage message(kKickWorldAccountMessageType);
    message.Base().Add(account);
    if (s_pNetServer_World != nullptr) {
        static_cast<void>(
            message.SendToWorldMap(s_pNetServer_World->CommandHandle(), worldId));
    }
    return true;
}

void CGame::AccountEnterLog(const char* account, std::uint32_t ip)
{
    if (account == nullptr) {
        return;
    }

    std::array<char, 64> time{};
    static_cast<void>(CMyAdoBase::GetTimeString(time.data(), time.size()));
    if (time[0] == '\0') {
        RecordTechnicalError("AccountEnterLog: не удалось получить локальное время");
        return;
    }

    const std::string ipText = LegacyIpv4Text(ip);
    // Прямой владелец использует char[512] + sprintf. Дошедшие сюда имена учётных
    // записей не длиннее 31 байта, поэтому штатное поведение побайтно совпадает.
    // Слишком большой внешний ввод — явная граница вместо выхода за стек.
    constexpr std::size_t kLegacySqlLiteralSize =
        sizeof("INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES('','','')") - 1U;
    if (kLegacySqlLiteralSize + std::strlen(account) + std::strlen(time.data()) +
            ipText.size() >=
        kLegacyAccountLogBufferSize) {
        RecordTechnicalError("AccountEnterLog: SQL не помещается в старый char[512]");
        return;
    }
    _acc_logs.Push(AccLogRecord{
        .account = account,
        .enteredAt = time.data(),
        .ip = ipText,
    });
}

std::int32_t CGame::PrepareEnter(const char* account,
                                 std::uint32_t ip,
                                 std::int32_t socketId,
                                 const char* worldServer,
                                 bool matrix)
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return 1;
    }

    if (worldServer[0] == '\0' && KickOut(account)) {
        // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x0041198C..0x004119B4: в отличие от старого
        // Linux-донора оригинал отправляет 0x08 в НОВЫЙ сокет и сразу возвращается.
        LoginNet::CMessage response(kLoginResponseMessageType);
        response.Base().Add(static_cast<char>(0x08));
        if (s_pNetServer_Client != nullptr) {
            static_cast<void>(response.SendToClientSocket(
                s_pNetServer_Client->CommandHandle(), socketId));
        }
        return 1;
    }

    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(
            s_pNetServer_Client->SetClientMapName(socketId, CStringBytes(account)));
    } else {
        RecordTechnicalError("PrepareEnter: отсутствует владелец клиентского сервера");
    }

    AccountEnterLog(account, ip);
    SetLoginCdkeyWorldServer(account, worldServer);

    if (worldServer[0] != '\0' || !matrix) {
        return 0;
    }

    if (m_pLoginQueue == nullptr) {
        RecordTechnicalError("PrepareEnter: для matrix_register отсутствует владелец очереди входа");
        return 1;
    }

    TagPwdChecked checked(socketId,
                          ip,
                          OwnedCStringBytes(account),
                          OwnedCStringBytes(worldServer),
                          true);
    if (auto error = m_pLoginQueue->MatrixRegister(
            checked,
            [this](const LoginNet::CMessage& message, std::int32_t targetSocket) {
                if (s_pNetServer_Client != nullptr) {
                    static_cast<void>(message.SendToClientSocket(
                        s_pNetServer_Client->CommandHandle(), targetSocket));
                }
            })) {
        RecordTechnicalError("PrepareEnter/matrix_register: " + error->detail);
    }
    return 1;
}

void CGame::EnterGame(const char* account,
                      std::uint32_t ip,
                      std::int32_t socketId,
                      const char* worldServer) const
{
    if (account == nullptr || ip == 0U || socketId == 0 || worldServer == nullptr) {
        return;
    }

    if (worldServer[0] != '\0') {
        static_cast<void>(L2W_PlayerBase_Send(worldServer, account));
        return;
    }

    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(0x02));
    response.Base().Add(account);
    AddWorldInfoToMsg(response, account);
    if (s_pNetServer_Client != nullptr) {
        static_cast<void>(response.SendToClientSocket(
            s_pNetServer_Client->CommandHandle(), socketId));
    }
}

bool CGame::ExecuteProce(std::string userId,
                         std::string userIp,
                         char* passwordHex,
                         std::int32_t unusedResult)
{
    // VERIFIED_ASSEMBLY 0x00407AB0..0x0040822C: этот аргумент не читается.
    static_cast<void>(unusedResult);
    if (passwordHex == nullptr) {
        RecordTechnicalError("ExecuteProce: указатель UserPwd равен null");
        return false;
    }

    const auto convertedUser = Nebokrai::Windows1251ToUtf8(userId);
    const auto convertedIp = Nebokrai::Windows1251ToUtf8(userIp);
    const auto convertedPassword = Nebokrai::Windows1251ToUtf8(passwordHex);

    const Nebokrai::TextConversionResult* conversions[] = {
        &convertedUser, &convertedIp, &convertedPassword};
    for (const Nebokrai::TextConversionResult* conversion : conversions) {
        if (!*conversion) {
            RecordTechnicalError("ExecuteProce: " + conversion->error);
            return false;
        }
    }

    // ADO/COM — техническая обвязка только Windows. Серверный стек Linux использует
    // unixODBC с Microsoft ODBC Driver 18; игровая семантика и контракт процедуры
    // остаются в исходном владельце CGame.
    auto connectionText = Nebokrai::Database::BuildMssqlConnectionString(
        m_Setup._db_ip,
        m_Setup._db_billing_name,
        m_Setup._db_user,
        m_Setup._db_psd);
    if (auto* error = std::get_if<Nebokrai::Database::OdbcError>(&connectionText)) {
        RecordTechnicalError("ExecuteProce: " + error->detail);
        return false;
    }

    Nebokrai::Database::OdbcConnection db;
    if (auto error = db.Open(std::get<std::string>(connectionText))) {
        RecordTechnicalError("ExecuteProce: " + error->detail);
        return false;
    }
    Nebokrai::Database::OdbcStatement statement;
    if (auto error = statement.Create(db)) {
        RecordTechnicalError("ExecuteProce: " + error->detail);
        return false;
    }

    // В оригинале ADODB CommandType=4 (хранимая процедура), CommandText=getAccInfoEx.
    SQLCHAR procedure[] = "{CALL getAccInfoEx(?,?,?,?)}";
    if (auto error = statement.Prepare(
            std::string_view(reinterpret_cast<const char*>(procedure)))) {
        RecordTechnicalError("ExecuteProce: " + error->detail);
        return false;
    }

    std::string parameterUser = *convertedUser.value;
    std::string parameterIp = *convertedIp.value;
    std::string parameterPassword = *convertedPassword.value;
    SQLLEN userLength = static_cast<SQLLEN>(parameterUser.size());
    SQLLEN ipLength = static_cast<SQLLEN>(parameterIp.size());
    SQLLEN passwordLength = static_cast<SQLLEN>(parameterPassword.size());
    SQLINTEGER procedureResult = 0;
    SQLLEN resultLength = 0;

    const auto bindText = [&](SQLUSMALLINT number,
                              std::string& value,
                              SQLULEN legacySize,
                              SQLLEN& length) -> bool {
        const SQLRETURN bound = SQLBindParameter(
            statement.NativeHandle(),
            number,
            SQL_PARAM_INPUT,
            SQL_C_CHAR,
            SQL_VARCHAR,
            legacySize,
            0,
            value.data(),
            static_cast<SQLLEN>(value.size() + 1U),
            &length);
        if (!Nebokrai::Database::OdbcSucceeded(bound)) {
            RecordTechnicalError(
                "ExecuteProce: " +
                Nebokrai::Database::OdbcDiagnostic(
                    SQL_HANDLE_STMT,
                    statement.NativeHandle(),
                    "SQLBindParameter").detail);
            return false;
        }
        return true;
    };

    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ: входной @UserID типа adVarChar имеет размер 0x20,
    // @UserIP size 0x18, @UserPwd size 0x40, затем @Result adInteger OUTPUT.
    if (!bindText(1, parameterUser, 0x20U, userLength) ||
        !bindText(2, parameterIp, 0x18U, ipLength) ||
        !bindText(3, parameterPassword, 0x40U, passwordLength)) {
        return false;
    }
    SQLRETURN result = SQLBindParameter(statement.NativeHandle(),
                              4,
                              SQL_PARAM_OUTPUT,
                              SQL_C_SLONG,
                              SQL_INTEGER,
                              0,
                              0,
                              &procedureResult,
                              sizeof(procedureResult),
                              &resultLength);
    if (!Nebokrai::Database::OdbcSucceeded(result)) {
        RecordTechnicalError(
            "ExecuteProce: " +
            Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT,
                statement.NativeHandle(),
                "SQLBindParameter(@Result)").detail);
        return false;
    }

    if (auto error = statement.Execute()) {
        RecordTechnicalError("ExecuteProce: " + error->detail);
        return false;
    }

    // Прямой EXE не читает выходной @Result и при успехе тоже делает xor al,al.
    static_cast<void>(procedureResult);
    return false;
}

void CGame::RecordTechnicalError(std::string detail)
{
    spdlog::error("{}", detail);
}
}
