/**
 * Five CLI ASCII Art and Visual Effects
 * 
 * Cool 2000s-style terminal aesthetics with colorful ASCII art,
 * gradients, and retro visual effects.
 */

import chalk from 'chalk';

/**
 * Main 5IVE ASCII art banner with demoscene styling
 * Enhanced step by step following user's iterative approach
 */
export function getFiveASCIIBanner(): string {
  // Step 2.5: Responsive demoscene-style banner (narrower and reactive)
  const terminalWidth = process.stdout.columns || 80;
  
  // ACID-MELTED LIQUID WASTE EFFECT - Compact banner for narrow terminals (< 70 columns)
  const bannerCompact = `
╔═══════════════════════════════════════════════════╗
║░▒▓██▓▒░░▒▓█████▓▒░░▒▓██▓▒░░▒▓██▓▒░░▒▓█████▓▒░░▒║
║  ███████╗██╗██╗   ██╗███████╗  ░▒▓█▓▒░       ║
║  ██╔═▓▒░▒██║██║▒▓░██║██╔══▒▓▒░              ░║
║  █████▓▒ ██║██║ ▒▓██║█████▓▒░               ░║
║  ██╔══░  ██║██║ ░▒██║██╔══░                 ░║
║  ██║ ░▒▓▒██║╚████▓▒▓███████╗                ░║
║  ╚═╝  ░▒▓╚═╝ ╚═▓▒░▒ ╚══════╝░▒▓█           ░║
║░ ░  ░▒▓░  ░░▒▓██▓▒░  ░ ░▒▓░ ░░▒▓█▓▒░        ░║
║░░▒▓░  ░ ░▒▓██▓▒░ ░  ░▒▓█▓▒░   ░▒▓█         ║
║░▒▓█▓▒░░▒▓██▓▒░░▒▓█▓▒░░▒▓██▓▒░░▒▓█████▓▒░░▒▓║
╚═══════════════════════════════════════════════════╝
 ░▒▓█▓▒░ ░▒▓█▓▒░ ░▒▓█▓▒░ ░▒▓█▓▒░ ░▒▓█▓▒░ 
   ░▒▓█▓▒░   ░▒▓█▓▒░   ░▒▓█▓▒░   ░▒▓█
     ░▒▓█       ░▒▓█       ░▒▓
       ░           ░         ░`;

  // ACID-MELTED LIQUID WASTE EFFECT - Medium banner for standard terminals (70-100 columns)
  const bannerMedium = `
╔══════════════════════════════════════════════════════════╗
║░▒▓███▓▒░░▒▓████▓▒░░▒▓███▓▒░░▒▓███▓▒░░▒▓████▓▒░░▒▓███▓▒║
║░▒▓█    ░  ░▒▓█  ░ ░▒▓█ ░   ░▒▓█  ░ ░▒▓█  ░  ░▒▓█   ░║
║    ███████╗██╗██╗   ██╗███████╗      ░▒▓█████▓▒░     ║
║    ██╔═▓▒░▒██║██║▒▓░██║██╔══▒▓▒░                     ║
║    █████▓▒ ██║██║ ▒▓██║█████▓▒░                      ║
║    ██╔══░  ██║██║ ░▒██║██╔══░                        ║
║    ██║ ░▒▓▒██║╚████▓▒▓███████╗                       ║
║    ╚═╝  ░▒▓╚═╝ ╚═▓▒░▒ ╚══════╝░▒▓█                  ║
║░ ░  ░▒▓░  ░░▒▓██▓▒░  ░ ░▒▓░ ░░▒▓█▓▒░                 ║
║░░▒▓░  ░ ░▒▓██▓▒░ ░  ░▒▓█▓▒░   ░▒▓█                  ║
║░▒▓█▓▒░░▒▓██▓▒░░▒▓█▓▒░░▒▓██▓▒░░▒▓█████▓▒░░▒▓█▓▒░░▒▓ ║
║░▒▓█  ░ ░▒▓█ ░  ░▒▓█  ░ ░▒▓█  ░ ░▒▓█  ░ ░▒▓█ ░  ░▒▓║
╚══════════════════════════════════════════════════════════╝
 ░▒▓███▓▒░ ░▒▓███▓▒░ ░▒▓███▓▒░ ░▒▓███▓▒░ ░▒▓███▓▒░ 
   ░▒▓███▓▒░   ░▒▓███▓▒░   ░▒▓███▓▒░   ░▒▓███▓▒░
     ░▒▓████       ░▒▓████       ░▒▓████     ░▒▓
       ░▒▓█▓         ░▒▓█▓         ░▒▓█▓       
         ░▒             ░▒           ░▒        
           ░               ░           ░`;

  // ACID-MELTED LIQUID WASTE EFFECT - Wide banner for large terminals (> 100 columns)
  const bannerWide = `
╔═══════════════════════════════════════════════════════════════════════╗
║░▒▓████▓▒░░▒▓█████▓▒░░▒▓████▓▒░░▒▓████▓▒░░▒▓█████▓▒░░▒▓████▓▒░░▒▓████║
║░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░▒║
║░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█    ░║
║      ███████╗██╗██╗   ██╗███████╗         ░▒▓██████▓▒░          ║
║      ██╔═▓▒░▒██║██║▒▓░██║██╔══▒▓▒░                             ║
║      █████▓▒ ██║██║ ▒▓██║█████▓▒░                              ║
║      ██╔══░  ██║██║ ░▒██║██╔══░                                ║
║      ██║ ░▒▓▒██║╚████▓▒▓███████╗                               ║
║      ╚═╝  ░▒▓╚═╝ ╚═▓▒░▒ ╚══════╝░▒▓█                          ║
║░ ░  ░▒▓░  ░░▒▓██▓▒░  ░ ░▒▓░ ░░▒▓█▓▒░                          ║
║░░▒▓░  ░ ░▒▓██▓▒░ ░  ░▒▓█▓▒░   ░▒▓█                           ║
║░▒▓█▓▒░░▒▓██▓▒░░▒▓█▓▒░░▒▓██▓▒░░▒▓█████▓▒░░▒▓█▓▒░░▒▓█▓▒░░▒▓█ ║
║░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██  ░ ░▒▓██║
║░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█     ░▒▓█  ║
╚═══════════════════════════════════════════════════════════════════════╝
 ░▒▓████▓▒░ ░▒▓████▓▒░ ░▒▓████▓▒░ ░▒▓████▓▒░ ░▒▓████▓▒░ ░▒▓████▓▒░
   ░▒▓████▓▒░   ░▒▓████▓▒░   ░▒▓████▓▒░   ░▒▓████▓▒░   ░▒▓████▓▒░
     ░▒▓█████       ░▒▓█████       ░▒▓█████       ░▒▓█████     ░▒▓
       ░▒▓███▓         ░▒▓███▓         ░▒▓███▓         ░▒▓███▓     
         ░▒▓█▓           ░▒▓█▓           ░▒▓█▓           ░▒▓█▓     
           ░▒▓             ░▒▓             ░▒▓             ░▒▓    
             ░               ░               ░               ░`;

  // Choose responsive banner based on terminal width
  const selectedBanner = terminalWidth < 70 ? bannerCompact :
                         terminalWidth < 100 ? bannerMedium : 
                         bannerWide;
  const lines = selectedBanner.split('\n');
  const coloredLines = lines.map((line, index) => {
    if (line.trim() === '') return line;
    
    // ACID-MELTED LIQUID WASTE COLOR SCHEME
    // Simulate toxic waste colors: acid greens, warning yellows, radioactive blues, dangerous reds
    
    if (line.includes('░▒▓') || line.includes('▓▒░')) {
      // Acid drip effects and waste patterns - radioactive green and toxic yellow
      if (line.includes('░▒▓█')) {
        return chalk.hex('#00FF41')(line); // Matrix green for heavy acid
      } else {
        return chalk.hex('#CCFF00')(line); // Radioactive yellow-green
      }
    } else if (line.includes('██') && line.includes('╗╝╚═')) {
      // Main FIVE text - corrupted but still readable - toxic blue-white
      return chalk.hex('#00FFFF')(line); // Bright cyan like acid
    } else if (line.includes('██')) {
      // FIVE letter forms - bright corrosive colors
      if (index % 3 === 0) {
        return chalk.hex('#FF0040')(line); // Danger red
      } else if (index % 3 === 1) {
        return chalk.hex('#40FF00')(line); // Toxic green  
      } else {
        return chalk.hex('#FFFF00')(line); // Warning yellow
      }
    } else if (line.includes('╔╗╚═')) {
      // Border frames - dark radioactive 
      return chalk.hex('#004000')(line); // Dark toxic green
    } else if (line.includes('░') || line.includes('▒') || line.includes('▓')) {
      // Dripping waste effects
      return chalk.hex('#80FF00')(line); // Bright lime acid
    } else {
      // Other elements - muted toxic glow
      return chalk.hex('#008040')(line); // Medium toxic green
    }
  });

  return coloredLines.join('\n');
}

