/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub const GREETING: &str = r#"
- /auth to authorize for Telegram and OneDrive.
- /clear to clear all history except status message.
- /autoDelete to toggle whether bot should auto delete message.
- /logs to send log file.
- /drive to list all OneDrive accounts.
- /dir to show current OneDrive directory.

<pre><code>/links $message_link $range</code></pre>
To transfer sequential restricted content.
<pre><code>/url $file_url</code></pre>
To upload file through url.
<pre><code>/logs clear</code></pre>
To clear logs.
<pre><code>/drive add</code></pre>
To add a OneDrive account.
<pre><code>/drive $index</code></pre>
To change the OneDrive account.
<pre><code>/drive logout</code></pre>
To logout current OneDrive account.
<pre><code>/drive logout $index</code></pre>
To logout specified OneDrive account.
<pre><code>/dir $remote_path</code></pre>
To set OneDrive directory.
<pre><code>/dir temp $remote_path</code></pre>
To set temporary OneDrive directory.
<pre><code>/dir temp cancel</code></pre>
To restore OneDrive directory to the previous one.
<pre><code>/dir reset</code></pre>
To reset OneDrive directory to default.

- To transfer files, forward or upload to me.
- To transfer restricted content, right click the content, copy the message link, and send to me.
- Tap Status on replied status message to locate current job.
- To upload files through url, the headers of the file response must includes Content-Length.
- Each log page contains 50 lines of logs.
- Support files with extension .t2o as scripts.

See <a href="https://github.com/hlf20010508/telegram-onedrive#example">example</a>.
"#;
