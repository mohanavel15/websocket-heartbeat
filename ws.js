const address = "ws://localhost:8080";

const ws_onopen = () => console.log('WebSocket connection opened');
const ws_onclose = () => console.log('WebSocket connection closed');
const ws_onmessage = (event) => {
    let data = JSON.parse(event.data);
    console.log("Message:", data);

    switch (data.op) {
        case 10:
            heartBeat(data.d.heartbeat_interval);
            break;
        case 11:
            console.log('Received heart beat ack');
            lastHeartBeatAck = new Date().getTime();
            break;
    }
}

const NewGateway = () => {
    let ws = new WebSocket(address);
    ws.onopen = ws_onopen;
    ws.onclose = ws_onclose;
    ws.onmessage = ws_onmessage;
    return ws;
}

let ws = NewGateway();

let lastHeartBeat = new Date().getTime();
let lastHeartBeatAck = new Date().getTime();

function heartBeat(interval) {
    setTimeout(() => {
        if (lastHeartBeat > lastHeartBeatAck) {
            console.log('Heart beat ack not received, reconnecting...');
            ws.close();
            setTimeout(() => {
                lastHeartBeat = new Date().getTime();
                lastHeartBeatAck = new Date().getTime();
                ws = NewGateway();
            }, 5000)
        } else {
            const message = JSON.stringify({ op: 1 });
            ws.send(message);
            lastHeartBeat = new Date().getTime();
            heartBeat(interval);
        }
    }, interval);
}