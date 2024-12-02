import socket
import struct

def create_stun_request():
    # 创建一个简单的STUN请求
    # STUN消息头部：类型（2字节），长度（2字节），魔法Cookie（4字节），事务ID（12字节）
    msg_type = 0x0001  # Binding Request
    msg_length = 0x0000  # No attributes
    magic_cookie = 0x2112A442
    transaction_id = b'\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B'
    stun_request = struct.pack('!HHI12s', msg_type, msg_length, magic_cookie, transaction_id)
    return stun_request

def parse_stun_response(response):
    # 解析STUN响应
    msg_type, msg_length, magic_cookie = struct.unpack('!HHI', response[:8])
    transaction_id = response[8:20]
    print(f"Message Type: {msg_type}")
    print(f"Message Length: {msg_length}")
    print(f"Magic Cookie: {magic_cookie}")
    print(f"Transaction ID: {transaction_id.hex()}")

    # 解析XorMappedAddress属性
    attribute_type, attribute_length = struct.unpack('!HH', response[20:24])
    if attribute_type == 0x0020:  # XorMappedAddress
        family = struct.unpack('!B', response[25:26])[0]
        if family == 0x01:  # IPv4
            port, = struct.unpack('!H', response[26:28])
            port ^= magic_cookie >> 16
            ip = struct.unpack('!I', response[28:32])[0]
            ip ^= magic_cookie
            ip = socket.inet_ntoa(struct.pack('!I', ip))
            print(f"XorMappedAddress: {ip}:{port}")

def test_stun_server(server_address):
    stun_request = create_stun_request()
    
    # 创建UDP套接字
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(5)  # 设置超时时间为5秒
    
    try:
        # 发送STUN请求
        sock.sendto(stun_request, server_address)
        print(f"Sent STUN request to {server_address}")
        
        # 接收STUN响应
        response, addr = sock.recvfrom(1024)
        print(f"Received response from {addr}")
        parse_stun_response(response)
    except socket.timeout:
        print("Request timed out")
    finally:
        sock.close()

if __name__ == "__main__":
    server_address = ("127.0.0.1", 3478)  # 替换为你的STUN服务器地址和端口
    test_stun_server(server_address)