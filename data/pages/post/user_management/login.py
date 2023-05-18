from lib.scripting_utils import *
import hashlib
import time
from jinja2 import Template
    
def send_error():
    with open("data/pages/get/user_management/login.html") as f:
        template:Template = Template(f.read())
        
    html_content = template.render(error_message=f"Your username or password is incorrect")
    Interface.send_to_http(200, "OK", {}, html_content)

# -- #

response_headers:dict = {}

config = Config()
database = Database()
body:dict = Interface.parse_body_query()
request:dict = Interface.parse_incoming_request()

row = database.get_row("users", "username" , body["username"])
if not row: send_error()

now:int = int(time.time())
message = "{}{}".format(body["username"], body["password"])
input_hash = Interface.sha256(message)
correct_hash = row["hash"]

if not (input_hash == correct_hash): send_error()

try: response_headers["Location"] = request["cookies"]["LoginRedirect"]
except: response_headers["Location"] = "/"

session_id = Interface.sha256("{}{}".format(row["username"], now))
session_expires = now + config.get("session_expiration_time")
database.update("users","username= '{}'".format(row["username"]),{"sessionID":session_id, "sessionExpires":session_expires})

response_headers["Set-Cookie"] = "sessionID={}; Max-Age={}; HttpOnly".format(session_id, config.get("session_expiration_time"))
response = Interface.send_to_http(200, "OK", response_headers, "")