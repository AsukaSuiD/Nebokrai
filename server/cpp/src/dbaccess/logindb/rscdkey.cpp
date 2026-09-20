#include "rscdkey.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace
{
constexpr int kSecurityValidatePosNum = 3;
constexpr std::size_t kCdkeyMaxBytes = 50;

CRsCDKey::Setup g_setup;
CRsCDKey::Database* g_database = nullptr;

bool AsciiEqualsIgnoreCase(std::string_view lhs, std::string_view rhs)
{
    if (lhs.size() != rhs.size()) {
        return false;
    }

    for (std::size_t i = 0; i < lhs.size(); ++i) {
        const auto left = static_cast<unsigned char>(lhs[i]);
        const auto right = static_cast<unsigned char>(rhs[i]);
        if (std::tolower(left) != std::tolower(right)) {
            return false;
        }
    }

    return true;
}

template <typename Map>
auto FindColumn(const Map& map, std::string_view name)
{
    return std::find_if(map.begin(), map.end(), [name](const auto& item) {
        return AsciiEqualsIgnoreCase(item.first, name);
    });
}

template <typename... Args>
std::string FormatString(const char* format, Args... args)
{
    std::vector<char> buffer(512U);
    for (;;) {
        const int written = std::snprintf(buffer.data(), buffer.size(), format, args...);
        if (written < 0) {
            throw std::runtime_error("Не удалось сформировать строку");
        }
        if (static_cast<std::size_t>(written) < buffer.size()) {
            return std::string(buffer.data(), static_cast<std::size_t>(written));
        }
        buffer.resize(static_cast<std::size_t>(written) + 1U);
    }
}

std::uint32_t ReverseIPv4Dword(std::uint32_t value)
{
    const std::uint32_t b1 = value & 0x000000ffU;
    const std::uint32_t b2 = value & 0x0000ff00U;
    const std::uint32_t b3 = value & 0x00ff0000U;
    const std::uint32_t b4 = value & 0xff000000U;
    return (b1 << 24U) | (b2 << 8U) | (b3 >> 8U) | (b4 >> 24U);
}

bool CountQueryMatches(const std::string& sql)
{
    if (g_database == nullptr) {
        return false;
    }

    const auto rows = g_database->Query(sql);
    if (rows.empty()) {
        return false;
    }

    return rows.front().GetLong("exp1") > 0;
}

bool CopyStringToOutput(char* output, std::size_t capacity, std::string_view value)
{
    if (output == nullptr || capacity == 0 || value.size() >= capacity) {
        return false;
    }

    std::memcpy(output, value.data(), value.size());
    output[value.size()] = '\0';
    return true;
}

bool IsBoundedValue(std::string_view value, std::size_t maxBytes,
                    bool allowEmpty = false)
{
    return (allowEmpty || !value.empty()) && value.size() <= maxBytes;
}

std::string EscapeSqlLiteral(std::string_view value)
{
    std::string escaped;
    escaped.reserve(value.size());
    for (const char ch : value) {
        escaped.push_back(ch);
        if (ch == '\'') {
            escaped.push_back('\'');
        }
    }
    return escaped;
}
} // namespace

std::string CRsCDKey::Database::Row::GetString(std::string_view name) const
{
    const auto it = FindColumn(columns, name);
    if (it == columns.end()) {
        return {};
    }

    return it->second;
}

long CRsCDKey::Database::Row::GetLong(std::string_view name) const
{
    const std::string value = GetString(name);
    if (value.empty()) {
        return 0;
    }

    return std::strtol(value.c_str(), nullptr, 0);
}

double CRsCDKey::Database::Row::GetDouble(std::string_view name) const
{
    const std::string value = GetString(name);
    if (value.empty()) {
        return 0.0;
    }

    return std::strtod(value.c_str(), nullptr);
}

std::vector<std::uint8_t> CRsCDKey::Database::Row::GetBlob(std::string_view name) const
{
    const auto blob = FindColumn(blobs, name);
    if (blob != blobs.end()) {
        return blob->second;
    }

    const auto text = GetString(name);
    return std::vector<std::uint8_t>(text.begin(), text.end());
}

CRsCDKey::CRsCDKey() = default;

CRsCDKey::~CRsCDKey() = default;

void CRsCDKey::Configure(Setup setup)
{
    g_setup = setup;
}

void CRsCDKey::SetDatabase(Database* database)
{
    g_database = database;
}

