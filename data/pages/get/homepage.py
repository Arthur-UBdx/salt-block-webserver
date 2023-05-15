from lib.scripting_utils import *
from jinja2 import Template 

request = Interface.parse_incoming_request()
cookies: dict = request["cookies"]
if "sessionID" not in list(cookies.keys()): Interface.send_file({}, "data/pages/get/homepage_unauth.html")
print("hello")