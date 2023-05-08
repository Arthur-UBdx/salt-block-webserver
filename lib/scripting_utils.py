import sqlite3 as sql
import json
import os
import sys

DIR = os.path.abspath(os.curdir)

class Database:
    def __init__(self,filename):
        self.filepath = os.path.join(DIR,filename)
        
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
            
class Config():
    def __init__(self):
        config_file_path = os.environ.get('SERVER_CONFIG')
        with open(config_file_path) as f:
            self.config = json.load(f)
            
    def get_value(self, value: str) -> str:
        return self.config[value]
    
class Interface():
    def parse_input() -> dict:
        return Interface.parse_json(sys.argv[1])
        
    def parse_json(target:str) -> dict:
        return json.loads(target)
    
    def export_to_http(headers:dict, body:str) -> str:
        output = ""
        for k,v in headers.items():
            output += f"{k}:{v}\r\n"
        output += f"\r\n{body}"
        return output