// Simple test to see if mouse control is working
const TcpSystemControllerClient = require('./lib/TcpSystemControllerClient');

async function testMouseControl() {
    console.log('Testing mouse control...');
    
    const client = new TcpSystemControllerClient('localhost', 8080);
    
    try {
        await client.connect();
        console.log('Connected to server');
        
        console.log('Moving mouse to center of screen (500, 500)...');
        await client.mouseMove(500, 500);
        
        console.log('Waiting 2 seconds...');
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        console.log('Moving mouse to corner (100, 100)...');
        await client.mouseMove(100, 100);
        
        console.log('Waiting 2 seconds...');
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        console.log('Clicking at current position...');
        await client.mouseClick('Left', 100, 100);
        
        console.log('Test complete! Did you see the mouse move and click?');
        
    } catch (error) {
        console.error('Test failed:', error.message);
    } finally {
        client.destroy();
    }
}

testMouseControl();