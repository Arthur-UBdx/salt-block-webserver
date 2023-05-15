from lib.scripting_utils import *
import json
import base64

def send_error(code:int):
    with open(f"data/pages/errors/{code}.json") as f:
        body = f.read()
    Interface.send_to_http({}, body)
    
def read_file(filename) -> bytes:
    try: 
        with open(filename, "rb") as file: return file.read()
    except: return None
    
def send_ressource(filename:str, accepted_extensions:list, headers:dict):
    extension = filename.split(".")[-1]
    if filename == extension: send_error(400)
    if not (extension in accepted_extensions): send_error(400)
    data = read_file(filename)
    if not data: send_error(404)
    Interface.send_to_http(headers, data)

def send_avatar(username:str): #NOPROD
    Interface.send_to_http({"Location":f"https://mc-heads.net/avatar/{username}"}, "")
    
# -- #
    
accepted_data_types = ["image", "file", "avatar", "css"] # "avatar" à supprimer dans la version de production #NOPROD
ressource_type:str = None 
request = Interface.parse_incoming_request()
query:dict = request["query"]

for type in accepted_data_types:
    if type in list(query.keys()): 
        ressource_type = type
        break

if not ressource_type: send_error(400)
    
requested_file:str = query[ressource_type]
match ressource_type:
    case "image": send_ressource(
        f"data/assets/images/{requested_file}",
        ["png", "jpg", "gif", "bmp"],
        {"Content-Type":f"image/{requested_file.split('.')[-1]}"})
        
    case "file": send_ressource(
        f"data/assets/images/{requested_file}",
        ["zip", "jar"],
        {"Content-Type":f"application/{requested_file.split('.')[-1]}", "Content-Disposition":f"attachement; filename=\"{requested_file.split('/')[-1]}\""})
        
    case "avatar": send_avatar(requested_file) # à supprimer dans la version de production  #NOPROD

    case "css": send_ressource(
        f"data/assets/css/{requested_file}",
        ["css"],
        {"Content-Type":"text/css"})
