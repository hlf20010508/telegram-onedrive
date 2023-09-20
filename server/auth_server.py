"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from flask import Flask, render_template, request, jsonify
import os

app = Flask(__name__)

code_tg = ''
code_od = ''


@app.route("/")
def telegram_code_index():
    return render_template("tg_code.html")


@app.route("/tg", methods=["GET", "POST"])
def telegram_code():
    global code_tg
    if request.method == "POST":
        code_tg = request.json["code"]
        return jsonify({"success": True})
    if request.method == "GET":
        if not code_tg:
            return jsonify({"success": False})
        else:
            return jsonify({"success": True, "code": code_tg})


@app.route("/auth")
def onedrive_code():
    global code_od
    if not request.args.get("get"):
        code_od = request.args.get("code")
        return "Authorization Successful!"
    else:
        if not code_od:
            return jsonify({"success": False})
        else:
            return jsonify({"success": True, "code": code_od})


if __name__ == "__main__":
    server_uri = os.environ["server_uri"]
    port = int(server_uri.split(':')[-1].split('/')[0])
    app.run(host="0.0.0.0", port=port, ssl_context=("server/ssl/server.crt", "server/ssl/server.key"))
