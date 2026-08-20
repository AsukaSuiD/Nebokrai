#pragma once

#include <cstdint>
#include <cstddef>
#include <filesystem>
#include <optional>
#include <string>

namespace Misc
{
struct IpPortSetup
{
    std::optional<std::string> worldIp;
    std::optional<std::uint16_t> worldPort;
    std::optional<std::string> localIp;
    std::optional<std::uint16_t> listenPort;
};

struct SetupLoadResult
{
    bool opened{};
    std::size_t parsedPairs{};
    std::optional<std::size_t> stoppedAtPair;
};

/*
 * Исходный владелец: miscserver/setup/setup.cpp / .h.
 * setup.ini содержит четыре позиционные пары label/value; сами labels исходник
 * не проверял. Поздний parse-fail сохраняет уже прочитанный prefix.
 */
class CSetup
{
public:
    [[nodiscard]] SetupLoadResult LoadSetup(const std::filesystem::path& path);
    [[nodiscard]] const IpPortSetup& IpPort() const noexcept;

private:
    IpPortSetup m_IpPort;
};
}
