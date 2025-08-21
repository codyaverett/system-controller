#!/usr/bin/env node
/**
 * System Controller WebSocket Client
 * 
 * Advanced WebSocket client for remote system control with:
 * - Multiple operation modes (demo, interactive, monitor, benchmark)
 * - Beautiful CLI interface with colors and progress indicators
 * - Real-time performance monitoring and metrics
 * - Automatic reconnection and error recovery
 * - Configuration management
 * 
 * Usage:
 *   npm start                    # Default demo mode
 *   npm run demo                 # Full feature demonstration
 *   npm run interactive          # Interactive command mode
 *   npm run monitor              # Real-time system monitoring
 *   npm run benchmark            # Performance benchmarking
 */

const { Command } = require('commander');
const chalk = require('chalk');
const ora = require('ora');
const Table = require('cli-table3');
const inquirer = require('inquirer');
const TcpSystemControllerClient = require('./lib/TcpSystemControllerClient');
const config = require('./config/default.json');

const program = new Command();

// CLI setup
program
  .name('system-controller-client')
  .description('System Controller WebSocket Client')
  .version('1.0.0')
  .option('-h, --host <host>', 'Server hostname', config.server.host)
  .option('-p, --port <port>', 'Server port', config.server.port)
  .option('--no-tls', 'Disable TLS encryption')
  .option('--verify-ssl', 'Verify SSL certificates')
  .option('-u, --username <username>', 'Username for authentication', config.auth.username)
  .option('-P, --password <password>', 'Password for authentication')
  .option('-v, --verbose', 'Enable verbose logging')
  .option('-q, --quiet', 'Suppress output');

program
  .command('demo')
  .description('Run comprehensive feature demonstration')
  .action(async (options) => {
    await runDemo(getClientOptions(options));
  });

program
  .command('interactive')
  .description('Start interactive command mode')
  .action(async (options) => {
    await runInteractive(getClientOptions(options));
  });

program
  .command('monitor')
  .description('Start real-time system monitoring')
  .option('-i, --interval <seconds>', 'Monitoring interval in seconds', '5')
  .action(async (options) => {
    await runMonitor(getClientOptions(options), parseInt(options.interval));
  });

program
  .command('benchmark')
  .description('Run performance benchmarks')
  .option('-n, --iterations <count>', 'Number of iterations per test', '100')
  .action(async (options) => {
    await runBenchmark(getClientOptions(options), parseInt(options.iterations));
  });

// Default action (demo mode)
program.action(async (options) => {
  await runDemo(getClientOptions(options));
});

/**
 * Extract client options from CLI arguments
 */
function getClientOptions(options) {
  const parent = options.parent || program;
  
  return {
    host: parent.opts().host || config.server.host,
    port: parent.opts().port || config.server.port,
    useTls: parent.opts().tls !== false,
    verifySSL: parent.opts().verifySsl || config.server.verify_ssl,
    username: parent.opts().username || config.auth.username,
    password: parent.opts().password || config.auth.password,
    verbose: parent.opts().verbose || false,
    quiet: parent.opts().quiet || false
  };
}

/**
 * Create and setup client
 */
async function createClient(options) {
  const client = new TcpSystemControllerClient(options.host, options.port, {
    reconnect: true,
    reconnectInterval: 1000,
    maxReconnectAttempts: 5,
    commandTimeout: 30000
  });
  
  if (options.verbose) {
    client.on('connected', () => console.log(chalk.green('📡 Connected to server')));
    client.on('disconnected', () => console.log(chalk.yellow('📡 Disconnected from server')));
    client.on('error', (error) => console.log(chalk.red('❌ Client error:'), error.message));
    client.on('reconnecting', (attempt) => console.log(chalk.yellow(`🔄 Reconnecting (${attempt})...`)));
  }
  
  return client;
}

/**
 * Demo Mode - Comprehensive feature demonstration
 */
