const dgram = require('dgram');

function createStunRequest() {
    // 创建一个简单的STUN请求
    // STUN消息头部：类型（2字节），长度（2字节），魔法Cookie（4字节），事务ID（12字节）
    const msgType = 0x0001;  // Binding Request
    const msgLength = 0x0000;  // No attributes
    const magicCookie = 0x2112A442;
    const transactionId = Buffer.from([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B]);
    const stunRequest = Buffer.alloc(20);
    stunRequest.writeUInt16BE(msgType, 0);
    stunRequest.writeUInt16BE(msgLength, 2);
    stunRequest.writeUInt32BE(magicCookie, 4);
    transactionId.copy(stunRequest, 8);
    return stunRequest;
}

function parseStunResponse(response) {
    // 解析STUN响应
    const msgType = response.readUInt16BE(0);
    const msgLength = response.readUInt16BE(2);
    const magicCookie = response.readUInt32BE(4);
    const transactionId = response.slice(8, 20).toString('hex');

    console.log(`Message Type: ${msgType}`);
    console.log(`Message Length: ${msgLength}`);
    console.log(`Magic Cookie: ${magicCookie}`);
    console.log(`Transaction ID: ${transactionId}`);

    // 解析XorMappedAddress属性
    const attributeType = response.readUInt16BE(20);
    const attributeLength = response.readUInt16BE(22);
    if (attributeType === 0x0020) {  // XorMappedAddress
        const family = response.readUInt8(25);
        if (family === 0x01) {  // IPv4
            let port = response.readUInt16BE(26);
            port ^= magicCookie >>> 16;
            let ip = response.readUInt32BE(28);
            ip ^= magicCookie;
            ip = Buffer.from([
                (ip >> 24) & 0xFF,
                (ip >> 16) & 0xFF,
                (ip >> 8) & 0xFF,
                ip & 0xFF
            ]).join('.');
            console.log(`XorMappedAddress: ${ip}:${port}`);
        }
    }
}

function testStunServer(serverAddress) {
    const stunRequest = createStunRequest();
    const client = dgram.createSocket('udp4');

    client.send(stunRequest, serverAddress.port, serverAddress.address, (err) => {
        if (err) {
            console.error(`Error sending STUN request: ${err.message}`);
            client.close();
            return;
        }
        console.log(`Sent STUN request to ${serverAddress.address}:${serverAddress.port}`);
    });

    client.on('message', (response, rinfo) => {
        console.log(`Received response from ${rinfo.address}:${rinfo.port}`);
        parseStunResponse(response);
        client.close();
    });

    client.on('error', (err) => {
        console.error(`Socket error: ${err.message}`);
        client.close();
    });

    client.setTimeout(5000, () => {
        console.log('Request timed out');
        client.close();
    });
}

const serverAddress = { address: '127.0.0.1', port: 3478 };  // 替换为你的STUN服务器地址和端口
testStunServer(serverAddress);