bool CRsCDKey::IPIsAllowed(std::uint32_t dwIP)
{
    if (!g_setup.bCheckAllowIP) {
        return true;
    }

    try {
        const auto queryIP = ReverseIPv4Dword(dwIP);
        return CountQueryMatches(FormatString(
            "SELECT count(*) as exp1 FROM ip_allow where 0x%x >= int_begin and 0x%x <= int_end",
            queryIP,
            queryIP));
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки разрешённых IP", error);
        }
        return false;
    }
}

bool CRsCDKey::IPIsForbidded(std::uint32_t dwIP)
{
    if (!g_setup.bCheckForbidIP) {
        return false;
    }

    try {
        const auto queryIP = ReverseIPv4Dword(dwIP);
        return CountQueryMatches(FormatString(
            "SELECT count(*) as exp1 FROM ip_forbid where 0x%x >= int_begin and 0x%x <= int_end",
            queryIP,
            queryIP));
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки запрещённых IP", error);
        }
        return false;
    }
}

bool CRsCDKey::IsBetweenIP(const char* szCdkey, std::uint32_t dwIP)
{
    if (!g_setup.bCheckBetweenIP) {
        return true;
    }

    if (szCdkey == nullptr || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return false;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(
            FormatString("SELECT * FROM ip_list WHERE cdkey='%s'", account.c_str()));
        // VERIFIED_DISASSEMBLY: исходный LoginServer сводит и EOF, и
        // найденный диапазон к true. Запрос и чтение строк сохраняются ради
        // ошибки БД, но фактическая фильтрация результата отсутствовала.
        for (const auto& row : rows) {
            static_cast<void>(row.GetString("ip_begin"));
            static_cast<void>(row.GetString("ip_end"));
        }
        static_cast<void>(dwIP);
        return true;
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки IP-списка аккаунта", error);
        }
    }

    return false;
}

bool CRsCDKey::matrix_used(const char* szCdkey)
{
    if (szCdkey == nullptr || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return false;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(FormatString(
            "SELECT matrix_card FROM csl_cdkey WHERE cdkey='%s' AND matrix_date > GETDATE() AND matrix_card IS NOT NULL",
            account.c_str()));
        return !rows.empty();
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки наличия matrix-карты", error);
        }
        return false;
    }
}

bool CRsCDKey::matrix_validate(const char* szCdkey, const std::uint8_t* pPos,
                              const std::uint8_t* pCode)
{
    if (szCdkey == nullptr || pPos == nullptr || pCode == nullptr || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return false;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(FormatString(
            "SELECT matrix_card FROM csl_cdkey WHERE cdkey='%s' AND matrix_date > GETDATE() AND matrix_card IS NOT NULL",
            account.c_str()));
        if (rows.empty()) {
            return false;
        }

        const auto matrixCard = rows.front().GetBlob("matrix_card");
        std::array<std::uint8_t, kSecurityValidatePosNum> expected = {0, 0, 0};
        for (int index = 0; index < kSecurityValidatePosNum; ++index) {
            if (pPos[index] >= matrixCard.size()) {
                return false;
            }
            expected[static_cast<std::size_t>(index)] = matrixCard[pPos[index]];
        }

        return std::memcmp(expected.data(), pCode, expected.size()) == 0;
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки matrix-ответа", error);
        }
        return false;
    }
}

bool CRsCDKey::GetSecurityState(const char* szCdkey, double* pBanTime,
                                bool* pMatrixUsed)
{
    if (pBanTime == nullptr || pMatrixUsed == nullptr) {
        return false;
    }

    *pBanTime = 0.0;
    *pMatrixUsed = false;
    if (szCdkey == nullptr || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return false;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(FormatString(
            "SELECT CAST(CAST(ban_time AS datetime) AS float)+2.0 AS ban_time,"
            "CASE WHEN matrix_date > GETDATE() AND matrix_card IS NOT NULL "
            "THEN 1 ELSE 0 END AS matrix_used FROM CSL_CDKEY WHERE cdkey='%s'",
            account.c_str()));
        if (!rows.empty()) {
            *pBanTime = rows.front().GetDouble("ban_time");
            *pMatrixUsed = rows.front().GetLong("matrix_used") != 0;
        }
        return true;
    } catch (const std::exception& error) {
        g_database->PrintErr("GetSecurityState: Ошибка чтения состояния безопасности", error);
        return false;
    }
}

void CRsCDKey::GetBanTime(const char* szCdkey, double* pBanTime)
{
    if (pBanTime == nullptr) {
        return;
    }

    *pBanTime = 0.0;
    if (szCdkey == nullptr || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(FormatString(
            "SELECT cdkey,CAST(CAST(ban_time AS datetime) AS float)+2.0 AS ban_time "
            "FROM CSL_CDKEY WHERE cdkey='%s' AND ban_time IS NOT NULL",
            account.c_str()));
        if (!rows.empty()) {
            *pBanTime = rows.front().GetDouble("ban_time");
        }
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("Ошибка проверки блокировки CDKey", error);
        }
    }
}

