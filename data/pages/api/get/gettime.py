from lib.scripting_utils import *
import time

result = '''{
    "status":"success",
    "status_code":"200",
    "message":"OK",
    "result":[{
        "time": "''' + f"{time.time()}" +'''"
    }]
}'''

Interface.send_to_http(200, "OK", {}, result)