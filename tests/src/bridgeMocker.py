import socket
import json

server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server.bind(('127.0.0.1', 8080))
server.listen(1)

conn, addr = server.accept()

manifest = """"
"name": "Test"
"description": "Testing",
"version": "1.0.0",
"author": "Tester",
"repository": "",
download_url: "",
id: "com.tester.test",
"category": "exporter",
"exports": [
    {
        "id": "main",
        "label": "Exporter",
        "description": "Accepts something",
        "entrypoint": "main.py:main",
    }
]
"""

# Send Manifest for Handhskae, wait for data.

def main():
    # Get the Network data, export to an XLSV, send a message back to server if succeed
    print("Testing")
