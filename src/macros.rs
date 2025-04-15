#[macro_export]
macro_rules! bind_delegate {
    ($instance:expr, $method:ident) => {
        let instance_clone = ::std::sync::Arc::clone(&$instance);

        ::delegate_rs::get_delegate_manager().bind(
            stringify!($method), 
            move |data| { instance_clone.$method(data)}
        )
    };
}

#[macro_export]
macro_rules! async_bind_delegate {
    ($instance:expr, $method:ident) => {{
        let instance_clone = ::std::sync::Arc::clone(&$instance);

        ::delegate_rs::get_delegate_manager().async_bind(stringify!($method), move |data| {
            // Clone again inside the closure so the async block doesn't borrow instance_clone.
            let instance_inner = instance_clone.clone();
            async move { instance_inner.$method(data).await }
        });
    }};
}

#[macro_export]
macro_rules! broadcast_delegate {
    ($name:expr, $arg:expr) => {
        ::delegate_rs::get_delegate_manager().broadcast::<_, ()>($name, $arg)
    };
    ($name:expr, $arg:expr, $ret_type:ty) => {
        ::delegate_rs::get_delegate_manager().broadcast::<_, $ret_type>($name, $arg)
    };
}

#[macro_export]
macro_rules! async_broadcast_delegate {
    ($delegate_name:expr, $data:expr) => {
        ::delegate_rs::get_delegate_manager()
            .async_broadcast::<_, ()>($delegate_name, $data)
            .await
    };
    ($delegate_name:expr, $data:expr, $ret_type:ty) => {
        ::delegate_rs::get_delegate_manager()
            .async_broadcast::<_, $ret_type>($delegate_name, $data)
            .await
    };
}