from lib.scripting_utils import *;

with open(f'data/pages/errors/403.json') as f: body = f.read()
headers = {"Location":"/"}
Interface.send_to_http(200, "OK", headers, body)