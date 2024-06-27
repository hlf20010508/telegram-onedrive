/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

pub fn trace_registor(state: AppState) {
    let file_appender = tracing_appender::rolling::never(".", "log.txt");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    let file_layer = fmt::layer().with_writer(file_writer);

    // let telegram_layer = fmt::layer().with_writer(|| {
    //     let client = &state.telegram_bot.client;
    //     client.send_message(chat, message)
    // });

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        // .with(custom_layer)
        .with(
            EnvFilter::from_default_env()
                .add_directive(format!("telegram_onedrive={}", "debug").parse().unwrap()),
        )
        .init();
}