async function runDemo(options) {
  console.log(chalk.cyan.bold('🚀 System Controller WebSocket Client Demo'));
  console.log(chalk.gray('=' .repeat(60)));
  
  const client = await createClient(options);
  let spinner = ora('Connecting to server...').start();
  
  try {
    // Connect
    await client.connect();
    spinner.succeed('Connected to System Controller');
    
    // Authenticate
    spinner = ora('Authenticating...').start();
    const authenticated = await client.authenticate(options.username, options.password);
    
    if (!authenticated) {
      spinner.fail('Authentication failed');
      return;
    }
    
    spinner.succeed(`Authenticated as ${options.username}`);
    
    // Test heartbeat
    console.log(chalk.blue('\n💓 Testing Connection Health'));
    spinner = ora('Sending heartbeat...').start();
    const pingResponse = await client.ping();
    spinner.succeed(`Heartbeat: ${pingResponse.data.server_time}`);
    
    // Get system information
    console.log(chalk.blue('\n📊 System Information'));
    spinner = ora('Testing display commands...').start();
    try {
      const displaysResponse = await client.getDisplays();
      if (displaysResponse.data && displaysResponse.data.displays) {
        const displays = displaysResponse.data.displays;
        spinner.succeed(`Found ${displays.length} display(s)`);
        
        const displayTable = new Table({
          head: ['ID', 'Name', 'Resolution', 'Position', 'Primary'],
          colWidths: [5, 20, 15, 15, 10]
        });
        
        displays.forEach(display => {
          displayTable.push([
            display.id,
            display.name,
            `${display.width}×${display.height}`,
            `(${display.x}, ${display.y})`,
            display.is_primary ? '✓' : ''
          ]);
        });
        
        console.log(displayTable.toString());
      } else {
        spinner.warn('Display enumeration not yet implemented on server');
        console.log('   Server acknowledged command but returned no display data');
      }
    } catch (error) {
      spinner.warn('Display enumeration not available on server');
    }
    
    // Test mouse operations
    console.log(chalk.blue('\n🖱️  Mouse Operations Demo'));
    const mouseOps = [
      { desc: 'Moving to (150, 150)', op: () => client.mouseMove(150, 150) },
      { desc: 'Left click at (150, 150)', op: () => client.mouseClick('Left', 150, 150) },
      { desc: 'Scrolling down', op: () => client.mouseScroll(0, -3) }
    ];
    
    for (const { desc, op } of mouseOps) {
      spinner = ora(desc).start();
      await op();
      spinner.succeed(desc);
      await sleep(200);
    }
    
    // Test keyboard operations
    console.log(chalk.blue('\n⌨️  Keyboard Operations Demo'));
    const keyboardOps = [
      { desc: 'Pressing key "a"', op: () => client.keyPress('a') },
      { desc: 'Typing text', op: () => client.typeText('Hello from System Controller!') },
      { desc: 'Pressing Enter', op: () => client.keyPress('Enter') }
    ];
    
    for (const { desc, op } of keyboardOps) {
      spinner = ora(desc).start();
      await op();
      spinner.succeed(desc);
      await sleep(100);
    }
    
    // Test screen capture
    console.log(chalk.blue('\n📸 Screen Capture Demo'));
    spinner = ora('Testing screen capture...').start();
    try {
      const captureResponse = await client.captureScreen(0);
      if (captureResponse.data && captureResponse.data.size) {
        const captureSize = (captureResponse.data.size / 1024).toFixed(1);
        spinner.succeed(`Screenshot captured: ${captureSize} KB`);
      } else {
        spinner.warn('Screen capture acknowledged but no data returned');
      }
    } catch (error) {
      spinner.warn('Screen capture not fully implemented on server');
    }
    
    // Test window operations
    console.log(chalk.blue('\n🪟 Window Operations Demo'));
    spinner = ora('Testing window enumeration...').start();
    try {
      const windowsResponse = await client.listWindows();
      if (windowsResponse.data && windowsResponse.data.windows) {
        const windows = windowsResponse.data.windows;
        spinner.succeed(`Found ${windows.length} window(s)`);
        
        if (windows.length > 0) {
          const windowTable = new Table({
            head: ['ID', 'Title', 'Size', 'Process'],
            colWidths: [10, 40, 15, 20]
          });
          
          windows.slice(0, 5).forEach(window => {
            windowTable.push([
              window.id.toString(),
              window.title.substring(0, 35) + (window.title.length > 35 ? '...' : ''),
              `${window.width}×${window.height}`,
              window.process_name
            ]);
          });
          
          console.log(windowTable.toString());
        }
      } else {
        spinner.warn('Window enumeration not yet implemented on server');
      }
    } catch (error) {
      spinner.warn('Window operations not available on server');
    }
    
    // Performance metrics
    console.log(chalk.blue('\n📈 Client Performance Metrics'));
    const metrics = client.getMetrics();
    const metricsTable = new Table();
    
    metricsTable.push(
      ['Commands Sent', metrics.commandsSent],
      ['Success Rate', `${metrics.successRate}%`],
      ['Avg Response Time', `${metrics.averageResponseTime.toFixed(0)}ms`],
      ['Connection Uptime', `${metrics.uptime}s`]
    );
    
    console.log(metricsTable.toString());
    
    console.log(chalk.green.bold('\n✅ Demo completed successfully!'));
    console.log(chalk.gray('Try running with --interactive flag for manual control'));
    
  } catch (error) {
    spinner.fail(`Demo failed: ${error.message}`);
    if (options.verbose) {
      console.error(chalk.red(error.stack));
    }
  } finally {
    await sleep(1000);
    client.destroy();
  }
}