/**
 * Alternative retro computer style banner
 */
export function getRetroComputerBanner(): string {
  const banner = `
┌─── ▄▄▄▄▄ ─── ▄ ─── ▄   ▄ ─── ▄▄▄▄▄ ────────────────────────────────────────────┐
│ ▐▌ █▀▀▀▀ ▐▌ █ ▐▌ █ ▐▌ █ ▐▌ █▀▀▀▀ ▐▌   F I V E   V I R T U A L   M A C H I N E │
│ ▐▌ █▄▄▄  ▐▌ █ ▐▌ █ ▐▌ █ ▐▌ █▄▄▄  ▐▌   ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ │
│ ▐▌     █ ▐▌ █ ▐▌ █ ▐▌ █ ▐▌     █ ▐▌   ░░ BLOCKCHAIN BYTECODE EXECUTION ░░   │
│ ▐▌ ▄▄▄▄█ ▐▌ █▄▄▄▄ ▐▌▄▄█ ▐▌ ▄▄▄▄█ ▐▌   ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ │
└─────────────────────────────────────────────────────────────────────────────┘`;

  return chalk.cyan(banner);
}

/**
 * Compact matrix/digital style banner
 */
export function getMatrixStyleBanner(): string {
  const banner = `
▀█▀▀█ █ █ █ █▀▀▀ ░▒▓█ FIVE VM █▓▒░ █▀▀▀ █ █ █ █▀▀█▀
▀▀▀▀█ █ █ █ █▄▄▄ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ █▄▄▄ █ █ █ █▀▀▀▀
    █ █ ▀▄█ █▄▄▄ ▒▒▒ SOL VM ▒▒▒ █▄▄▄ █▄▀ █ █    
▄▄▄▄█ █ ▄▄█ ▄▄▄▄ ░░░░░░░░░░░░░░░ ▄▄▄▄ █▄▄ █ █▄▄▄▄`;

  const lines = banner.split('\n');
  return lines.map((line, index) => {
    const colors = [chalk.green, chalk.cyan, chalk.magenta, chalk.yellow];
    return colors[index % colors.length](line);
  }).join('\n');
}

