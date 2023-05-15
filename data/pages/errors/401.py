from lib.scripting_utils import *

request = Interface.parse_incoming_request()
if request:
    with open(f'data/pages/errors/401.json') as f: body = f.read()
    headers = {"Location":"/login","Set-Cookie":"LoginRedirect={}; Path=/login; HttpOnly; Max-Age=60".format(request["path"])}
    Interface.send_to_http(headers, body)
else:
    Interface.send_to_http({},"")