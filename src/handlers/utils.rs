/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use regex::Regex;
use std::fmt::Display;

pub fn cmd_parser<T>(cmd: T) -> Vec<String>
where
    T: Display,
{
    cmd.to_string()
        .purify()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

pub trait TextExt {
    fn purify(&self) -> String;
}

impl<T> TextExt for T
where
    T: Display,
{
    fn purify(&self) -> String {
        let text = self
            .to_string()
            .trim()
            .replace("*", "")
            .replace("`", "")
            .replace("~", "")
            .replace("<b>", "")
            .replace("</b>", "")
            .replace("<strong>", "")
            .replace("</strong>", "")
            .replace("<i>", "")
            .replace("</i>", "")
            .replace("<em>", "")
            .replace("</em>", "")
            .replace("<code>", "")
            .replace("</code>", "")
            .replace("<s>", "")
            .replace("</s>", "")
            .replace("<strike>", "")
            .replace("</strike>", "")
            .replace("<del>", "")
            .replace("</del>", "")
            .replace("<u>", "")
            .replace("</u>", "")
            .replace("</pre>", "");

        let re = Regex::new(r#"<pre[^>]*>"#).unwrap();
        re.replace_all(&text, "").to_string()
    }
}
