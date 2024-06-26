/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use futures::future::BoxFuture;
use futures::FutureExt;
use grammers_client::types::Message;
use std::sync::Arc;

use crate::client::telegram_bot::TelegramBotClient;
use crate::error::Result;

pub trait Extractor: Sized + Send + Sync {
    fn extract(
        message: Message,
        client: Arc<TelegramBotClient>,
    ) -> BoxFuture<'static, Result<Self>>;
}

pub trait Handler<Args> {
    fn handle(
        &'static self,
        message: Message,
        client: Arc<TelegramBotClient>,
    ) -> BoxFuture<'static, Result<()>>;
}

macro_rules! impl_handler {
    ($($name:ident),*) => {
        impl<F, $($name),*> Handler<($($name),*,)> for F
        where
            F: Fn($($name),*) -> BoxFuture<'static, Result<()>> + Sync,
            $($name: Extractor),*
        {
            fn handle(
                &'static self,
                message: Message,
                client: Arc<TelegramBotClient>,
            ) -> BoxFuture<'static, Result<()>> {
                async move {
                    self(
                        $($name::extract(message.clone(), client.clone()).await?),*
                    ).await
                }.boxed()
            }
        }
    };
}

impl_handler!(A1);
impl_handler!(A1, A2);
impl_handler!(A1, A2, A3);
impl_handler!(A1, A2, A3, A4);
impl_handler!(A1, A2, A3, A4, A5);
impl_handler!(A1, A2, A3, A4, A5, A6);
impl_handler!(A1, A2, A3, A4, A5, A6, A7);
impl_handler!(A1, A2, A3, A4, A5, A6, A7, A8);
impl_handler!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_handler!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
