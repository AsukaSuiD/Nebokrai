#include "gasoperator.h"

#include <new>
#include <string>

namespace Login
{
std::mutex CGasOperator::s_InstanceMutex;
std::unique_ptr<CGasOperator> CGasOperator::s_Instance;

CGasOperator* CGasOperator::GetInstance() noexcept
{
    std::lock_guard guard(s_InstanceMutex);
    if (!s_Instance) {
        s_Instance.reset(new (std::nothrow) CGasOperator());
    }
    return s_Instance.get();
}

void CGasOperator::ReleaseInstance() noexcept
{
    std::lock_guard guard(s_InstanceMutex);
    s_Instance.reset();
}

std::string CGasOperator::GetIP(std::uint32_t ip) const
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00421500..0x0042157F: sprintf("%u.%u.%u.%u")
    // получает байты raw IPv4 именно от младшего к старшему.
    return std::to_string(ip & 0xFFU) + "." +
           std::to_string((ip >> 8U) & 0xFFU) + "." +
           std::to_string((ip >> 16U) & 0xFFU) + "." +
           std::to_string((ip >> 24U) & 0xFFU);
}
}