/**
 * Alternative stylized 5IVE logo for compact display
 */
export function getCompactFiveLogo(): string {
  return chalk.cyan('█') + chalk.magenta('5') + chalk.yellow('█') + 
         chalk.cyan('I') + chalk.magenta('V') + chalk.yellow('E') + chalk.cyan('█');
}

/**
 * 2000s-style welcome message with decorative elements
 */
export function getWelcomeMessage(): string {
  const decorativeBar = chalk.cyan('═'.repeat(60));
  const tagline = chalk.bold.magenta('Five VM - The Future of Blockchain Execution');
  const subtitle = chalk.gray('Ultra-fast bytecode VM for Solana smart contracts');
  
  return `
${decorativeBar}
${chalk.bold.cyan('                    Welcome to Five CLI')}
${tagline}
${subtitle}
${decorativeBar}
`;
}

/**
 * Stylized section header with 2000s aesthetics
 */
export function createSectionHeader(title: string, color: 'cyan' | 'magenta' | 'yellow' | 'green' = 'cyan'): string {
  const colorFn = chalk[color];
  const border = '▓'.repeat(title.length + 4);
  
  return `
${colorFn(border)}
${colorFn(`▓ ${title} ▓`)}
${colorFn(border)}`;
}

/**
 * Retro-style command example with syntax highlighting
 */
export function styleCommandExample(command: string, description: string): string {
  // Split command into parts for colorization
  const parts = command.split(' ');
  const baseCommand = chalk.bold.cyan(parts[0]);
  const subCommand = parts[1] ? chalk.bold.yellow(parts[1]) : '';
  const flags = parts.slice(2).map(part => 
    part.startsWith('-') ? chalk.green(part) : chalk.white(part)
  ).join(' ');
  
  const styledCommand = [baseCommand, subCommand, flags].filter(Boolean).join(' ');
  const arrow = chalk.magenta('▶');
  const desc = chalk.gray(description);
  
  return `  ${arrow} ${styledCommand.padEnd(35)} ${desc}`;
}

