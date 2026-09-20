#pragma once

#include <cstdint>
#include <map>
#include <memory>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/rssetup.cpp/.h.
 * ADO/COM заменён общей Linux ODBC-границей Nebokrai. SQL, порядок параметров,
 * транзакционные фазы и интерпретация строк остаются в конкретных WorldDB
 * owner-ах. @P<n> преобразуются в позиционные ODBC-параметры внутри адаптера,
 * включая повторное использование одного параметра в batch SQL.
 */
using WorldDbValue = std::variant<std::monostate, std::int64_t, std::uint64_t,
                                  double, std::string, std::vector<std::uint8_t>>;
using WorldDbRow = std::map<std::string, WorldDbValue, std::less<>>;

struct WorldDbCommand {
    std::string statement;
    std::vector<WorldDbValue> parameters;
};

struct WorldDbResult {
    bool success{};
    std::vector<WorldDbRow> rows;
    std::uint64_t affectedRows{};
    std::string error;
};

class IWorldDbExecutor {
public:
    virtual ~IWorldDbExecutor() = default;
    virtual WorldDbResult Execute(const WorldDbCommand&) = 0;
    virtual bool BeginTransaction() { return false; }
    virtual bool CommitTransaction() { return false; }
    virtual bool RollbackTransaction() { return false; }
};

class OdbcWorldDbExecutor final : public IWorldDbExecutor {
public:
    explicit OdbcWorldDbExecutor(std::string connectionString);
    ~OdbcWorldDbExecutor() override;
    OdbcWorldDbExecutor(const OdbcWorldDbExecutor&) = delete;
    OdbcWorldDbExecutor& operator=(const OdbcWorldDbExecutor&) = delete;

    [[nodiscard]] bool Open();
    [[nodiscard]] bool IsOpen() const noexcept;
    [[nodiscard]] std::string_view LastError() const noexcept;
    WorldDbResult Execute(const WorldDbCommand&) override;
    bool BeginTransaction() override;
    bool CommitTransaction() override;
    bool RollbackTransaction() override;

private:
    struct Impl;
    std::unique_ptr<Impl> m_Impl;
};

class CRsSetup {
public:
    [[nodiscard]] static WorldDbResult Load(IWorldDbExecutor&);
    [[nodiscard]] static WorldDbResult SavePlayerId(IWorldDbExecutor&, std::int32_t id);
    [[nodiscard]] static WorldDbResult SaveLeaveWordId(IWorldDbExecutor&, std::int32_t id);
};
