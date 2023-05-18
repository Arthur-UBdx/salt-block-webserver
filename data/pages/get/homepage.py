from lib.scripting_utils import *
from jinja2 import Template 

request = Interface.parse_incoming_request()
cookies: dict = request["cookies"]

def send_unauth():
    page_header:str = Interface.read_file("data/pages/get/header_unauth.html")
    page:Template = Template(Interface.read_file("data/pages/get/homepage.html"))
    body = page.render(header=page_header)
    Interface.send_to_http(200, "OK", {}, body)
    
if "sessionID" not in list(cookies.keys()): send_unauth()
print("hello")