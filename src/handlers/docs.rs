/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

const GREETING: &str = "\
Transfer files to Onedrive.

Forward or upload files to me, or pass message link to transfer restricted content from group or channel.

- /help: Ask for help.
";

const HELP_BASE: &str = "\
<pre><code>/auth</code></pre>
To authorize for Telegram and OneDrive.
<pre><code>/clear</code></pre>
To clear all history.
<pre><code>/autoDelete</code></pre>
To toggle whether bot should auto delete message.
";

const HELP_LINKS: &str = "\
<pre><code>/links $message_link $num</code></pre>
To transfer sequential restricted content.
<pre><code>/links help</code></pre>
To show command help.
";

const HELP_URL: &str = "\
<pre><code>/url $url</code></pre>
To upload file through url.
<pre><code>/url help</code></pre>
To show command help.
";

const HELP_LOGS: &str = "\
<pre><code>/logs</code></pre>
To send logs zip.
<pre><code>/logs clear</code></pre>
To clear logs.
<pre><code>/logs help</code></pre>
To show command help.
";

const HELP_DRIVE: &str = "\
<pre><code>/drive</code></pre>
To list all OneDrive accounts.
<pre><code>/drive add</code></pre>
To add a OneDrive account.
<pre><code>/drive $index</code></pre>
To change the OneDrive account.
<pre><code>/drive logout</code></pre>
To logout current OneDrive account.
<pre><code>/drive logout $index</code></pre>
To logout specified OneDrive account.
<pre><code>/drive help</code></pre>
To show command help.
";

const HELP_DIR: &str = "\
<pre><code>/dir</code></pre>
To show current OneDrive directory.
<pre><code>/dir $path</code></pre>
To set OneDrive directory.
<pre><code>/dir temp $path</code></pre>
To set temporary OneDrive directory.
<pre><code>/dir temp cancel</code></pre>
To restore OneDrive directory to the previous one.
<pre><code>/dir reset</code></pre>
To reset OneDrive directory to default.
<pre><code>/dir help</code></pre>
To show command help.
";

const INSTRUCTION: &str = "\
- To transfer files, forward or upload to me.
- To transfer restricted content, right click the content, copy the message link, and send to me.
- Tap the file name on the Progress message to locate the job.
- To upload files through url, the headers of the file response must includes Content-Length.
- To cancel a job, delete the related message.
- Support files with extension .t2o as scripts.

See <a href=\"https://github.com/hlf20010508/telegram-onedrive#example\">example</a>.
";

pub fn format_unknown_command_help(name: &str) -> String {
    format!(
        "Unknown command for {}\n\nUsage:\n{}",
        name,
        format_help(name)
    )
}

pub fn format_help(name: &str) -> String {
    match name {
        "/help" => {
            format!(
                "{}{}{}{}{}{}\n{}",
                HELP_BASE, HELP_LINKS, HELP_URL, HELP_LOGS, HELP_DRIVE, HELP_DIR, INSTRUCTION
            )
        }
        "/start" => GREETING.to_string(),
        "/links" => HELP_LINKS.to_string(),
        "/url" => HELP_URL.to_string(),
        "/logs" => HELP_LOGS.to_string(),
        "/drive" => HELP_DRIVE.to_string(),
        "/dir" => HELP_DIR.to_string(),
        _ => String::new(),
    }
}
