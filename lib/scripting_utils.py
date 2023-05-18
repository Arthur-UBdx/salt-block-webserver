import sqlite3 as sql
import json
import os
import sys
import hashlib
from time import time

DIR = os.path.abspath(os.curdir)

class Database:
    def __init__(self):
        self.filepath = Config().get("database")
        
    def insert(self,table,kwargs):
        with sql.connect(self.filepath) as conn:
            request = f'''INSERT INTO {table} ({','.join(kwargs.keys())}) VALUES ({','.join(['?' for _ in kwargs])})'''
            request_values = tuple(kwargs.values())
            conn.cursor().execute(request,request_values)
            conn.commit()

    def update(self,table,where,kwargs):
        with sql.connect(self.filepath) as conn:
            request = f'''UPDATE {table} SET {','.join([f'{key}=?' for key in kwargs])} WHERE {where}'''
            request_values = tuple(kwargs.values())
            conn.cursor().execute(request,request_values)
            conn.commit()
        
    def get_row(self, table, key_column, key) -> dict:
        with sql.connect(self.filepath) as conn:
            cursor = conn.cursor()
            request = f'''SELECT * FROM {table} WHERE {key_column}=?'''
            cursor.execute(request,[key])
            conn.commit()
            row = cursor.fetchone()
            if row:
                return dict(zip([description[0] for description in cursor.description], row))
            else:
                return None
            
    def auth_user(self, session_id:str) -> dict:
        result:dict = self.get_row("users", "sessionID", session_id)
        if not result or result["sessionExpires"] < int(time()):
            return None
        else:
            return result
        
            
class Config():
    def __init__(self):
        config_file_path = os.environ.get('SERVER_CONFIG')
        with open(config_file_path) as f:
            self.config = json.load(f)
            
    def get(self, key: str) -> str:
        return self.config[key]
    
class Interface:
    def parse_incoming_request() -> dict:
        try: requests = json.loads(sys.argv[1])
        except: return None
        else:  return requests
    
    def export_to_http(status_code: int, message: str, headers:dict, body:bytes) -> bytes:
        output = ""
        for k,v in headers.items(): output += f"{k}:{v}\r\n"
        return bytes(f"{status_code} {message}", "utf-8") + b"\r\n" + bytes(output, "utf-8") + b"\r\n" + body

    def send_to_http(code:int, message:str, headers:dict, body):
        if type(body) != bytes: body = bytes(body, "utf-8")
        data = Interface.export_to_http(code, message, headers, body)
        sys.stdout.buffer.write(data)
        sys.stdout.flush()
        sys.exit()
        
    def send_file(headers:dict, filename:str):
        with open(filename, "rb") as f: data=f.read()
        Interface.send_to_http(headers, data)
        
    def read_file(filename: str) -> str:
        with open(filename, "r") as f: data=f.read()
        return data
    
    def parse_body_query() -> dict:
        result = {}
        body:str = Interface.parse_incoming_request()["body"]
        for s in body.split("&"):
            parts = s.split("=")
            result[parts[0]] = parts[1]
        return result
    
    def sha256(message:str) -> str:
        return hashlib.sha256(message.encode()).hexdigest()
    