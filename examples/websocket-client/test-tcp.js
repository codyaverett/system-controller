const net = require('net');

// Test TCP connection to server
const client = new net.Socket();

client.connect(8080, 'localhost', () => {
    console.log('Connected to server');
    
    // Send a valid command
    const command = {
        id: 'test-1',
        type: 'mouse_move',
        payload: {
            type: 'mouse_move',
            x: 100,
            y: 100
        },
        timestamp: new Date().toISOString()
    };
    
    client.write(JSON.stringify(command) + '\n');
});

client.on('data', (data) => {
    console.log('Received:', data.toString());
    client.destroy();
});

client.on('close', () => {
    console.log('Connection closed');
});

client.on('error', (err) => {
    console.log('Error:', err);
});