use std::future::Future;
use std::sync::mpsc;

use super::state::AsyncResult;

/// Spawn an async task from the synchronous egui update() context.
pub fn spawn_async<F, Fut>(
    handle: &tokio::runtime::Handle,
    tx: &mpsc::Sender<AsyncResult>,
    ctx: &eframe::egui::Context,
    f: F,
) where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = AsyncResult> + Send + 'static,
{
    let tx = tx.clone();
    let ctx = ctx.clone();
    handle.spawn(async move {
        let result = f().await;
        let _ = tx.send(result);
        ctx.request_repaint();
    });
}