/**
 * Interactive Mode - Manual command input
 */
async function runInteractive(options) {
  console.log(chalk.cyan.bold('🎮 Interactive System Controller'));
  console.log(chalk.gray('Type "help" for available commands, "quit" to exit'));
  console.log(chalk.gray('=' .repeat(50)));
  
  const client = await createClient(options);
  
  try {
    await client.connect();
    console.log(chalk.green('✅ Connected to server'));
    
    if (!await client.authenticate(options.username, options.password)) {
      console.log(chalk.red('❌ Authentication failed'));
      return;
    }
    
    console.log(chalk.green('✅ Authenticated successfully\n'));
    
    while (true) {
      const { command } = await inquirer.prompt([{
        type: 'input',
        name: 'command',
        message: chalk.blue('Command:'),
        prefix: '📝'
      }]);
      
      const parts = command.trim().split(' ');
      const cmd = parts[0].toLowerCase();
      
      try {
        if (cmd === 'quit' || cmd === 'exit') {
          break;
        } else if (cmd === 'help') {
          showHelp();
        } else if (cmd === 'move' && parts.length >= 3) {
          await client.mouseMove(parseInt(parts[1]), parseInt(parts[2]));
          console.log(chalk.green('✅ Mouse moved'));
        } else if (cmd === 'click' && parts.length >= 2) {
          const button = parts[1].charAt(0).toUpperCase() + parts[1].slice(1);
          await client.mouseClick(button, 100, 100);
          console.log(chalk.green('✅ Mouse clicked'));
        } else if (cmd === 'type' && parts.length >= 2) {
          await client.typeText(parts.slice(1).join(' '));
          console.log(chalk.green('✅ Text typed'));
        } else if (cmd === 'key' && parts.length >= 2) {
          await client.keyPress(parts[1]);
          console.log(chalk.green('✅ Key pressed'));
        } else if (cmd === 'capture') {
          const response = await client.captureScreen();
          const size = (response.data.data_size / 1024).toFixed(1);
          console.log(chalk.green(`✅ Screenshot captured: ${size} KB`));
        } else if (cmd === 'displays') {
          const response = await client.getDisplays();
          response.data.displays.forEach(d => 
            console.log(`   ${chalk.cyan(d.id)}: ${d.name} (${d.width}×${d.height})`));
        } else if (cmd === 'windows') {
          const response = await client.listWindows();
          response.data.windows.slice(0, 10).forEach(w => 
            console.log(`   ${chalk.cyan(w.id)}: ${w.title.substring(0, 60)}`));
        } else if (cmd === 'ping') {
          const response = await client.ping();
          console.log(chalk.green(`✅ Pong: ${response.data.server_time}`));
        } else if (cmd === 'metrics') {
          const metrics = client.getMetrics();
          console.log(chalk.green(`✅ ${metrics.commandsSent} sent, ${metrics.successRate}% success, ${metrics.averageResponseTime.toFixed(0)}ms avg`));
        } else if (cmd === 'clear') {
          console.clear();
        } else {
          console.log(chalk.yellow('❓ Unknown command. Type "help" for available commands.'));
        }
      } catch (error) {
        console.log(chalk.red('❌ Command failed:'), error.message);
      }
    }
    
  } catch (error) {
    console.log(chalk.red('❌ Interactive mode failed:'), error.message);
  } finally {
    client.destroy();
  }
}

/**
 * Monitor Mode - Real-time system monitoring
 */
