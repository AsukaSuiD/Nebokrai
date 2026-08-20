#include "setup.h"

#include <charconv>
#include <fstream>
#include <vector>

namespace Misc
{
namespace
{
std::optional<std::uint16_t> ParsePort(const std::string& text)
{
    unsigned int value{};
    const auto result = std::from_chars(text.data(), text.data() + text.size(), value);
    if (result.ec != std::errc{} || result.ptr != text.data() + text.size() ||
        value > 0xFFFFU) {
        return std::nullopt;
    }
    return static_cast<std::uint16_t>(value);
}
}

SetupLoadResult CSetup::LoadSetup(const std::filesystem::path& path)
{
    std::ifstream input(path);
    if (!input) return {};

    std::vector<std::string> tokens;
    for (std::string token; input >> token;) tokens.push_back(std::move(token));
    SetupLoadResult result{.opened = true};
    auto value = [&](std::size_t pair) -> const std::string* {
        const std::size_t index = pair * 2U + 1U;
        if (index >= tokens.size()) {
            result.stoppedAtPair = pair + 1U;
            return nullptr;
        }
        return &tokens[index];
    };

    const std::string* worldIp = value(0);
    if (!worldIp) return result;
    m_IpPort.worldIp = *worldIp;
    ++result.parsedPairs;

    const std::string* worldPort = value(1);
    if (!worldPort) return result;
    m_IpPort.worldPort = ParsePort(*worldPort);
    if (!m_IpPort.worldPort) {
        result.stoppedAtPair = 2;
        return result;
    }
    ++result.parsedPairs;

    const std::string* localIp = value(2);
    if (!localIp) return result;
    m_IpPort.localIp = *localIp;
    ++result.parsedPairs;

    const std::string* listenPort = value(3);
    if (!listenPort) return result;
    m_IpPort.listenPort = ParsePort(*listenPort);
    if (!m_IpPort.listenPort) {
        result.stoppedAtPair = 4;
        return result;
    }
    ++result.parsedPairs;
    return result;
}

const IpPortSetup& CSetup::IpPort() const noexcept { return m_IpPort; }
}
