/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use tracing::field::{Field, Visit};

#[derive(Default)]
pub struct MessageVisitor {
    pub message: String,
}

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message.push_str(&format!("{:?}", value));
        }
    }
}

// events that converted from log crate don't convert meta correctly
// so manually get it
#[derive(Default)]
pub struct MetaVisitor {
    pub target: String,
    pub module_path: String,
    pub file: String,
    pub line: String,
}

impl Visit for MetaVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "log.target" => self.target.push_str(value),
            "log.module_path" => self.module_path.push_str(value),
            "log.file" => self.file.push_str(value),
            "log.line" => self.line.push_str(value),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "log.target" => self.target.push_str(&format!("{:?}", value)),
            "log.module_path" => self.module_path.push_str(&format!("{:?}", value)),
            "log.file" => self.file.push_str(&format!("{:?}", value)),
            "log.line" => self.line.push_str(&format!("{:?}", value)),
            _ => {}
        }
    }
}
