import requests

response = requests.get("http://google.com")
encoding = response.encoding or 'utf-8'
print(response.content.decode(encoding))