/**
 * 2000s-style loading animation frames
 */
export const loadingFrames = [
  chalk.cyan('⠋'),
  chalk.magenta('⠙'), 
  chalk.yellow('⠹'),
  chalk.green('⠸'),
  chalk.blue('⠼'),
  chalk.red('⠴'),
  chalk.cyan('⠦'),
  chalk.magenta('⠧'),
  chalk.yellow('⠇'),
  chalk.green('⠏')
];

/**
 * Status indicator with retro styling
 */
export function getStatusIndicator(status: 'success' | 'error' | 'warning' | 'info'): string {
  const indicators = {
    success: chalk.green('✓'),
    error: chalk.red('✗'),
    warning: chalk.yellow('⚠'),
    info: chalk.cyan('ℹ')
  };
  
  return indicators[status];
}

/**
 * Retro progress bar
 */
export function createProgressBar(progress: number, width: number = 20): string {
  const completed = Math.floor((progress / 100) * width);
  const remaining = width - completed;
  
  const completedBar = chalk.cyan('█'.repeat(completed));
  const remainingBar = chalk.gray('░'.repeat(remaining));
  const percentage = chalk.bold.magenta(`${progress}%`);
  
  return `[${completedBar}${remainingBar}] ${percentage}`;
}

/**
 * Test responsive banner at different widths (for debugging)
 */
export function testResponsiveBanner(): void {
  console.log('\n=== Responsive Banner Test ===\n');
  
  // Test different terminal widths
  const testWidths = [60, 80, 120];
  
  testWidths.forEach(width => {
    console.log(`\n--- Terminal Width: ${width} columns ---`);
    
    // Temporarily override columns for testing
    const originalColumns = process.stdout.columns;
    (process.stdout as any).columns = width;
    
    console.log(getFiveASCIIBanner());
    
    // Restore original
    (process.stdout as any).columns = originalColumns;
  });
}

/**
 * Network status display with colors
 */
export function getNetworkDisplay(network: string): string {
  const networkColors = {
    'wasm': chalk.cyan('WASM'),
    'local': chalk.gray('LOCAL'),
    'devnet': chalk.yellow('DEVNET'),
    'testnet': chalk.magenta('TESTNET'),
    'mainnet': chalk.red('MAINNET')
  };
  
  const displayName = networkColors[network as keyof typeof networkColors] || chalk.white(network.toUpperCase());
  return `[${displayName}]`;
}

/**
 * Error message with retro styling
 */
export function styleError(message: string): string {
  const errorIcon = chalk.red('◢◤');
  const border = chalk.red('━'.repeat(Math.min(message.length + 4, 60)));
  
  return `
${border}
${errorIcon} ${chalk.bold.red('ERROR:')} ${chalk.white(message)}
${border}`;
}

/**
 * Success message with celebration
 */
export function styleSuccess(message: string): string {
  const successIcon = chalk.green('★');
  const celebration = chalk.yellow('◆') + chalk.cyan('◇') + chalk.magenta('◆');
  
  return `${celebration} ${successIcon} ${chalk.bold.green(message)} ${successIcon} ${celebration}`;
}

/**
 * Info box with decorative border
 */
export function createInfoBox(title: string, content: string[]): string {
  const maxWidth = Math.max(title.length, ...content.map(line => line.length)) + 4;
  const topBorder = chalk.cyan('┌' + '─'.repeat(maxWidth - 2) + '┐');
  const bottomBorder = chalk.cyan('└' + '─'.repeat(maxWidth - 2) + '┘');
  const titleLine = chalk.cyan('│ ') + chalk.bold.yellow(title) + ' '.repeat(maxWidth - title.length - 3) + chalk.cyan('│');
  
  const contentLines = content.map(line => 
    chalk.cyan('│ ') + line + ' '.repeat(maxWidth - line.length - 3) + chalk.cyan('│')
  );
  
  return [topBorder, titleLine, ...contentLines, bottomBorder].join('\n');
}

/**
 * Command not found with suggestions
 */
export function styleCommandNotFound(command: string, suggestions: string[]): string {
  const sadFace = chalk.yellow('(╯°□°）╯');
  const suggestion = suggestions.length > 0 
    ? `\n${chalk.cyan('Did you mean:')} ${suggestions.map(s => chalk.yellow(s)).join(', ')}`
    : '';
    
  return `${sadFace} ${chalk.red('Command not found:')} ${chalk.bold(command)}${suggestion}`;
}