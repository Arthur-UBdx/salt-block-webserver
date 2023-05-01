import time

result = '''{
    "status":"success",
    "status_code":"200",
    "message":"OK",
    "result":[{
        "time": "''' + f"{time.time()}" +'''"
    }]
}'''

print(result)