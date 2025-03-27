use once_cell::sync::OnceCell;

use crate::DelegateManager;

pub static SINGLETON_DELEGATE_MANAGER: OnceCell<DelegateManager> = OnceCell::new();

pub fn init() {
    match SINGLETON_DELEGATE_MANAGER.set(DelegateManager::default()) {
        Ok(_) => log::info!("Singleton Delegate Manager is successfully initialized!"),
        Err(_) => log::warn!("Singel Delegate Manager is already initialized!"),
    }
}

pub fn get_delegate_manager() -> &'static DelegateManager {
    SINGLETON_DELEGATE_MANAGER
        .get()
        .expect("Singleton Delegate Manager is not initialized yet! Make sure to call `delegate_rs::init()`")
}
