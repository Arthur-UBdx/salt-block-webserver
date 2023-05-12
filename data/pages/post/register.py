from lib.scripting_utils import *
import time
import re
from jinja2 import Template

def send_error(message:str):
    with open("data/pages/get/register_error.html") as f:
        template:Template = Template(f.read())

    html_content = template.render(error_message=message)
    Interface.send_to_http({}, html_content)
    
database = Database()
body = Interface.parse_body_query()

if body["password"] != body["cpwd"]:
    send_error("The passwords were differents")
    
if len(body["password"]) < 8:
    send_error("The password is too short, try at least 8 characters")

email_check_pattern:str = r"^[a-zA-Z0-9._%+-]+%40[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$" #merci ChatGPT
if not re.match(email_check_pattern, body["email"]):
    send_error("The email is in a wrong format")

if database.get_row("users", "username", body["username"]):
    send_error("This user already exists")
    
if database.get_row("users", "email", body["email"]):
    send_error("This e-mail is already taken")

now:int = int(time.time())
session_id:str = hashlib.sha256("{}{}".format(body["username"], now).encode()).hexdigest()
session_expiration_time = Config().get("session_expiration_time")
session_expires = now + session_expiration_time

pwdhash = Interface.sha256("{}{}".format(body["username"], body["password"]))
database.insert("users", {"username":body["username"], "hash":pwdhash, "credits":0, "auth_level":1, "email":body["email"]})
Interface.send_to_http({"Location":"/", "Set-Cookie": "sessionID={}; Max-Age={}; HttpOnly".format(session_id, session_expiration_time)}, "")