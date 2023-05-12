from lib.scripting_utils import *
from jinja2 import Template

database = Database()
request = Interface.parse_input()

try:
    session_id = request["cookies"]["sessionID"]
except KeyError:
    session_id = None

if session_id: auth = database.auth_user(session_id)
else: auth = None

if not auth:
    with open("data/pages/get/login.html") as f:
        template:Template = Template(f.read())
        
    html_content = template.render(error_message=f"")
    Interface.send_to_http({}, html_content)
    
else:
    try:
        Interface.send_to_http({"Location": request["cookies"]["LoginRedirect"]}, "")
    except:
        Interface.send_to_http({"Location": "/"}, "")
    