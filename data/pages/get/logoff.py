from lib.scripting_utils import *

session_id = Interface.parse_input()["cookies"]["sessionID"]
Database().update("users","sessionID= '{}'".format(session_id),{"sessionExpires":0})
response_headers = {"Location":"/"}
Interface.send_to_http(response_headers, "")