"""
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2023 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
"""

from flask import Flask, render_template, request, jsonify
import os

app = Flask(__name__)

temp_dir = "temp"

if not os.path.exists(temp_dir):
    os.mkdir(temp_dir)


@app.route("/")
def telegram_code_index():
    return render_template("tg_code.html")


@app.route("/tg", methods=["GET", "POST"])
def telegram_code():
    code_path = os.path.join(temp_dir, "tg_code")
    if request.method == "POST":
        code = request.json["code"]
        with open(code_path, "w") as file:
            file.write(code)
        return jsonify({"success": True})
    if request.method == "GET":
        if not os.path.exists(code_path):
            return jsonify({"success": False})
        else:
            code = ""
            with open(code_path, "r") as file:
                code = file.read()
            os.remove(code_path)
            return jsonify({"success": True, "code": code})


@app.route("/auth")
def onedrive_code():
    code_path = os.path.join(temp_dir, "od_code")
    if not request.args.get("get"):
        code = request.args.get("code")
        with open(code_path, "w") as file:
            file.write(code)
        return "Authorization Successful!"
    else:
        if not os.path.exists(code_path):
            return jsonify({"success": False})
        else:
            code = ""
            with open(code_path, "r") as file:
                code = file.read()
            os.remove(code_path)
            return jsonify({"success": True, "code": code})


if __name__ == "__main__":
    server_uri = os.environ["server_uri"]
    port = int(server_uri.split(':')[-1].split('/')[0])
    app.run(host="0.0.0.0", port=port, ssl_context=("ssl/server.crt", "ssl/server.key"))
