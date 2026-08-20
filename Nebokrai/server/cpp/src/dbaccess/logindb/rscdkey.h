#pragma once

#include <cstdint>
#include <exception>
#include <map>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: dbaccess/logindb/rscdkey.cpp / rscdkey.h.
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb;
 * исходный путь PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\dbaccess\\logindb\\rscdkey.cpp.
 *
 * Сохранены достигнутые операции CDKey/IP/matrix/local-password и исходный
 * отдельный DB-сеанс на каждый вызов. ADO/COM заменён абстракцией Database и
 * Linux ODBC-адаптером. Существенные уточнения поздней реконструкции находятся
 * у соответствующих тел: IsBetweenIP сохраняет фактический no-op дефект,
 * FixPtAcc принимает только цифровой originsdid, CDKeyBan сохраняет исходный
 * нетранзакционный SELECT WITH(NOLOCK) -> UPDATE/INSERT.
 *
 * Неизвестная реакция оригинала на matrix_card короче запрошенной позиции в
 * безопасной C++-форме даёт false: небезопасное чтение за blob не повторяется.
 */

class CRsCDKey
{
public:
    struct Setup
    {
        bool bCheckForbidIP = false;
        bool bCheckAllowIP = false;
        bool bCheckBetweenIP = false;
    };

    class Database
    {
    public:
        struct Row
        {
            std::map<std::string, std::string> columns;
            std::map<std::string, std::vector<std::uint8_t>> blobs;

            [[nodiscard]] std::string GetString(std::string_view name) const;
            [[nodiscard]] long GetLong(std::string_view name) const;
            [[nodiscard]] double GetDouble(std::string_view name) const;
            [[nodiscard]] std::vector<std::uint8_t> GetBlob(std::string_view name) const;
        };

        virtual ~Database() = default;

        [[nodiscard]] virtual std::vector<Row> Query(const std::string& sql) = 0;
        [[nodiscard]] virtual bool Execute(const std::string& sql) = 0;
        virtual void PutStringToFile(const std::string&, const std::string&) {}
        virtual void PrintErr(const std::string&, const std::exception&) {}
    };

    CRsCDKey();
    virtual ~CRsCDKey();

    static void Configure(Setup setup);
    static void SetDatabase(Database* database);

    bool IPIsAllowed(std::uint32_t dwIP);
    bool IPIsForbidded(std::uint32_t dwIP);
    bool IsBetweenIP(const char* szCdkey, std::uint32_t dwIP);
    bool matrix_used(const char* szCdkey);
    bool matrix_validate(const char* szCdkey, const std::uint8_t* pPos,
                         const std::uint8_t* pCode);
    bool GetSecurityState(const char* szCdkey, double* pBanTime, bool* pMatrixUsed);
    void GetBanTime(const char* szCdkey, double* pBanTime);
    void FixPtAcc(char* szCdkey, std::size_t capacity);
    bool ValidateLocalPassord(const char* szCdkey, const char* szPassword,
                              char* szOutCdkey, std::size_t outputCapacity);
    bool CDKeyBan(const char* szCdkey, long lMinute);
};
