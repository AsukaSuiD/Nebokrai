#include "billing/billingserver.h"

#include <spdlog/spdlog.h>

#include <csignal>
#include <cstdlib>
#include <filesystem>

/*
 * Техническая Linux-точка входа отдельного BillingServer-процесса.
 * WinMain/GameThreadFunc заменены main и POSIX-сигналами; исходный lifecycle
 * находится в billing/game.cpp, а не дублируется здесь.
 */
namespace
{
volatile std::sig_atomic_t g_StopRequested{};
void RequestStop(int) { g_StopRequested = 1; }
}

int main(int argc, char* argv[])
{
    std::signal(SIGINT, RequestStop);
    std::signal(SIGTERM, RequestStop);
    const std::filesystem::path runtimeDirectory =
        std::filesystem::absolute(argc > 1 ? std::filesystem::path(argv[1])
                                           : std::filesystem::current_path());

    Billing::BillingServer server;
    if (const Billing::BillingGameResult result = server.Initialize(runtimeDirectory);
        !result) {
        spdlog::error("BillingServer: инициализация остановлена: {}", result.detail);
        static_cast<void>(server.Release());
        return EXIT_FAILURE;
    }

    bool clean = true;
    while (g_StopRequested == 0) {
        const Billing::BillingGameResult turn = server.RunTurn();
        if (!turn) {
            spdlog::error("BillingServer: main-loop остановлен: {}", turn.detail);
            clean = false;
            break;
        }
    }
    if (const Billing::BillingGameResult result = server.Release(); !result) {
        spdlog::error("BillingServer: ошибка завершения: {}", result.detail);
        clean = false;
    }
    return clean ? EXIT_SUCCESS : EXIT_FAILURE;
}
