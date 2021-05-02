
from pythonosc.udp_client import SimpleUDPClient
import time

ip = "192.168.0.36"
port = 9997

client = SimpleUDPClient(ip, port)  # Create client

while True:
    client.send_message("/track/2/volume", 123)   # Send float message
    client.send_message("/track/2/volume", 321)   # Send float message
    time.sleep(0.5)
# client.send_message("/some/address", [1, 2., "hello"])  # Send message with int, float and string