async function runMonitor(options, interval = 5) {
  console.log(chalk.cyan.bold('📊 System Controller Real-time Monitor'));
  console.log(chalk.gray(`Monitoring interval: ${interval} seconds`));
  console.log(chalk.gray('Press Ctrl+C to stop'));
  console.log(chalk.gray('=' .repeat(50)));
  
  const client = await createClient(options);
  let monitoringActive = true;
  
  // Handle Ctrl+C gracefully
  process.on('SIGINT', () => {
    console.log(chalk.yellow('\n⚠️  Stopping monitor...'));
    monitoringActive = false;
  });
  
  try {
    await client.connect();
    console.log(chalk.green('✅ Connected and starting monitor...\n'));
    
    if (!await client.authenticate(options.username, options.password)) {
      console.log(chalk.red('❌ Authentication failed'));
      return;
    }
    
    let iterationCount = 0;
    
    while (monitoringActive) {
      try {
        iterationCount++;
        const timestamp = new Date().toLocaleTimeString();
        
        console.log(chalk.blue(`\n📊 Monitor Update #${iterationCount} - ${timestamp}`));
        console.log(chalk.gray('-'.repeat(50)));
        
        // System status check
        const startTime = Date.now();
        const displaysResponse = await client.getDisplays();
        const responseTime = Date.now() - startTime;
        
        if (displaysResponse.status === 'success') {
          const displays = displaysResponse.data.displays;
          console.log(chalk.green(`✅ System responsive (${responseTime}ms)`));
          console.log(`   Displays: ${displays.length}`);
          
          // Show primary display info
          const primary = displays.find(d => d.is_primary);
          if (primary) {
            console.log(`   Primary: ${primary.name} (${primary.width}×${primary.height})`);
          }
        } else {
          console.log(chalk.red(`❌ System error: ${displaysResponse.error}`));
        }
        
        // Client metrics
        const metrics = client.getMetrics();
        console.log(`   Commands: ${metrics.commandsSent} sent, ${metrics.successRate}% success`);
        console.log(`   Performance: ${metrics.averageResponseTime.toFixed(0)}ms avg response`);
        console.log(`   Uptime: ${metrics.uptime}s`);
        
        // Health indicators
        if (responseTime < 100) {
          console.log(chalk.green('   Health: Excellent'));
        } else if (responseTime < 500) {
          console.log(chalk.yellow('   Health: Good'));
        } else {
          console.log(chalk.red('   Health: Poor'));
        }
        
        if (monitoringActive) {
          await sleep(interval * 1000);
        }
        
      } catch (error) {
        console.log(chalk.red(`❌ Monitor error: ${error.message}`));
        await sleep(interval * 1000);
      }
    }
    
  } catch (error) {
    console.log(chalk.red('❌ Monitor failed:'), error.message);
  } finally {
    client.destroy();
    console.log(chalk.gray('\n📊 Monitor stopped'));
  }
}

/**
 * Benchmark Mode - Performance testing
 */
