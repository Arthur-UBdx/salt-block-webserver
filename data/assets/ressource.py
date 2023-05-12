from lib.scripting_utils import *;
import json
import base64

def send_error(code:int):
    with open(f"data/pages/errors/{code}.json") as f:
        body = f.read()
    Interface.send_to_http({}, body)
    
def send_image(filename:str):
    accepted_images_format = ["png", "jpg", "gif", "bmp"]
    extension = filename.split(".")[-1]
    if filename == extension: send_error(404)
    if not (extension in accepted_images_format): send_error(404)
    try: 
        with open(filename, "rb") as file:
            image_data:bytes = file.read()
    except FileNotFoundError: send_error(404)
    Interface.send_to_http({"Content-Type":f"image/{extension}"}, image_data) #

accepted_data_types = ["image"]
ressource_type:str = None
request = Interface.parse_input()
query:dict = request["query"]


for type in accepted_data_types:
    if type in list(query.keys()): 
        ressource_type = type
        break

if not ressource_type: send_error(400)
    
requested_file = query[ressource_type]
match ressource_type:
    case "image":
        send_image(f"data/assets/images/{requested_file}")
