from lib.scripting_utils import *
import hashlib
import time
from jinja2 import Template
import os

response_headers:dict = {}

config = Config()
database = Database()
body:dict = Interface.parse_body_query()

row = database.get_row("users", "username" , body["username"])
if not row:
    with open("data/pages/get/login_error.html") as f:
        template:Template = Template(f.read())
        
    html_content = template.render(error_message="Your username or password is incorrect")
    print(html_content)
    sys.exit()
    
now:int = int(time.time())
message = "{}{}".format(body["username"], body["password"])
input_hash = hashlib.sha256(message.encode()).hexdigest()
correct_hash = row["hash"]

if input_hash == correct_hash :
    session_id = hashlib.sha256("{}{}".format(row["username"], now).encode()).hexdigest()
    session_expires = now + config.get("session_expiration_time")
    database.update("users","username= '{}'".format(row["username"]),{"sessionID":session_id, "sessionExpires":session_expires})
    
    response_headers["Location"] = "/api/time"
    response_headers["Set-Cookie"] = "sessionID={}; Max-Age={}; HttpOnly".format(session_id, session_expires)
    response = Interface.export_to_http(response_headers, "")
    print(response)
else:
    with open("data/pages/get/login_error.html") as f:
        template:Template = Template(f.read())
        
    html_content = template.render(error_message="Your username or password is incorrect")
    print(html_content)
    sys.exit()