async function runBenchmark(options, iterations = 100) {
  console.log(chalk.cyan.bold('🏃 System Controller Performance Benchmark'));
  console.log(chalk.gray(`Running ${iterations} iterations per test`));
  console.log(chalk.gray('=' .repeat(60)));
  
  const client = await createClient(options);
  
  try {
    await client.connect();
    console.log(chalk.green('✅ Connected to server'));
    
    if (!await client.authenticate(options.username, options.password)) {
      console.log(chalk.red('❌ Authentication failed'));
      return;
    }
    
    const benchmarks = {
      'Mouse Move': {
        iterations: iterations,
        operation: (i) => client.mouseMove(100 + i, 100 + i),
        times: []
      },
      'Key Press': {
        iterations: iterations,
        operation: () => client.keyPress('a'),
        times: []
      },
      'Get Displays': {
        iterations: Math.min(50, iterations), // Fewer iterations for heavier ops
        operation: () => client.getDisplays(),
        times: []
      },
      'Type Text': {
        iterations: Math.min(25, iterations),
        operation: () => client.typeText('benchmark'),
        times: []
      },
      'Ping': {
        iterations: iterations,
        operation: () => client.ping(),
        times: []
      }
    };
    
    for (const [testName, benchmark] of Object.entries(benchmarks)) {
      console.log(chalk.blue(`\n🧪 Testing ${testName}...`));
      const spinner = ora(`Running ${benchmark.iterations} iterations...`).start();
      
      for (let i = 0; i < benchmark.iterations; i++) {
        try {
          const start = Date.now();
          await benchmark.operation(i);
          const time = Date.now() - start;
          benchmark.times.push(time);
          
          if (i % 10 === 0 && i > 0) {
            spinner.text = `Running ${testName}: ${i}/${benchmark.iterations}`;
          }
        } catch (error) {
          benchmark.times.push(-1); // Mark failed operations
        }
      }
      
      spinner.succeed(`${testName} completed`);
    }
    
    // Calculate and display results
    console.log(chalk.blue('\n📊 Benchmark Results'));
    console.log(chalk.gray('=' .repeat(80)));
    
    const resultsTable = new Table({
      head: ['Test', 'Iterations', 'Avg (ms)', 'Min (ms)', 'Max (ms)', 'P95 (ms)', 'Success %'],
      colWidths: [15, 12, 10, 10, 10, 10, 12]
    });
    
    for (const [testName, benchmark] of Object.entries(benchmarks)) {
      const validTimes = benchmark.times.filter(t => t >= 0);
      const failedCount = benchmark.times.filter(t => t < 0).length;
      
      if (validTimes.length > 0) {
        const avg = validTimes.reduce((a, b) => a + b, 0) / validTimes.length;
        const min = Math.min(...validTimes);
        const max = Math.max(...validTimes);
        const p95 = validTimes.sort((a, b) => a - b)[Math.floor(validTimes.length * 0.95)];
        const successRate = ((validTimes.length / benchmark.times.length) * 100).toFixed(1);
        
        resultsTable.push([
          testName,
          benchmark.times.length.toString(),
          avg.toFixed(1),
          min.toString(),
          max.toString(),
          (p95 || 0).toString(),
          `${successRate}%`
        ]);
      } else {
        resultsTable.push([
          testName,
          benchmark.times.length.toString(),
          'FAILED',
          'FAILED',
          'FAILED',
          'FAILED',
          '0%'
        ]);
      }
    }
    
    console.log(resultsTable.toString());
    
    // Overall client metrics
    const metrics = client.getMetrics();
    console.log(chalk.blue('\n📈 Overall Client Performance'));
    const overallTable = new Table();
    overallTable.push(
      ['Total Commands', metrics.commandsSent],
      ['Overall Success Rate', `${metrics.successRate}%`],
      ['Client Avg Response', `${metrics.averageResponseTime.toFixed(0)}ms`],
      ['Connection Uptime', `${metrics.uptime}s`]
    );
    
    console.log(overallTable.toString());
    console.log(chalk.green.bold('\n✅ Benchmark completed!'));
    
  } catch (error) {
    console.log(chalk.red('❌ Benchmark failed:'), error.message);
  } finally {
    client.destroy();
  }
}

/**
 * Show help information
 */
function showHelp() {
  console.log(chalk.cyan('\n📚 Available Commands:'));
  console.log(chalk.gray('─────────────────────'));
  
  const commands = [
    ['move X Y', 'Move mouse to coordinates'],
    ['click BUTTON', 'Click mouse button (Left/Right/Middle)'],
    ['type TEXT', 'Type text'],
    ['key KEY', 'Press key (Enter, Space, Tab, etc.)'],
    ['capture', 'Capture screenshot'],
    ['displays', 'List available displays'],
    ['windows', 'List open windows'],
    ['ping', 'Test server connection'],
    ['metrics', 'Show client performance metrics'],
    ['clear', 'Clear screen'],
    ['help', 'Show this help'],
    ['quit/exit', 'Exit interactive mode']
  ];
  
  const helpTable = new Table({
    head: ['Command', 'Description'],
    colWidths: [15, 40]
  });
  
  commands.forEach(([cmd, desc]) => {
    helpTable.push([chalk.cyan(cmd), desc]);
  });
  
  console.log(helpTable.toString());
}

/**
 * Sleep utility
 */
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Error handling
process.on('unhandledRejection', (error) => {
  console.error(chalk.red('💥 Unhandled rejection:'), error.message);
  process.exit(1);
});

process.on('uncaughtException', (error) => {
  console.error(chalk.red('💥 Uncaught exception:'), error.message);
  process.exit(1);
});

// Parse command line arguments
if (require.main === module) {
  program.parse();
}