//! Технический поток `kl_thread.cpp`, подтверждённый `authserver.exe` и
//! `authserver.pdb`. Управлением владеют конкретные Tokio/std worker-owner-ы:
//! Win32 handle и принудительная остановка не входят во внешний контракт сервиса.