void CRsCDKey::FixPtAcc(char* szCdkey, std::size_t capacity)
{
    if (szCdkey == nullptr || capacity == 0 || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return;
    }

    try {
        const std::string_view account(szCdkey);
        if (!std::all_of(account.begin(), account.end(), [](unsigned char ch) {
                return std::isdigit(ch) != 0;
            })) {
            return;
        }
        const auto rows = g_database->Query(
            FormatString("SELECT userid FROM userinfo WHERE originsdid=%s", szCdkey));
        if (!rows.empty()) {
            const std::string userId = rows.front().GetString("userid");
            if (IsBoundedValue(userId, kCdkeyMaxBytes)) {
                CopyStringToOutput(szCdkey, capacity, userId);
            }
        }
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("FixPtAcc: Ошибка получения связанного аккаунта", error);
        }
    }
}

bool CRsCDKey::ValidateLocalPassord(const char* szCdkey, const char* szPassword,
                                   char* szOutCdkey, std::size_t outputCapacity)
{
    if (szCdkey == nullptr || szPassword == nullptr || szOutCdkey == nullptr ||
        outputCapacity == 0 || g_database == nullptr ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes) ||
        !IsBoundedValue(szPassword, 50, true)) {
        return false;
    }

    bool originSdid = true;
    for (const unsigned char* cursor = reinterpret_cast<const unsigned char*>(szCdkey);
         *cursor != 0; ++cursor) {
        if (std::isdigit(*cursor) == 0) {
            originSdid = false;
            break;
        }
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto rows = g_database->Query(originSdid
                                                ? FormatString(
                                                      "SELECT CAST(passwd AS VARCHAR(50)) AS pwd,userid,originsdid,passwd FROM userinfo WITH(NOLOCK) WHERE originsdid='%s';",
                                                      account.c_str())
                                                : FormatString(
                                                      "SELECT CAST(passwd AS VARCHAR(50)) AS pwd,userid,passwd FROM userinfo WITH(NOLOCK) WHERE userid='%s';",
                                                      account.c_str()));
        if (rows.empty()) {
            return false;
        }

        const auto& row = rows.front();
        const bool valid = AsciiEqualsIgnoreCase(row.GetString("pwd"), szPassword);
        const std::string output = originSdid ? row.GetString("userid") : std::string(szCdkey);
        return valid && IsBoundedValue(output, kCdkeyMaxBytes) &&
               CopyStringToOutput(szOutCdkey, outputCapacity, output);
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("ValidateLocalPassword: Ошибка проверки пароля", error);
        }
        return false;
    }
}

bool CRsCDKey::CDKeyBan(const char* szCdkey, long lMinute)
{
    if (szCdkey == nullptr || g_database == nullptr || lMinute == 0 ||
        !IsBoundedValue(szCdkey, kCdkeyMaxBytes)) {
        return false;
    }

    try {
        const std::string account = EscapeSqlLiteral(szCdkey);
        const auto existing = g_database->Query(FormatString(
            "SELECT cdkey FROM csl_cdkey WITH(NOLOCK) WHERE cdkey='%s'",
            account.c_str()));
        const bool inserted = existing.empty();
        const std::string sql = inserted
            ? FormatString(
                  "INSERT INTO csl_cdkey(cdkey,password,ban_time) "
                  "VALUES('%s','',DATEADD(minute,%ld,GETDATE()))",
                  account.c_str(), lMinute)
            : FormatString(
                  "UPDATE CSL_CDKEY SET ban_time=DATEADD(minute,%ld,GETDATE()) "
                  "WHERE cdkey='%s'",
                  lMinute, account.c_str());
        if (!g_database->Execute(sql)) {
            const std::runtime_error error("Не удалось выполнить команду БД");
            g_database->PrintErr("CDKeyBan: Ошибка изменения блокировки", error);
            return false;
        }

        std::string log =
            FormatString("Аккаунт %s заблокирован на %d минут.", szCdkey, static_cast<int>(lMinute));
        log += inserted ? " <добавление>" : " <обновление>";
        g_database->PutStringToFile("banplayer", log);
        return true;
    } catch (const std::exception& error) {
        if (g_database != nullptr) {
            g_database->PrintErr("CDKeyBan: Ошибка изменения блокировки", error);
        }
        return false;
    }
}
